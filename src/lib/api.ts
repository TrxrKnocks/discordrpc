import { invoke } from "@tauri-apps/api/core";

export interface Button {
  label: string;
  url: string;
}

export interface Timestamp {
  kind: "none" | "elapsed" | "since" | "countdown";
  value: number;
}

export interface Profile {
  id: string;
  name: string;
  tags: string[];
  clientId: string;
  activityType: 0 | 2 | 3 | 5;
  nameOverride: string;
  details: string;
  state: string;
  statusDisplay: 0 | 1 | 2;
  largeImage: string;
  largeText: string;
  smallImage: string;
  smallText: string;
  timestamp: Timestamp;
  partyCurrent: number;
  partyMax: number;
  buttons: Button[];
}

export interface Settings {
  startMinimized: boolean;
  closeToTray: boolean;
  theme: "dark" | "light" | "system";
  accent: string;
  resumeLast: boolean;
  lastProfileId: string | null;
  defaultClientId: string;
}

export interface DiscordUser {
  id: string;
  username: string;
  globalName: string | null;
  avatar: string | null;
}

export interface Status {
  connected: boolean;
  user: DiscordUser | null;
  error: string | null;
}

export interface Resolved {
  nameOverride: string;
  details: string;
  state: string;
  largeText: string;
  smallText: string;
  buttons: Button[];
}

export interface AppStateDto {
  profiles: Profile[];
  settings: Settings;
  status: Status;
  activeId: string | null;
  variables: [string, string][];
  version: string;
}

export function blankProfile(): Profile {
  return {
    id: "",
    name: "New profile",
    tags: [],
    clientId: "",
    activityType: 0,
    nameOverride: "",
    details: "",
    state: "",
    statusDisplay: 0,
    largeImage: "",
    largeText: "",
    smallImage: "",
    smallText: "",
    timestamp: { kind: "elapsed", value: 0 },
    partyCurrent: 0,
    partyMax: 0,
    buttons: [],
  };
}

export const api = {
  getState: () => invoke<AppStateDto>("get_state"),
  saveProfile: (profile: Profile) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  activate: (id: string) => invoke<void>("activate", { id }),
  deactivate: () => invoke<void>("deactivate"),
  preview: (profile: Profile) => invoke<Resolved>("preview", { profile }),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  exportProfiles: (path: string, ids: string[] | null) =>
    invoke<number>("export_profiles", { path, ids }),
  importProfiles: (path: string) => invoke<Profile[]>("import_profiles", { path }),
};
