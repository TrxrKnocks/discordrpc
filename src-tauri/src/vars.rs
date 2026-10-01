//! Template variables usable in any text field, e.g. `{time}` or `{cpu}`.

use std::collections::HashMap;
use std::sync::Mutex;

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
];

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

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
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        map.insert("cpu", format!("{:.0}", sys.global_cpu_usage()));
        let (used, total) = (sys.used_memory() as f64, sys.total_memory() as f64);
        if total > 0.0 {
            map.insert("ram", format!("{:.0}", used / total * 100.0));
            map.insert("ram_used", format!("{:.1}", used / 1e9));
            map.insert("ram_total", format!("{:.0}", total / 1e9));
        }
        map.insert("uptime", fmt_duration(System::uptime()));
        map.insert("os", System::name().unwrap_or_default());

        Vars(map)
    }

    pub fn set(&mut self, key: &'static str, value: String) {
        self.0.insert(key, value);
    }

    /// Current values in catalog order, skipping names with nothing to show.
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        CATALOG
            .iter()
            .filter_map(|(name, _)| self.0.get(name).map(|v| (*name, v.clone())))
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
    fn duration_formats() {
        assert_eq!(fmt_duration(59), "0m");
        assert_eq!(fmt_duration(3 * 3600 + 5 * 60), "3h 5m");
        assert_eq!(fmt_duration(2 * 86_400 + 4 * 3600), "2d 4h");
    }
}
