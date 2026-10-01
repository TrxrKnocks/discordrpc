<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { api, type Profile, type Resolved } from "./api";
  import { app } from "./state.svelte";
  import { cue } from "./sound";
  import Stage from "./Stage.svelte";
  import Segmented from "./Segmented.svelte";
  import Icon from "./Icon.svelte";

  let { profile }: { profile: Profile } = $props();

  // svelte-ignore state_referenced_locally
  let draft = $state<Profile>(structuredClone($state.snapshot(profile)));
  let resolved = $state<Resolved | null>(null);

  let timer: ReturnType<typeof setTimeout> | undefined;
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
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

  async function refreshPreview() {
    try {
      resolved = await api.preview($state.snapshot(draft));
    } catch {
      // Only fails while the window is closing.
    }
  }

  $effect(() => {
    const snap = JSON.stringify($state.snapshot(draft));
    clearTimeout(previewTimer);
    previewTimer = setTimeout(refreshPreview, 100);
    if (first) {
      first = false;
      return;
    }
    pending = snap;
    clearTimeout(timer);
    timer = setTimeout(flush, 350);
  });

  onMount(() => {
    app.refreshValues();
    const poll = setInterval(() => {
      refreshPreview();
      app.refreshValues();
    }, 5000);
    return () => clearInterval(poll);
  });
  onDestroy(() => {
    clearTimeout(previewTimer);
    flush();
  });

  const active = $derived(app.activeId === draft.id);

  const verbs: Record<number, string> = { 0: "Playing", 2: "Listening to", 3: "Watching", 5: "Competing in" };

  const headlines = [
    { value: 0, label: "Name" },
    { value: 1, label: "State" },
    { value: 2, label: "Details" },
  ] as const;

  const headline = $derived(
    draft.statusDisplay === 1 && resolved?.state
      ? resolved.state
      : draft.statusDisplay === 2 && resolved?.details
        ? resolved.details
        : `${verbs[draft.activityType]} ${resolved?.nameOverride.trim() || "the default application"}`,
  );

  const user = $derived(app.status.user);
  const avatar = $derived(
    user?.avatar ? `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64` : null,
  );
  const who = $derived(user ? (user.globalName ?? user.username) : null);

  const appId = $derived((draft.clientId.trim() || app.settings.defaultClientId || "").trim());
  const appIdShort = $derived(appId.length > 10 ? `${appId.slice(0, 4)}…${appId.slice(-4)}` : appId || "none");

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

<div class="page">
  <div class="inner enter">
    <header>
      <input class="title" type="text" bind:value={draft.name} maxlength="60" aria-label="Profile name" />
      <button class="btn ghost small" title="Duplicate this profile" onclick={() => app.create(draft)}>
        <Icon name="copy" size={14} /> Duplicate
      </button>
      <button class="btn primary go" class:danger={active} onclick={toggle}>
        {#key active}
          <span class="lbl" in:fly={{ y: 8, duration: 200, easing: cubicOut }}>
            <Icon name={active ? "stop" : "play"} size={13} />
            {active ? "Stop" : "Start"}
          </span>
        {/key}
      </button>
    </header>

    {#if active && app.status.error}
      <p class="note bad">Discord rejected this: {app.status.error}</p>
    {:else if active}
      <p class="note live"><span class="dot"></span> Showing on your Discord profile now. Changes apply as you type.</p>
    {:else if !app.status.connected}
      <p class="note">Waiting for Discord. Open the desktop app and this connects on its own.</p>
    {:else}
      <p class="note">Click any text on the card to edit it. Press Start when you're ready.</p>
    {/if}

    <div class="cols">
      <div class="left">
        <Stage bind:draft {resolved} live={active} />

        <details>
          <summary>More options</summary>
          <div class="more">
            <div class="field">
              <span class="label">Application ID</span>
              <input type="text" aria-label="Application ID" maxlength="32" placeholder={app.settings.defaultClientId || "Uses the default"} bind:value={draft.clientId} />
              <p class="hint">Leave empty to use the default from Settings.</p>
            </div>
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
          </div>
        </details>
      </div>

      <aside class="right">
        <section class="panel">
          <h3>Connection</h3>
          <dl>
            <dt>Discord</dt>
            <dd class:ok={app.status.connected}>{app.status.connected ? (who ? `Connected as ${who}` : "Connected") : "Not detected"}</dd>
            <dt>Presence</dt>
            <dd class:ok={active}>{active ? "Showing" : "Off"}</dd>
            <dt>Application</dt>
            <dd class="mono" title={appId}>{appIdShort}</dd>
          </dl>
        </section>

        <section class="panel">
          <h3>Member list</h3>
          <div class="member">
            {#if avatar}<img src={avatar} alt="" />{:else}<span class="av"></span>{/if}
            <div class="who">
              <div class="name">{who ?? "You"}</div>
              <div class="status">{headline}</div>
            </div>
          </div>
          <div class="seg-wrap">
            <span class="label">Show as headline</span>
            <Segmented options={[...headlines]} bind:value={draft.statusDisplay} label="Status headline" fill />
          </div>
          <p class="fine">Buttons only appear to other people, never on your own profile.</p>
        </section>

        <section class="panel">
          <h3>Variables</h3>
          <p class="fine top">Click one to add it to the line you're editing.</p>
          <ul class="vars">
            {#each app.variables as [name, desc]}
              <li>
                <button title={desc} onmousedown={(e) => e.preventDefault()} onclick={() => { cue("tick"); app.insertVariable(name); }}>
                  <code>{`{${name}}`}</code>
                  <span>{app.values[name] ?? ""}</span>
                </button>
              </li>
            {/each}
          </ul>
        </section>
      </aside>
    </div>
  </div>
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
  }
  .inner {
    max-width: 1020px;
    margin: 0 auto;
    padding: 20px 28px 48px;
  }
  header {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 6px;
  }
  .title {
    flex: 1;
    height: 40px;
    padding: 0 10px;
    margin-left: -10px;
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.015em;
    color: var(--text-strong);
    background: none;
    border-color: transparent;
  }
  .go {
    position: relative;
    min-width: 96px;
    overflow: hidden;
  }
  .lbl {
    display: inline-flex;
    gap: 7px;
    align-items: center;
  }
  .note {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 0 0 18px;
    color: var(--muted);
  }
  .note.live {
    color: var(--ok);
  }
  .note.bad {
    color: var(--warn);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    animation: pulse 2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 292px;
    gap: 22px;
    align-items: start;
  }
  .left {
    min-width: 0;
  }
  .right {
    display: grid;
    gap: 14px;
  }
  .right > :global(*) {
    animation: enter 260ms var(--ease-out) both;
  }
  .right > :global(:nth-child(2)) {
    animation-delay: 45ms;
  }
  .right > :global(:nth-child(3)) {
    animation-delay: 90ms;
  }

  .panel {
    padding: 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--hi);
  }
  h3 {
    margin: 0 0 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 14px;
    margin: 0;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
    overflow: hidden;
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-strong);
  }
  dd.ok {
    color: var(--ok);
  }
  .mono {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 12px;
  }

  .member {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 9px 10px;
    background: var(--bg-1);
    border: 1px solid var(--border);
    border-radius: 9px;
  }
  .member img,
  .av {
    flex: none;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: var(--bg-3);
  }
  .who {
    min-width: 0;
  }
  .name {
    font-weight: 600;
    color: var(--text-strong);
  }
  .status {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--muted);
  }
  .seg-wrap {
    display: grid;
    gap: 6px;
    margin-top: 12px;
  }
  .fine {
    margin: 10px 0 0;
    font-size: 12px;
    color: var(--muted);
  }
  .fine.top {
    margin: -4px 0 8px;
  }

  .vars {
    margin: 0 -6px -6px;
    padding: 0;
    list-style: none;
  }
  .vars button {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 30px;
    padding: 0 8px;
    border-radius: 6px;
    transition:
      background-color 120ms var(--ease-out),
      transform 120ms var(--ease-out);
  }
  .vars button:active {
    transform: scale(0.98);
  }
  @media (hover: hover) and (pointer: fine) {
    .vars button:hover {
      background: var(--bg-3);
    }
    .vars button:hover code {
      color: var(--accent);
    }
  }
  .vars code {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 12px;
    color: var(--text-strong);
    transition: color 120ms var(--ease-out);
  }
  .vars span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  details {
    margin-top: 20px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
  summary {
    cursor: pointer;
    color: var(--muted);
    font-weight: 500;
  }
  @media (hover: hover) and (pointer: fine) {
    summary:hover {
      color: var(--text-strong);
    }
  }
  .more {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    margin-top: 14px;
  }

  @media (max-width: 860px) {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
