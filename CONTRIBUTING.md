# Contributing

Bug reports and pull requests are welcome.

## Setup

Follow the build steps in the [README](README.md#building-from-source). `pnpm tauri dev` reloads the interface as you edit and rebuilds the Rust side when needed.

## Before you open a pull request

```sh
pnpm check
cargo test --manifest-path src-tauri/Cargo.toml
```

Both should pass. If you change what the app does, add a line to `CHANGELOG.md` under the next version.

## Layout

- `src/` is the Svelte interface. `src/lib/state.svelte.ts` holds the shared state.
- `src-tauri/src/ipc.rs` talks to Discord, `presence.rs` turns a profile into an activity, `vars.rs` fills in variables, `media.rs` reads what's playing.
- Profiles and settings are JSON files in the app data folder.

## Style

Keep comments for the reasons behind something, not for what the line does. Small commits with plain messages are easiest to review.
