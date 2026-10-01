import { listen } from "@tauri-apps/api/event";
import { api, blankProfile, type Profile, type Settings, type Status } from "./api";

const emptySettings: Settings = {
  startMinimized: false,
  closeToTray: true,
  theme: "dark",
  accent: "#5865f2",
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
  version = $state("");
  view = $state<"editor" | "settings">("editor");
  tab = $state<"presence" | "images" | "timer" | "buttons" | "advanced">("presence");
  toast = $state<string | null>(null);

  selected = $derived(this.profiles.find((p) => p.id === this.selectedId) ?? null);

  async init() {
    const s = await api.getState();
    this.profiles = s.profiles;
    this.settings = s.settings;
    this.status = s.status;
    this.activeId = s.activeId;
    this.variables = s.variables;
    this.version = s.version;
    this.selectedId = s.activeId ?? s.profiles[0]?.id ?? null;
    this.ready = true;

    await listen<Status>("rpc-status", (e) => (this.status = e.payload));
    await listen<string | null>("active-changed", (e) => (this.activeId = e.payload));
  }

  notify(message: string) {
    this.toast = message;
    setTimeout(() => {
      if (this.toast === message) this.toast = null;
    }, 3500);
  }

  async create(from?: Profile) {
    const base = from ? { ...structuredClone($state.snapshot(from)), id: "", name: `${from.name} copy` } : blankProfile();
    const saved = await api.saveProfile(base);
    this.profiles.push(saved);
    this.selectedId = saved.id;
    this.view = "editor";
  }

  /** Called by the editor after every autosave so the sidebar stays current. */
  replace(saved: Profile) {
    const i = this.profiles.findIndex((p) => p.id === saved.id);
    if (i >= 0) this.profiles[i] = saved;
  }

  async remove(id: string) {
    await api.deleteProfile(id);
    this.profiles = this.profiles.filter((p) => p.id !== id);
    if (this.selectedId === id) this.selectedId = this.profiles[0]?.id ?? null;
  }

  async toggle(id: string) {
    if (this.activeId === id) await api.deactivate();
    else await api.activate(id);
  }

  async updateSettings(patch: Partial<Settings>) {
    this.settings = { ...this.settings, ...patch };
    await api.saveSettings($state.snapshot(this.settings));
  }
}

export const app = new AppModel();
