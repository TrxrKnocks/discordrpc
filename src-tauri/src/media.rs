//! What the system is playing right now, read from the OS media session.
//!
//! Windows uses the System Media Transport Controls, Linux uses MPRIS and
//! macOS asks Spotify and Music through AppleScript.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

const POLL_EVERY: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub player: String,
    pub playing: bool,
    /// Unix seconds at which the track would have started, given the current position.
    pub started_at: i64,
    /// Unix seconds at which it ends. 0 when the player doesn't report a length.
    pub ends_at: i64,
}

static TRACK: Mutex<Option<Track>> = Mutex::new(None);
static ENABLED: AtomicBool = AtomicBool::new(true);
static COVERS: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);

fn lock<T>(m: &'static Mutex<T>) -> std::sync::MutexGuard<'static, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
    if !on {
        *lock(&TRACK) = None;
    }
}

pub fn track() -> Option<Track> {
    lock(&TRACK).clone()
}

fn unix_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Starts the polling thread. `on_change` runs when the visible track state changes.
pub fn start(on_change: impl Fn(Option<Track>) + Send + 'static) {
    let _ = thread::Builder::new().name("media".into()).spawn(move || {
        platform::init();
        let mut last: Option<Track> = None;
        loop {
            let mut now = if ENABLED.load(Ordering::Relaxed) { platform::current() } else { None };

            // The position is sampled, so the computed start time wobbles by a second.
            // Keep the previous anchor when nothing real changed to avoid pointless updates.
            if let (Some(n), Some(p)) = (now.as_mut(), last.as_ref()) {
                let same = n.title == p.title && n.artist == p.artist && n.playing == p.playing;
                if same && (n.started_at - p.started_at).abs() <= 2 {
                    n.started_at = p.started_at;
                    n.ends_at = p.ends_at;
                }
            }

            if now != last {
                *lock(&TRACK) = now.clone();
                on_change(now.clone());
                last = now;
            }
            thread::sleep(POLL_EVERY);
        }
    });
}

// ---- cover art ----

fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn lookup_cover(artist: &str, title: &str) -> Option<String> {
    let term = encode(&format!("{artist} {title}"));
    let url = format!("https://itunes.apple.com/search?term={term}&media=music&entity=song&limit=1");
    let body: Value = ureq::get(&url).timeout(Duration::from_secs(6)).call().ok()?.into_json().ok()?;
    let small = body["results"][0]["artworkUrl100"].as_str()?;
    Some(small.replace("100x100bb", "600x600bb"))
}

fn cover_key(t: &Track) -> String {
    format!("{}\u{1f}{}", t.artist, t.title)
}

/// The cover for the current track if it's already known. Never touches the network.
pub fn cover_cached() -> Option<String> {
    let t = track()?;
    lock(&COVERS).as_ref()?.get(&cover_key(&t))?.clone()
}

/// Like `cover_cached`, but starts a lookup the first time a track is asked about.
/// The result shows up on a later refresh.
pub fn cover_request() -> Option<String> {
    let t = track()?;
    if t.title.is_empty() {
        return None;
    }
    let key = cover_key(&t);
    {
        let mut guard = lock(&COVERS);
        let map = guard.get_or_insert_with(HashMap::new);
        if let Some(found) = map.get(&key) {
            return found.clone();
        }
        // Mark as in flight so repeated renders don't start more lookups.
        map.insert(key.clone(), None);
    }
    thread::spawn(move || {
        let found = lookup_cover(&t.artist, &t.title);
        lock(&COVERS).get_or_insert_with(HashMap::new).insert(key, found);
    });
    None
}

// ---- platforms ----

#[cfg(windows)]
mod platform {
    use super::{unix_now, Track};
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    /// 100ns ticks between 1601-01-01 (Windows epoch) and 1970-01-01.
    const UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

    pub fn init() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    pub fn current() -> Option<Track> {
        let manager = Manager::RequestAsync().ok()?.get().ok()?;
        let session = manager.GetCurrentSession().ok()?;
        let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;

        let title = props.Title().ok()?.to_string();
        if title.is_empty() {
            return None;
        }
        let playing = session.GetPlaybackInfo().ok()?.PlaybackStatus().ok()? == Status::Playing;

        let (mut started_at, mut ends_at) = (0, 0);
        if let Ok(tl) = session.GetTimelineProperties() {
            let start = tl.StartTime().map(|t| t.Duration).unwrap_or(0);
            let end = tl.EndTime().map(|t| t.Duration).unwrap_or(0);
            let length = (end - start).max(0);
            if length > 0 {
                let mut position = tl.Position().map(|t| t.Duration).unwrap_or(0) - start;
                if playing {
                    // Position is a snapshot; add the time since it was taken.
                    if let Ok(updated) = tl.LastUpdatedTime() {
                        let now_ticks = unix_now() * 10_000_000 + UNIX_EPOCH_TICKS;
                        position += (now_ticks - updated.UniversalTime).max(0);
                    }
                }
                let position = position.clamp(0, length);
                started_at = unix_now() - position / 10_000_000;
                ends_at = started_at + length / 10_000_000;
            }
        }

        let id = session.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default();

        Some(Track {
            title,
            artist: props.Artist().map(|s| s.to_string()).unwrap_or_default(),
            album: props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default(),
            player: friendly_player(&id),
            playing,
            started_at,
            ends_at,
        })
    }

    fn friendly_player(id: &str) -> String {
        let name = id.rsplit('!').next().unwrap_or(id);
        let name = name.strip_suffix(".exe").unwrap_or(name);
        let name = name.rsplit('.').next().unwrap_or(name);
        let mut chars = name.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{unix_now, Track};
    use mpris::{PlaybackStatus, PlayerFinder};

    pub fn init() {}

    pub fn current() -> Option<Track> {
        let player = PlayerFinder::new().ok()?.find_active().ok()?;
        let meta = player.get_metadata().ok()?;
        let title = meta.title()?.to_owned();
        let playing = player.get_playback_status().ok()? == PlaybackStatus::Playing;

        let (mut started_at, mut ends_at) = (0, 0);
        if let Some(length) = meta.length().filter(|l| !l.is_zero()) {
            let position = player.get_position().unwrap_or_default().min(length);
            started_at = unix_now() - position.as_secs() as i64;
            ends_at = started_at + length.as_secs() as i64;
        }

        Some(Track {
            title,
            artist: meta.artists().map(|a| a.join(", ")).unwrap_or_default(),
            album: meta.album_name().unwrap_or_default().to_owned(),
            player: player.identity().to_owned(),
            playing,
            started_at,
            ends_at,
        })
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{unix_now, Track};
    use std::process::Command;

    pub fn init() {}

    pub fn current() -> Option<Track> {
        ["Spotify", "Music"].iter().find_map(|app| query(app))
    }

    /// Asks one player for its state. Output is `state|title|artist|album|position|length`.
    fn query(app: &str) -> Option<Track> {
        let length = if app == "Spotify" { "(duration of current track) / 1000" } else { "(duration of current track)" };
        let script = format!(
            r#"if application "{app}" is running then
                tell application "{app}"
                    if player state is stopped then return ""
                    return (player state as text) & "|" & (name of current track) & "|" & (artist of current track) & "|" & (album of current track) & "|" & (player position as text) & "|" & ({length} as text)
                end tell
            end if
            return """#
        );
        let out = Command::new("osascript").args(["-e", &script]).output().ok()?;
        let text = String::from_utf8(out.stdout).ok()?;
        let parts: Vec<&str> = text.trim().split('|').collect();
        if parts.len() < 6 {
            return None;
        }
        let position: f64 = parts[4].replace(',', ".").parse().unwrap_or(0.0);
        let length: f64 = parts[5].replace(',', ".").parse().unwrap_or(0.0);
        let started_at = unix_now() - position as i64;
        Some(Track {
            title: parts[1].to_owned(),
            artist: parts[2].to_owned(),
            album: parts[3].to_owned(),
            player: app.to_owned(),
            playing: parts[0] == "playing",
            started_at: if length > 0.0 { started_at } else { 0 },
            ends_at: if length > 0.0 { started_at + length as i64 } else { 0 },
        })
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod platform {
    use super::Track;
    pub fn init() {}
    pub fn current() -> Option<Track> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encoding() {
        assert_eq!(encode("a b&c"), "a%20b%26c");
        assert_eq!(encode("Beyoncé"), "Beyonc%C3%A9");
    }
}
