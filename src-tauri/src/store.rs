//! JSON files in the app data directory.

use std::fs;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const DEFAULT_CLIENT_ID: &str = "1555153922883325972";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub start_minimized: bool,
    pub close_to_tray: bool,
    /// "dark", "light" or "system"
    pub theme: String,
    pub accent: String,
    /// "off", "soft" or "normal"
    pub sound: String,
    /// Read the system's now-playing info for the media variables.
    pub media: bool,
    /// The user agreed to uploading images to the public host.
    pub upload_consent: bool,
    pub resume_last: bool,
    pub last_profile_id: Option<String>,
    /// Application id used when a profile doesn't set its own.
    pub default_client_id: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            start_minimized: false,
            close_to_tray: true,
            theme: "dark".into(),
            accent: "#5865f2".into(),
            sound: "soft".into(),
            media: true,
            upload_consent: false,
            resume_last: false,
            last_profile_id: None,
            default_client_id: DEFAULT_CLIENT_ID.into(),
        }
    }
}

/// An image uploaded from this app, kept so it can be reused.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LibraryImage {
    pub url: String,
    pub name: String,
}

impl Default for LibraryImage {
    fn default() -> Self {
        LibraryImage { url: String::new(), name: String::new() }
    }
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        let _ = fs::create_dir_all(&dir);
        Store { dir }
    }

    pub fn load<T: DeserializeOwned + Default>(&self, name: &str) -> T {
        let path = self.dir.join(name);
        let Ok(raw) = fs::read_to_string(&path) else { return T::default() };
        match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                // Keep the unreadable file around instead of overwriting it.
                let _ = fs::rename(&path, path.with_extension("json.bak"));
                T::default()
            }
        }
    }

    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> Result<(), String> {
        let path = self.dir.join(name);
        let tmp = path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
        fs::write(&tmp, data).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }
}
