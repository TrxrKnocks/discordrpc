mod ipc;
mod media;
mod presence;
mod store;
mod upload;
mod vars;

use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use ipc::{Rpc, Status};
use presence::{resolve, to_activity, Profile, Resolved};
use store::{LibraryImage, Settings, Store};
use vars::Vars;

const REFRESH_EVERY: Duration = Duration::from_secs(5);

struct Active {
    profile: Profile,
    started_at: i64,
}

struct AppState {
    store: Store,
    profiles: Mutex<Vec<Profile>>,
    images: Mutex<Vec<LibraryImage>>,
    settings: Mutex<Settings>,
    active: Mutex<Option<Active>>,
    status: Mutex<Status>,
    rpc: Rpc,
}

fn unix_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl AppState {
    fn client_id_for(&self, p: &Profile) -> String {
        let id = p.client_id.trim();
        if id.is_empty() {
            lock(&self.settings).default_client_id.clone()
        } else {
            id.to_owned()
        }
    }

    /// Re-renders the active profile and hands it to the IPC worker.
    fn push(&self) {
        let active = lock(&self.active);
        let Some(a) = active.as_ref() else { return };
        let track = media::track();
        if a.profile.hide_when_idle && !track.as_ref().is_some_and(|t| t.playing) {
            self.rpc.clear();
            return;
        }
        let r = resolve(&a.profile, &Vars::snapshot());
        let activity = to_activity(&a.profile, &r, a.started_at, track.as_ref());
        self.rpc.set(&self.client_id_for(&a.profile), activity);
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StateDto {
    profiles: Vec<Profile>,
    images: Vec<LibraryImage>,
    settings: Settings,
    status: Status,
    active_id: Option<String>,
    variables: Vec<(&'static str, &'static str)>,
    track: Option<media::Track>,
    version: &'static str,
}

#[tauri::command]
fn get_state(state: State<AppState>) -> StateDto {
    StateDto {
        profiles: lock(&state.profiles).clone(),
        images: lock(&state.images).clone(),
        settings: lock(&state.settings).clone(),
        status: lock(&state.status).clone(),
        active_id: lock(&state.active).as_ref().map(|a| a.profile.id.clone()),
        variables: vars::CATALOG.to_vec(),
        track: media::track(),
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
fn save_profile(mut profile: Profile, state: State<AppState>) -> Result<Profile, String> {
    if profile.id.is_empty() {
        profile.id = uuid::Uuid::new_v4().to_string();
    }
    {
        let mut profiles = lock(&state.profiles);
        match profiles.iter_mut().find(|p| p.id == profile.id) {
            Some(slot) => *slot = profile.clone(),
            None => profiles.push(profile.clone()),
        }
        state.store.save("profiles.json", &*profiles)?;
    }

    let is_active = {
        let mut active = lock(&state.active);
        match active.as_mut().filter(|a| a.profile.id == profile.id) {
            Some(a) => {
                a.profile = profile.clone();
                true
            }
            None => false,
        }
    };
    if is_active {
        state.push();
    }
    Ok(profile)
}

#[tauri::command]
fn delete_profile(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    {
        let mut profiles = lock(&state.profiles);
        profiles.retain(|p| p.id != id);
        state.store.save("profiles.json", &*profiles)?;
    }
    if lock(&state.active).as_ref().is_some_and(|a| a.profile.id == id) {
        deactivate_inner(&app, &state);
    }
    Ok(())
}

fn activate_inner(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let profile = lock(&state.profiles)
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or("profile not found")?;
    *lock(&state.active) = Some(Active { profile, started_at: unix_now() });
    state.push();

    let mut settings = lock(&state.settings);
    settings.last_profile_id = Some(id.to_owned());
    let _ = state.store.save("settings.json", &*settings);
    let _ = app.emit("active-changed", Some(id));
    Ok(())
}

fn deactivate_inner(app: &AppHandle, state: &AppState) {
    *lock(&state.active) = None;
    state.rpc.clear();
    let _ = app.emit("active-changed", None::<String>);
}

#[tauri::command]
fn activate(id: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    activate_inner(&app, &state, &id)
}

#[tauri::command]
fn deactivate(app: AppHandle, state: State<AppState>) {
    deactivate_inner(&app, &state);
}

#[tauri::command]
async fn upload_image(path: String, state: State<'_, AppState>) -> Result<LibraryImage, String> {
    // The upload blocks on the network, so keep it off the async runtime threads.
    let (url, name) = tauri::async_runtime::spawn_blocking(move || upload::upload(std::path::Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;

    let image = LibraryImage { url, name };
    let mut images = lock(&state.images);
    images.retain(|i| i.url != image.url);
    images.insert(0, image.clone());
    images.truncate(60);
    state.store.save("images.json", &*images)?;
    Ok(image)
}

#[tauri::command]
fn remove_image(url: String, state: State<AppState>) -> Result<(), String> {
    let mut images = lock(&state.images);
    images.retain(|i| i.url != url);
    state.store.save("images.json", &*images)
}

#[tauri::command]
fn variable_values() -> Vec<(&'static str, String)> {
    Vars::snapshot().entries()
}

#[tauri::command]
fn preview(profile: Profile) -> Resolved {
    resolve(&profile, &Vars::snapshot())
}

#[tauri::command]
fn save_settings(settings: Settings, state: State<AppState>) -> Result<(), String> {
    media::set_enabled(settings.media);
    let mut slot = lock(&state.settings);
    *slot = settings;
    state.store.save("settings.json", &*slot)
}

#[tauri::command]
fn export_profiles(path: String, ids: Option<Vec<String>>, state: State<AppState>) -> Result<usize, String> {
    let profiles = lock(&state.profiles);
    let chosen: Vec<&Profile> = profiles
        .iter()
        .filter(|p| ids.as_ref().map_or(true, |ids| ids.contains(&p.id)))
        .collect();
    let doc = serde_json::json!({ "format": "discordrpc-profiles", "version": 1, "profiles": chosen });
    let data = serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(chosen.len())
}

#[tauri::command]
fn import_profiles(path: String, state: State<AppState>) -> Result<Vec<Profile>, String> {
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let doc: serde_json::Value = serde_json::from_str(&raw).map_err(|_| "not a JSON file")?;

    // Accept both the wrapped export format and a bare array or single profile.
    let list = match (doc.get("profiles"), &doc) {
        (Some(p), _) => p.clone(),
        (None, serde_json::Value::Array(_)) => doc.clone(),
        _ => serde_json::Value::Array(vec![doc.clone()]),
    };
    let incoming: Vec<Profile> = serde_json::from_value(list).map_err(|e| format!("invalid profile data: {e}"))?;
    if incoming.is_empty() {
        return Err("the file contains no profiles".into());
    }

    let mut added = Vec::with_capacity(incoming.len());
    let mut profiles = lock(&state.profiles);
    for mut p in incoming {
        // New ids so imports never overwrite existing profiles.
        p.id = uuid::Uuid::new_v4().to_string();
        profiles.push(p.clone());
        added.push(p);
    }
    state.store.save("profiles.json", &*profiles)?;
    Ok(added)
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Open DiscordRPC", true, None::<&str>)?;
    let clear = MenuItem::with_id(app, "clear", "Clear presence", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &clear, &sep, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("DiscordRPC")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "clear" => deactivate_inner(app, &app.state::<AppState>()),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_profile,
            delete_profile,
            activate,
            deactivate,
            preview,
            variable_values,
            upload_image,
            remove_image,
            save_settings,
            export_profiles,
            import_profiles,
        ])
        .setup(|app| {
            vars::start_cpu_sampler();
            let dir = app.path().app_data_dir()?;
            let store = Store::new(dir);
            let settings: Settings = store.load("settings.json");
            let profiles: Vec<Profile> = store.load("profiles.json");
            let images: Vec<LibraryImage> = store.load("images.json");

            let handle = app.handle().clone();
            let rpc = Rpc::spawn(settings.default_client_id.clone(), move |status| {
                // The worker reports before the state is registered on startup.
                if let Some(state) = handle.try_state::<AppState>() {
                    *lock(&state.status) = status.clone();
                }
                let _ = handle.emit("rpc-status", status);
            });

            media::set_enabled(settings.media);
            let resume = settings.resume_last.then(|| settings.last_profile_id.clone()).flatten();
            let hidden = settings.start_minimized || std::env::args().any(|a| a == "--minimized");

            app.manage(AppState {
                store,
                profiles: Mutex::new(profiles),
                images: Mutex::new(images),
                settings: Mutex::new(settings),
                active: Mutex::new(None),
                status: Mutex::new(Status::default()),
                rpc,
            });

            let handle = app.handle().clone();
            thread::Builder::new().name("refresh".into()).spawn(move || loop {
                thread::sleep(REFRESH_EVERY);
                handle.state::<AppState>().push();
            })?;

            let handle = app.handle().clone();
            media::start(move |track| {
                let _ = handle.emit("media-changed", track);
                if let Some(state) = handle.try_state::<AppState>() {
                    state.push();
                }
            });

            build_tray(app.handle())?;

            if let Some(id) = resume {
                let _ = activate_inner(app.handle(), &app.state::<AppState>(), &id);
            }
            if !hidden {
                show_main(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let to_tray = lock(&window.app_handle().state::<AppState>().settings).close_to_tray;
                if to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to build the application");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            app.state::<AppState>().rpc.shutdown();
        }
    });
}
