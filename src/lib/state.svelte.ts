import { listen } from "@tauri-apps/api/event";
import { api, blankProfile, type Profile, type Settings, type Status, type Track, type LibraryImage } from "./api";
import { cue, setSoundLevel } from "./sound";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const emptySettings: Settings = {
  startMinimized: false,
  closeToTray: true,
  theme: "dark",
  accent: "#5865f2",
  sound: "soft",
  media: true,
  uploadConsent: false,
  checkUpdates: true,
  resumeLast: false,
  lastProfileId: null,
  defaultClientId: "",
};

class AppModel {
  ready = $state(false);
  profiles = $state<Profile[]>([]);
  settings = $state<Settings>(emptySettings);
  status = $state<Status>({ connected: false, user: null, error: null });
  activeId = $state<string | null>(null);
  selectedId = $state<string | null>(null);
  variables = $state<[string, string][]>([]);
  /** Current value of each variable, refreshed while the editor is open. */
  values = $state<Record<string, string>>({});
  track = $state<Track | null>(null);
  images = $state<LibraryImage[]>([]);

  /** "idle" until the first check; "none" means up to date. */
  updateState = $state<"idle" | "checking" | "none" | "available" | "downloading" | "error">("idle");
  updateInfo = $state<{ version: string; notes: string } | null>(null);
  updateProgress = $state(0);
  updateError = $state("");
  private pending: Update | null = null;
  version = $state("");
  view = $state<"editor" | "settings">("editor");
  toast = $state<string | null>(null);

  /** The text input the user edited last; variable chips insert into it. */
  lastField: HTMLInputElement | null = null;

  selected = $derived(this.profiles.find((p) => p.id === this.selectedId) ?? null);

  async init() {
    const s = await api.getState();
    this.profiles = s.profiles;
    this.settings = s.settings;
    this.status = s.status;
    this.activeId = s.activeId;
    this.variables = s.variables;
    this.track = s.track;
    this.images = s.images;
    this.version = s.version;
    this.selectedId = s.activeId ?? s.profiles[0]?.id ?? null;
    setSoundLevel(s.settings.sound);
    this.ready = true;

    if (s.settings.checkUpdates) setTimeout(() => this.checkForUpdates(true), 4000);

    // Sounds stay quiet for the first moments so a resumed profile doesn't chime at launch.
    let quiet = true;
    setTimeout(() => (quiet = false), 2000);

    await listen<Status>("rpc-status", (e) => {
      const failedBefore = this.status.error;
      this.status = e.payload;
      if (!quiet && this.activeId && e.payload.error && e.payload.error !== failedBefore) cue("error");
    });
    await listen<Track | null>("media-changed", (e) => {
      this.track = e.payload;
      this.refreshValues();
    });
    await listen<string | null>("active-changed", (e) => {
      const was = this.activeId;
      this.activeId = e.payload;
      if (!quiet && was !== e.payload) cue(e.payload ? "start" : "stop");
    });
  }

  async checkForUpdates(silent = false) {
    if (this.updateState === "checking" || this.updateState === "downloading") return;
    this.updateState = "checking";
    this.updateError = "";
    try {
      const update = await check();
      if (update) {
        this.pending = update;
        this.updateInfo = { version: update.version, notes: update.body ?? "" };
        this.updateState = "available";
        if (silent) this.notify(`Version ${update.version} is available. Open Settings to install it.`);
      } else {
        this.updateState = "none";
      }
    } catch (e) {
      // Before the first public release the update feed simply doesn't exist yet.
      this.updateState = silent ? "idle" : "error";
      this.updateError = String(e);
    }
  }

  async installUpdate() {
    if (!this.pending) return;
    this.updateState = "downloading";
    this.updateProgress = 0;
    let total = 0;
    let done = 0;
    try {
      await this.pending.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          done += event.data.chunkLength;
          this.updateProgress = total ? Math.min(1, done / total) : 0;
        }
      });
      await relaunch();
    } catch (e) {
      this.updateState = "error";
      this.updateError = String(e);
    }
  }

  async uploadImage(path: string): Promise<LibraryImage> {
    const image = await api.uploadImage(path);
    this.images = [image, ...this.images.filter((i) => i.url !== image.url)];
    return image;
  }

  async removeImage(url: string) {
    await api.removeImage(url);
    this.images = this.images.filter((i) => i.url !== url);
  }

  async refreshValues() {
    try {
      this.values = Object.fromEntries(await api.variableValues());
    } catch {
      // Only fails while the window is closing.
    }
  }

  notify(message: string) {
    this.toast = message;
    setTimeout(() => {
      if (this.toast === message) this.toast = null;
    }, 3500);
  }

  async create(from?: Profile) {
    const base = from ? { ...structuredClone($state.snapshot(from)), id: "", name: `${from.name} copy` } : blankProfile();
    await this.add(base);
  }

  async add(base: Profile) {
    const saved = await api.saveProfile(base);
    this.profiles.push(saved);
    this.selectedId = saved.id;
    this.view = "editor";
    cue("add");
  }

  insertVariable(name: string) {
    const el = this.lastField;
    if (!el || !el.isConnected) {
      this.notify("Click a line on the card first, then pick a variable.");
      return;
    }
    const start = el.selectionStart ?? el.value.length;
    const end = el.selectionEnd ?? start;
    el.focus();
    el.setRangeText(`{${name}}`, start, end, "end");
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }

  /** Called by the editor after every autosave so the sidebar stays current. */
  replace(saved: Profile) {
    const i = this.profiles.findIndex((p) => p.id === saved.id);
    if (i >= 0) this.profiles[i] = saved;
  }

  async remove(id: string) {
    await api.deleteProfile(id);
    cue("remove");
    this.profiles = this.profiles.filter((p) => p.id !== id);
    if (this.selectedId === id) this.selectedId = this.profiles[0]?.id ?? null;
  }

  async toggle(id: string) {
    if (this.activeId === id) await api.deactivate();
    else await api.activate(id);
  }

  async updateSettings(patch: Partial<Settings>) {
    this.settings = { ...this.settings, ...patch };
    if (patch.sound) setSoundLevel(patch.sound);
    await api.saveSettings($state.snapshot(this.settings));
  }
}

export const app = new AppModel();
