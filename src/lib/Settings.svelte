<script lang="ts">
  import { onMount } from "svelte";
  import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { app } from "./state.svelte";
  import Segmented from "./Segmented.svelte";
  import { cue } from "./sound";

  let autostart = $state(false);

  onMount(async () => {
    try {
      autostart = await isEnabled();
    } catch {
      autostart = false;
    }
  });

  async function setAutostart(on: boolean) {
    try {
      if (on) await enable();
      else await disable();
      autostart = on;
    } catch (e) {
      autostart = !on;
      app.notify(`Could not change autostart: ${e}`);
    }
  }

  const themes = [
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
    { value: "system", label: "System" },
  ] as const;

  const sounds = [
    { value: "off", label: "Off" },
    { value: "soft", label: "Soft" },
    { value: "normal", label: "Normal" },
  ] as const;

  const accents = ["#5865f2", "#3b82f6", "#14b8a6", "#22c55e", "#f59e0b", "#ef4444", "#ec4899", "#a855f7"];
</script>

<div class="page">
  <div class="inner enter">
    <h2>Settings</h2>

    <section>
      <h3>Startup</h3>
      <label class="row">
        <span>
          <b>Launch at sign-in</b>
          <small>Start with your computer.</small>
        </span>
        <input class="switch" type="checkbox" checked={autostart} onchange={(e) => {
          cue("tick");
          setAutostart(e.currentTarget.checked);
        }} />
      </label>
      <label class="row">
        <span>
          <b>Start hidden</b>
          <small>Open in the tray without showing the window.</small>
        </span>
        <input class="switch" type="checkbox" checked={app.settings.startMinimized} onchange={(e) => {
          cue("tick");
          app.updateSettings({ startMinimized: e.currentTarget.checked });
        }} />
      </label>
      <label class="row">
        <span>
          <b>Resume last profile</b>
          <small>Show your last used profile as soon as the app starts.</small>
        </span>
        <input class="switch" type="checkbox" checked={app.settings.resumeLast} onchange={(e) => {
          cue("tick");
          app.updateSettings({ resumeLast: e.currentTarget.checked });
        }} />
      </label>
      <label class="row">
        <span>
          <b>Close to tray</b>
          <small>Closing the window keeps your presence running.</small>
        </span>
        <input class="switch" type="checkbox" checked={app.settings.closeToTray} onchange={(e) => {
          cue("tick");
          app.updateSettings({ closeToTray: e.currentTarget.checked });
        }} />
      </label>
    </section>

    <section>
      <h3>Appearance</h3>
      <div class="row">
        <span><b>Theme</b></span>
        <Segmented options={[...themes]} bind:value={() => app.settings.theme, (v) => app.updateSettings({ theme: v })} label="Theme" />
      </div>
      <div class="row">
        <span><b>Accent</b></span>
        <div class="swatches">
          {#each accents as c}
            <button
              class="swatch"
              class:on={app.settings.accent.toLowerCase() === c}
              style:background={c}
              aria-label={`Accent ${c}`}
              onclick={() => app.updateSettings({ accent: c })}
            ></button>
          {/each}
          <input type="color" value={app.settings.accent} aria-label="Custom accent" onchange={(e) => app.updateSettings({ accent: e.currentTarget.value })} />
        </div>
      </div>
    </section>

    <section>
      <h3>Now playing</h3>
      <label class="row">
        <span>
          <b>Detect what's playing</b>
          <small>Lets profiles use {"{title}"}, {"{artist}"} and friends. Read locally; only the cover art lookup goes online, and only if a profile uses it.</small>
        </span>
        <input
          class="switch"
          type="checkbox"
          checked={app.settings.media}
          onchange={(e) => {
            cue("tick");
            app.updateSettings({ media: e.currentTarget.checked });
          }}
        />
      </label>
    </section>

    <section>
      <h3>Sound</h3>
      <div class="row">
        <span>
          <b>Sound effects</b>
          <small>Quiet cues when a profile starts, stops or something goes wrong.</small>
        </span>
        <Segmented
          options={[...sounds]}
          quiet
          bind:value={() => app.settings.sound, (v) => {
            app.updateSettings({ sound: v });
            cue("add");
          }}
          label="Sound effects"
        />
      </div>
    </section>

    <section>
      <h3>Discord</h3>
      <div class="field">
        <span class="label">Default application ID</span>
        <input type="text" aria-label="Default application ID" value={app.settings.defaultClientId} maxlength="32" onchange={(e) => app.updateSettings({ defaultClientId: e.currentTarget.value.trim() })} />
        <p class="hint">Used when a profile doesn't set its own. The name Discord shows comes from each profile's Name field.</p>
      </div>
    </section>

    <section>
      <h3>About</h3>
      <p class="hint">DiscordRPC {app.version}. Not affiliated with or endorsed by Discord Inc.</p>
    </section>
  </div>
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
  }
  .inner {
    max-width: 600px;
    padding: 24px 32px 48px;
  }
  h2 {
    margin: 0 0 22px;
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-strong);
  }
  section {
    display: grid;
    gap: 2px;
    margin-bottom: 30px;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .row {
    display: flex;
    gap: 16px;
    align-items: center;
    justify-content: space-between;
    min-height: 52px;
    padding: 8px 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    cursor: default;
  }
  section > .row:first-of-type,
  section > h3 + .row {
    border-top-left-radius: 10px;
    border-top-right-radius: 10px;
  }
  section > .row:last-of-type {
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 10px;
  }
  .row span {
    display: grid;
  }
  .row b {
    font-weight: 500;
    color: var(--text-strong);
  }
  .row small {
    font-size: 12px;
    color: var(--muted);
  }
  .swatches {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid var(--bg-2);
    box-shadow: 0 0 0 1px var(--border-strong);
    transition: transform 140ms var(--ease-out);
  }
  .swatch:active {
    transform: scale(0.9);
  }
  .swatch.on {
    box-shadow: 0 0 0 2px var(--text-strong);
  }
  .swatches input[type="color"] {
    width: 30px;
    height: 24px;
    margin-left: 4px;
  }
</style>
