//! Profile model and conversion into a Discord activity payload.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::media::Track;
use crate::vars::Vars;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Button {
    pub label: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Timestamp {
    /// "none", "elapsed", "since" (value = unix seconds), "countdown" (value = seconds)
    /// or "media" (follows the playing track)
    pub kind: String,
    pub value: i64,
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp { kind: "elapsed".into(), value: 0 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    /// Discord application id the presence is sent under.
    pub client_id: String,
    /// 0 playing, 2 listening, 3 watching, 5 competing
    pub activity_type: u8,
    /// Overrides the application name when Discord honours it. Empty = unused.
    pub name_override: String,
    pub details: String,
    pub state: String,
    /// 0 app name, 1 state, 2 details
    pub status_display: u8,
    pub large_image: String,
    pub large_text: String,
    pub small_image: String,
    pub small_text: String,
    pub timestamp: Timestamp,
    pub party_current: u32,
    pub party_max: u32,
    pub buttons: Vec<Button>,
    /// Clear the presence while no media is playing. Meant for profiles built on `{title}` and friends.
    pub hide_when_idle: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            id: String::new(),
            name: "Untitled".into(),
            tags: Vec::new(),
            client_id: String::new(),
            activity_type: 0,
            name_override: String::new(),
            details: String::new(),
            state: String::new(),
            status_display: 0,
            large_image: String::new(),
            large_text: String::new(),
            small_image: String::new(),
            small_text: String::new(),
            timestamp: Timestamp::default(),
            party_current: 0,
            party_max: 0,
            buttons: Vec::new(),
            hide_when_idle: false,
        }
    }
}

/// The profile with every text field resolved, shown by the preview card.
#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Resolved {
    pub name_override: String,
    pub details: String,
    pub state: String,
    pub large_image: String,
    pub small_image: String,
    pub large_text: String,
    pub small_text: String,
    pub buttons: Vec<Button>,
}

pub fn resolve(p: &Profile, vars: &Vars) -> Resolved {
    Resolved {
        name_override: vars.render(&p.name_override),
        details: vars.render(&p.details),
        state: vars.render(&p.state),
        large_image: vars.render(&p.large_image),
        small_image: vars.render(&p.small_image),
        large_text: vars.render(&p.large_text),
        small_text: vars.render(&p.small_text),
        buttons: p
            .buttons
            .iter()
            .map(|b| Button { label: vars.render(&b.label), url: b.url.trim().to_owned() })
            .collect(),
    }
}

/// Discord rejects text shorter than 2 or longer than 128 characters.
fn text(s: &str) -> Option<String> {
    let s = s.trim();
    let n = s.chars().count();
    (n >= 2).then(|| s.chars().take(128).collect())
}

fn image(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_owned())
}

/// `started_at` is the unix time the profile was activated; it anchors the
/// elapsed and countdown timers so they don't restart on every refresh.
pub fn to_activity(p: &Profile, r: &Resolved, started_at: i64, track: Option<&Track>) -> Value {
    let mut a = Map::new();

    if matches!(p.activity_type, 2 | 3 | 5) {
        a.insert("type".into(), json!(p.activity_type));
    }
    if let Some(v) = text(&r.name_override) {
        a.insert("name".into(), json!(v));
    }
    if let Some(v) = text(&r.details) {
        a.insert("details".into(), json!(v));
    }
    if let Some(v) = text(&r.state) {
        a.insert("state".into(), json!(v));
    }
    if matches!(p.status_display, 1 | 2) {
        a.insert("status_display_type".into(), json!(p.status_display));
    }

    let mut assets = Map::new();
    if let Some(v) = image(&r.large_image) {
        assets.insert("large_image".into(), json!(v));
        if let Some(t) = text(&r.large_text) {
            assets.insert("large_text".into(), json!(t));
        }
    }
    if let Some(v) = image(&r.small_image) {
        assets.insert("small_image".into(), json!(v));
        if let Some(t) = text(&r.small_text) {
            assets.insert("small_text".into(), json!(t));
        }
    }
    if !assets.is_empty() {
        a.insert("assets".into(), Value::Object(assets));
    }

    match p.timestamp.kind.as_str() {
        "elapsed" => {
            a.insert("timestamps".into(), json!({ "start": started_at }));
        }
        "since" if p.timestamp.value > 0 => {
            a.insert("timestamps".into(), json!({ "start": p.timestamp.value }));
        }
        "countdown" if p.timestamp.value > 0 => {
            a.insert("timestamps".into(), json!({ "end": started_at + p.timestamp.value }));
        }
        "media" => {
            if let Some(t) = track.filter(|t| t.playing && t.started_at > 0) {
                let mut ts = json!({ "start": t.started_at });
                if t.ends_at > t.started_at {
                    ts["end"] = json!(t.ends_at);
                }
                a.insert("timestamps".into(), ts);
            }
        }
        _ => {}
    }

    if p.party_max > 0 {
        let cur = p.party_current.clamp(1, p.party_max);
        a.insert("party".into(), json!({ "id": p.id, "size": [cur, p.party_max] }));
    }

    let buttons: Vec<Value> = r
        .buttons
        .iter()
        .filter(|b| !b.label.trim().is_empty() && is_http_url(&b.url))
        .take(2)
        .map(|b| json!({ "label": b.label.trim().chars().take(32).collect::<String>(), "url": b.url }))
        .collect();
    if !buttons.is_empty() {
        a.insert("buttons".into(), Value::Array(buttons));
    }

    Value::Object(a)
}

pub fn is_http_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("https://") || s.starts_with("http://")) && s.len() > 8 && s.len() <= 512
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(p: &Profile) -> Value {
        let vars = Vars::default();
        to_activity(p, &resolve(p, &vars), 1000, None)
    }

    #[test]
    fn drops_short_text() {
        let p = Profile { details: "a".into(), state: "ok".into(), ..Default::default() };
        let a = build(&p);
        assert!(a.get("details").is_none());
        assert_eq!(a["state"], "ok");
    }

    #[test]
    fn countdown_uses_start_anchor() {
        let mut p = Profile::default();
        p.timestamp = Timestamp { kind: "countdown".into(), value: 600 };
        assert_eq!(build(&p)["timestamps"]["end"], 1600);
    }

    #[test]
    fn invalid_buttons_are_skipped() {
        let mut p = Profile::default();
        p.buttons = vec![
            Button { label: "Bad".into(), url: "nope".into() },
            Button { label: "Good".into(), url: "https://example.com".into() },
        ];
        let a = build(&p);
        assert_eq!(a["buttons"].as_array().unwrap().len(), 1);
        assert_eq!(a["buttons"][0]["label"], "Good");
    }

    #[test]
    fn media_timer_follows_the_track() {
        let mut p = Profile::default();
        p.timestamp = Timestamp { kind: "media".into(), value: 0 };
        let track = Track { playing: true, started_at: 500, ends_at: 740, ..Default::default() };
        let vars = Vars::default();
        let a = to_activity(&p, &resolve(&p, &vars), 1000, Some(&track));
        assert_eq!(a["timestamps"]["start"], 500);
        assert_eq!(a["timestamps"]["end"], 740);

        let paused = Track { playing: false, ..track };
        let a = to_activity(&p, &resolve(&p, &vars), 1000, Some(&paused));
        assert!(a.get("timestamps").is_none());
    }

    #[test]
    fn default_type_is_omitted() {
        assert!(build(&Profile::default()).get("type").is_none());
    }
}
