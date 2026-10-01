<script lang="ts">
  import { onDestroy } from "svelte";
  import { api, type Profile } from "./api";
  import { app } from "./state.svelte";
  import TextField from "./TextField.svelte";
  import ImageField from "./ImageField.svelte";
  import Segmented from "./Segmented.svelte";
  import Preview from "./Preview.svelte";
  import Icon from "./Icon.svelte";

  let { profile }: { profile: Profile } = $props();

  // svelte-ignore state_referenced_locally
  let draft = $state<Profile>(structuredClone($state.snapshot(profile)));

  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: string | null = null;
  let first = true;

  async function flush() {
    clearTimeout(timer);
    if (pending === null) return;
    const body = pending;
    pending = null;
    try {
      app.replace(await api.saveProfile(JSON.parse(body)));
    } catch (e) {
      app.notify(`Could not save: ${e}`);
    }
  }

  $effect(() => {
    const snap = JSON.stringify($state.snapshot(draft));
    if (first) {
      first = false;
      return;
    }
    pending = snap;
    clearTimeout(timer);
    timer = setTimeout(flush, 350);
  });

  onDestroy(flush);

  const active = $derived(app.activeId === draft.id);

  const tabs = [
    { value: "presence", label: "Presence" },
    { value: "images", label: "Images" },
    { value: "timer", label: "Timer & party" },
    { value: "buttons", label: "Buttons" },
    { value: "advanced", label: "Advanced" },
  ] as const;

  const types = [
    { value: 0, label: "Playing" },
    { value: 2, label: "Listening" },
    { value: 3, label: "Watching" },
    { value: 5, label: "Competing" },
  ] as const;

  const headlines = [
    { value: 0, label: "Activity name" },
    { value: 1, label: "State line" },
    { value: 2, label: "Details line" },
  ] as const;

  const timers = [
    { value: "none", label: "None" },
    { value: "elapsed", label: "Elapsed" },
    { value: "since", label: "Since a time" },
    { value: "countdown", label: "Countdown" },
  ] as const;

  function toLocalInput(secs: number): string {
    if (!secs) return "";
    const d = new Date(secs * 1000);
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  function fromLocalInput(v: string): number {
    const ms = new Date(v).getTime();
    return Number.isNaN(ms) ? 0 : Math.floor(ms / 1000);
  }

  function setButton(i: number, key: "label" | "url", value: string) {
    while (draft.buttons.length <= i) draft.buttons.push({ label: "", url: "" });
    draft.buttons[i][key] = value;
    draft.buttons = draft.buttons.filter((b, idx) => b.label || b.url || idx < i);
  }

  async function toggle() {
    await flush();
    try {
      await app.toggle(draft.id);
    } catch (e) {
      app.notify(`${e}`);
    }
  }

  const tagText = $derived(draft.tags.join(", "));
</script>

<div class="wrap">
  <div class="main">
    <header>
      <input class="title" type="text" bind:value={draft.name} maxlength="60" aria-label="Profile name" />
      <button class="btn ghost small" title="Duplicate this profile" onclick={() => app.create(draft)}>
        <Icon name="copy" size={14} /> Duplicate
      </button>
      <button class="btn primary go" class:danger={active} onclick={toggle}>
        <Icon name={active ? "stop" : "play"} size={13} />
        {active ? "Stop" : "Start"}
      </button>
    </header>

    {#if active && app.status.error}
      <p class="alert">Discord rejected this: {app.status.error}</p>
    {:else if !app.status.connected}
      <p class="alert">Waiting for Discord. Open the desktop app and this connects on its own.</p>
    {/if}

    <div class="tabs">
      <Segmented options={[...tabs]} bind:value={app.tab} label="Section" />
    </div>

    <div class="scroll">
      {#key app.tab}
        <div class="panel">
          {#if app.tab === "presence"}
            <div class="field span">
              <span class="label">Activity type</span>
              <Segmented options={[...types]} bind:value={draft.activityType} label="Activity type" fill />
            </div>
            <div class="grid">
              <TextField label="Name" bind:value={draft.nameOverride} max={64} placeholder="Shown after the type" hint="Empty uses the application's own name." span />
              <TextField label="Details" bind:value={draft.details} placeholder="First line" />
              <TextField label="State" bind:value={draft.state} placeholder="Second line" />
            </div>
            <div class="field">
              <span class="label">Status headline</span>
              <Segmented options={[...headlines]} bind:value={draft.statusDisplay} label="Status headline" fill />
              <p class="hint">What the member list shows under your name.</p>
            </div>
          {:else if app.tab === "images"}
            <ImageField title="Large image" bind:url={draft.largeImage} bind:tooltip={draft.largeText} />
            <ImageField title="Small image" bind:url={draft.smallImage} bind:tooltip={draft.smallText} round />
            <p class="hint">Paste a direct link to a PNG, JPG or GIF. Discord proxies it, so it needs to be reachable over HTTPS.</p>
          {:else if app.tab === "timer"}
            <div class="field">
              <span class="label">Timer</span>
              <Segmented options={[...timers]} bind:value={draft.timestamp.kind} label="Timer mode" fill />
            </div>
            {#if draft.timestamp.kind === "since"}
              <div class="field">
                <span class="label">Started at</span>
                <input type="datetime-local" aria-label="Started at" value={toLocalInput(draft.timestamp.value)} onchange={(e) => (draft.timestamp.value = fromLocalInput(e.currentTarget.value))} />
              </div>
            {:else if draft.timestamp.kind === "countdown"}
              <div class="field">
                <span class="label">Minutes</span>
                <input type="number" min="1" aria-label="Minutes" value={Math.round(draft.timestamp.value / 60) || ""} onchange={(e) => (draft.timestamp.value = Math.max(0, Math.round(Number(e.currentTarget.value) * 60)))} />
              </div>
            {/if}
            <div class="grid party">
              <div class="field">
                <span class="label">Party size</span>
                <input type="number" min="0" aria-label="Party size" bind:value={draft.partyCurrent} />
              </div>
              <div class="field">
                <span class="label">Party maximum</span>
                <input type="number" min="0" aria-label="Party maximum" bind:value={draft.partyMax} />
              </div>
              <p class="hint span">Set the maximum to 0 to hide the party counter.</p>
            </div>
          {:else if app.tab === "buttons"}
            {#each [0, 1] as i}
              <div class="box">
                <div class="box-title">Button {i + 1}</div>
                <div class="grid">
                  <TextField label="Label" bind:value={() => draft.buttons[i]?.label ?? "", (v) => setButton(i, "label", v)} max={32} />
                  <div class="field">
                    <span class="label">Link</span>
                    <input type="url" aria-label={`Button ${i + 1} link`} placeholder="https://" value={draft.buttons[i]?.url ?? ""} oninput={(e) => setButton(i, "url", e.currentTarget.value)} />
                  </div>
                </div>
              </div>
            {/each}
            <p class="hint">Needs a label and a full http(s) link to show up.</p>
          {:else}
            <TextField
              label="Application ID"
              bind:value={draft.clientId}
              max={32}
              picker={false}
              placeholder={app.settings.defaultClientId || "Uses the default"}
              hint="Empty uses the default from Settings. Changing it reconnects to Discord."
            />
            <div class="field">
              <span class="label">Tags</span>
              <input
                type="text"
                aria-label="Tags"
                placeholder="work, gaming"
                value={tagText}
                onchange={(e) =>
                  (draft.tags = e.currentTarget.value
                    .split(",")
                    .map((t) => t.trim())
                    .filter(Boolean))}
              />
              <p class="hint">Comma separated. The sidebar search matches them.</p>
            </div>
          {/if}
        </div>
      {/key}
    </div>
  </div>

  <Preview profile={draft} />
</div>

<style>
  .wrap {
    display: flex;
    height: 100%;
    min-width: 0;
  }
  .main {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    padding: 18px 24px 0;
  }
  header {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 14px;
  }
  .title {
    flex: 1;
    height: 38px;
    padding: 0 10px;
    margin-left: -10px;
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-strong);
    background: none;
    border-color: transparent;
  }
  .go {
    min-width: 92px;
  }
  .alert {
    margin: 0 0 14px;
    padding: 9px 12px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--warn), transparent 90%);
    border: 1px solid color-mix(in srgb, var(--warn), transparent 70%);
  }
  .tabs {
    margin-bottom: 18px;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    margin-right: -24px;
    padding-right: 24px;
    overflow-y: auto;
  }
  .panel {
    display: grid;
    gap: 18px;
    padding-bottom: 28px;
    animation: enter 160ms var(--ease-out);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
  .span {
    grid-column: 1 / -1;
  }
  .box {
    padding: 16px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .box-title {
    margin-bottom: 12px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .party {
    max-width: 440px;
  }
</style>
