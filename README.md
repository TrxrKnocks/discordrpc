# DiscordRPC

A small desktop app for setting your Discord Rich Presence. You edit a card that looks like the one Discord shows, press Start, and it's on your profile. No developer portal, no scripts.

Runs on Windows, Linux and macOS. Built with Tauri, so the installer is a few megabytes and it idles at a small memory footprint.

## What it does

- Edit the activity type, name, two text lines, images, timer, party size and up to two buttons, right on the card
- Keep as many profiles as you like, start from a template, search, tag, import and export them
- Put variables in any line: `{time}`, `{date}`, `{uptime}`, `{cpu}`, `{ram}`, `{os}` and so on, refreshed while the profile is live
- Follow what you're playing with `{title}`, `{artist}`, `{album}`, `{player}` and `{cover}`, including a progress bar
- Upload an image from disk instead of hunting for a link
- Tray icon, launch at sign-in, start hidden, close to tray
- Light and dark themes, accent colours, optional sound effects

## Install

Grab the installer for your system from the [releases page](https://github.com/TrxrKnocks/discordrpc/releases). There is also a portable zip for Windows.

The builds are not code signed yet, so Windows SmartScreen and macOS Gatekeeper will warn the first time you open the app. On Windows choose "More info", then "Run anyway". On macOS, right-click the app and choose Open.

Discord's desktop app has to be running. The app finds it on its own and reconnects if Discord restarts.

## Using it

1. Create a profile from the **+** button and pick a template.
2. Click the text on the card to change it. Click the artwork to set images.
3. Press **Start**. Edits apply while it's live.

The name after "Playing" comes from the **Name** field. The app sends it with the activity, so you don't need your own Discord application. A default application ID is built in, and you can set your own under Settings if you prefer.

Buttons are shown to other people only. Discord doesn't show them on your own profile.

## Network use

The app talks to the internet in three cases, all optional:

- **Cover art**: if a profile uses `{cover}`, it asks the iTunes Search API for album art matching the current track.
- **Image uploads**: only when you press Upload. The file goes to [catbox.moe](https://catbox.moe), a free public host, and you're asked before the first one.
- **Update checks**: it looks at this repository's releases on startup. You can turn that off in Settings.

Now-playing information is read locally from the operating system and never leaves your machine, apart from the cover lookup above.

## Building from source

You need Node 22, pnpm, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform. On Linux, `libdbus-1-dev` is needed too for media detection.

```sh
pnpm install
pnpm tauri dev      # run it
pnpm tauri build    # make an installer
```

Release builds also produce signed update files, which needs the project's signing key. For a local build without it:

```sh
pnpm tauri build --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

Checks:

```sh
pnpm check
cargo test --manifest-path src-tauri/Cargo.toml
```

## Platform notes

Now playing reads the Windows media session, MPRIS on Linux, and Spotify or Music through AppleScript on macOS. The Windows path gets the most use. The Linux and macOS ones are newer, so reports are welcome.

## License

MIT. See [LICENSE](LICENSE). This project isn't affiliated with or endorsed by Discord Inc.
