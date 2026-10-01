import { blankProfile, type Profile } from "./api";

export interface Template {
  id: string;
  label: string;
  blurb: string;
  make: () => Profile;
}

function from(label: string, patch: Partial<Profile>): Profile {
  return { ...blankProfile(), name: label, ...patch };
}

export const templates: Template[] = [
  {
    id: "game",
    label: "Playing a game",
    blurb: "Game name, a status line and a timer",
    make: () =>
      from("Playing a game", {
        nameOverride: "My Game",
        details: "Ranked match",
        state: "Level 24",
        timestamp: { kind: "elapsed", value: 0 },
      }),
  },
  {
    id: "music",
    label: "Listening to music",
    blurb: "Shows as 'Listening to'",
    make: () =>
      from("Listening to music", {
        activityType: 2,
        nameOverride: "Lo-fi beats",
        details: "Late night playlist",
        state: "by Various Artists",
        timestamp: { kind: "none", value: 0 },
      }),
  },
  {
    id: "watch",
    label: "Watching something",
    blurb: "Shows as 'Watching'",
    make: () =>
      from("Watching something", {
        activityType: 3,
        nameOverride: "A good show",
        details: "Season 1",
        state: "Episode 4",
        timestamp: { kind: "elapsed", value: 0 },
      }),
  },
  {
    id: "work",
    label: "Working",
    blurb: "Include your local time",
    make: () =>
      from("Working", {
        nameOverride: "Coding",
        details: "Heads down on a project",
        state: "Local time {time}",
        timestamp: { kind: "elapsed", value: 0 },
      }),
  },
  {
    id: "nowplaying",
    label: "Now playing",
    blurb: "Follows whatever you're listening to",
    make: () =>
      from("Now playing", {
        activityType: 2,
        nameOverride: "{player}",
        details: "{title}",
        state: "by {artist}",
        largeImage: "{cover}",
        largeText: "{album}",
        timestamp: { kind: "media", value: 0 },
        hideWhenIdle: true,
      }),
  },
  {
    id: "system",
    label: "System stats",
    blurb: "CPU, memory and uptime, updated live",
    make: () =>
      from("System stats", {
        nameOverride: "My PC",
        details: "CPU {cpu}%  |  RAM {ram}%",
        state: "Up for {uptime}",
        timestamp: { kind: "none", value: 0 },
      }),
  },
  {
    id: "blank",
    label: "Blank",
    blurb: "Start from nothing",
    make: () => blankProfile(),
  },
];
