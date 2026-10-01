//! Template variables usable in any text field, e.g. `{time}` or `{cpu}`.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use sysinfo::System;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

const TIME_24: &[FormatItem<'static>] = format_description!("[hour]:[minute]");
const TIME_12: &[FormatItem<'static>] =
    format_description!("[hour repr:12 padding:none]:[minute] [period]");
const DATE: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");
const WEEKDAY: &[FormatItem<'static>] = format_description!("[weekday]");

/// Names the editor offers in its variable picker, with a short description.
pub const CATALOG: &[(&str, &str)] = &[
    ("time", "Local time, 24-hour"),
    ("time12", "Local time, 12-hour"),
    ("date", "Local date, YYYY-MM-DD"),
    ("weekday", "Day of the week"),
    ("uptime", "System uptime"),
    ("cpu", "CPU usage in percent"),
    ("ram", "Memory usage in percent"),
    ("ram_used", "Memory in use, GB"),
    ("ram_total", "Installed memory, GB"),
    ("os", "Operating system name"),
    ("title", "Now playing: track title"),
    ("artist", "Now playing: artist"),
    ("album", "Now playing: album"),
    ("player", "Now playing: app name"),
    ("cover", "Now playing: cover art link"),
];

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
/// CPU usage samples, in percent, newest last.
static CPU_SAMPLES: Mutex<VecDeque<f32>> = Mutex::new(VecDeque::new());
const CPU_WINDOW: usize = 3;

/// CPU usage is a rate, so it needs two readings with a gap between them.
/// One thread samples once a second; everything else reads the average.
pub fn start_cpu_sampler() {
    let _ = thread::Builder::new().name("cpu-sampler".into()).spawn(|| {
        let mut read = cpu_reader();
        read();
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Some(pct) = read() {
                let mut samples = CPU_SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
                samples.push_back(pct);
                if samples.len() > CPU_WINDOW {
                    samples.pop_front();
                }
            }
        }
    });
}

/// Returns a closure giving CPU usage since its previous call.
#[cfg(windows)]
fn cpu_reader() -> Box<dyn FnMut() -> Option<f32> + Send> {
    // Read the kernel's own counters. sysinfo goes through the performance
    // counter service, which is disabled or broken on some Windows installs.
    let mut prev = system_times();
    Box::new(move || {
        let now = system_times()?;
        let usage = prev.and_then(|p| usage_between(p, now));
        prev = Some(now);
        usage
    })
}

#[cfg(not(windows))]
fn cpu_reader() -> Box<dyn FnMut() -> Option<f32> + Send> {
    let mut sys = System::new();
    // A fresh System knows no CPUs; refresh_cpu_usage only updates known ones.
    sys.refresh_cpu_all();
    Box::new(move || {
        sys.refresh_cpu_usage();
        Some(sys.global_cpu_usage())
    })
}

/// (idle, kernel, user) in 100ns ticks. Kernel time includes idle time.
#[cfg(windows)]
fn system_times() -> Option<(u64, u64, u64)> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetSystemTimes;

    let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut idle, mut kernel, mut user) = (zero, zero, zero);
    // SAFETY: the three pointers are valid for writes for the duration of the call.
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let ticks = |f: FILETIME| (u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime);
    Some((ticks(idle), ticks(kernel), ticks(user)))
}

#[cfg(any(windows, test))]
fn usage_between(prev: (u64, u64, u64), now: (u64, u64, u64)) -> Option<f32> {
    let idle = now.0.checked_sub(prev.0)?;
    let busy_and_idle = now.1.checked_sub(prev.1)?.checked_add(now.2.checked_sub(prev.2)?)?;
    if busy_and_idle == 0 {
        return None;
    }
    Some(((1.0 - idle as f64 / busy_and_idle as f64) * 100.0).clamp(0.0, 100.0) as f32)
}

fn cpu_percent() -> Option<f32> {
    let samples = CPU_SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    (!samples.is_empty()).then(|| samples.iter().sum::<f32>() / samples.len() as f32)
}

#[derive(Default)]
pub struct Vars(HashMap<&'static str, String>);

impl Vars {
    pub fn snapshot() -> Vars {
        let mut map = HashMap::new();

        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        map.insert("time", now.format(TIME_24).unwrap_or_default());
        map.insert("time12", now.format(TIME_12).unwrap_or_default());
        map.insert("date", now.format(DATE).unwrap_or_default());
        map.insert("weekday", now.format(WEEKDAY).unwrap_or_default());

        let mut guard = SYSTEM.lock().unwrap_or_else(|e| e.into_inner());
        let sys = guard.get_or_insert_with(System::new);
        sys.refresh_memory();

        if let Some(cpu) = cpu_percent() {
            map.insert("cpu", format!("{cpu:.0}"));
        }
        let (used, total) = (sys.used_memory() as f64, sys.total_memory() as f64);
        if total > 0.0 {
            map.insert("ram", format!("{:.0}", used / total * 100.0));
            map.insert("ram_used", format!("{:.1}", used / 1e9));
            map.insert("ram_total", format!("{:.0}", total / 1e9));
        }
        map.insert("uptime", fmt_duration(System::uptime()));
        map.insert("os", System::name().unwrap_or_default());

        // Empty while nothing plays, so lines that use them simply disappear.
        let track = crate::media::track().unwrap_or_default();
        map.insert("title", track.title);
        map.insert("artist", track.artist);
        map.insert("album", track.album);
        map.insert("player", track.player);

        Vars(map)
    }

    #[cfg(test)]
    pub fn set(&mut self, key: &'static str, value: String) {
        self.0.insert(key, value);
    }

    /// Current values in catalog order, skipping names with nothing to show.
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        CATALOG
            .iter()
            .filter_map(|(name, _)| match *name {
                "cover" => Some(("cover", crate::media::cover_cached().unwrap_or_default())),
                _ => self.0.get(name).map(|v| (*name, v.clone())),
            })
            .collect()
    }

    /// Replaces `{name}` with its value. Unknown names are left untouched so
    /// typos stay visible in the preview.
    pub fn render(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            match after.find('}') {
                Some(end) => {
                    let key = &after[..end];
                    match self.0.get(key) {
                        Some(v) => out.push_str(v),
                        // Covers cost a network lookup, so only fetch them when a field asks.
                        None if key == "cover" => out.push_str(&crate::media::cover_request().unwrap_or_default()),
                        None => {
                            out.push('{');
                            out.push_str(key);
                            out.push('}');
                        }
                    }
                    rest = &after[end + 1..];
                }
                None => {
                    out.push_str(&rest[start..]);
                    rest = "";
                }
            }
        }
        out.push_str(rest);
        out
    }
}

fn fmt_duration(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs / 3600 % 24, secs / 60 % 60);
    match (d, h) {
        (0, 0) => format!("{m}m"),
        (0, _) => format!("{h}h {m}m"),
        _ => format!("{d}d {h}h"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&'static str, &str)]) -> Vars {
        let mut v = Vars::default();
        for (k, val) in pairs {
            v.set(k, val.to_string());
        }
        v
    }

    #[test]
    fn replaces_known_and_keeps_unknown() {
        let v = vars(&[("time", "12:30")]);
        assert_eq!(v.render("at {time} {nope}"), "at 12:30 {nope}");
    }

    #[test]
    fn handles_unclosed_brace() {
        let v = vars(&[("a", "x")]);
        assert_eq!(v.render("{a} and {b"), "x and {b");
    }

    #[test]
    fn cpu_usage_from_system_times() {
        // 1000 ticks of kernel+user, 750 of them idle: 25% busy.
        assert_eq!(usage_between((0, 0, 0), (750, 800, 200)), Some(25.0));
        // No time passed: nothing to report.
        assert_eq!(usage_between((5, 5, 5), (5, 5, 5)), None);
        // Counters going backwards are ignored rather than panicking.
        assert_eq!(usage_between((10, 10, 10), (5, 5, 5)), None);
    }

    #[test]
    fn duration_formats() {
        assert_eq!(fmt_duration(59), "0m");
        assert_eq!(fmt_duration(3 * 3600 + 5 * 60), "3h 5m");
        assert_eq!(fmt_duration(2 * 86_400 + 4 * 3600), "2d 4h");
    }
}
