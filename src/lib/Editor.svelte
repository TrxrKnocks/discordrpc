<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { api, type Profile, type Resolved } from "./api";
  import { app } from "./state.svelte";
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
    const poll = setInterval(refreshPreview, 5000);
    return () => clearInterval(poll);
  });
  onDestroy(() => {
    clearTimeout(previewTimer);
    flush();
  });

  const active = $derived(app.activeId === draft.id);

  const verbs: Record<number, string> = { 0: "Playing", 2: "Listening to", 3: "Watching", 5: "Competing in" };

  const headlines = [
    { value: 0, label: "Activity name" },
    { value: 1, label: "State line" },
    { value: 2, label: "Details line" },
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
  <div class="inner">
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
      <p class="note bad">Discord rejected this: {app.status.error}</p>
    {:else if active}
      <p class="note live"><span class="dot"></span> Showing on your Discord profile now. Changes apply as you type.</p>
    {:else if !app.status.connected}
      <p class="note">Waiting for Discord. Open the desktop app and this connects on its own.</p>
    {:else}
      <p class="note">Click any text on the card to edit it. Press Start when you're ready.</p>
    {/if}

    <Stage bind:draft {resolved} />

    <section class="vars">
      <div class="head">
        <h3>Variables</h3>
        <span>Click one to add it to the line you're editing. They update live.</span>
      </div>
      <div class="chips">
        {#each app.variables as [name, desc]}
          <button class="var" title={desc} onmousedown={(e) => e.preventDefault()} onclick={() => app.insertVariable(name)}>{`{${name}}`}</button>
        {/each}
      </div>
    </section>

    <section class="member">
      <div class="head">
        <h3>In the member list</h3>
      </div>
      <div class="member-card">
        {#if avatar}<img src={avatar} alt="" />{:else}<span class="av"></span>{/if}
        <div class="who">
          <div class="name">{user ? (user.globalName ?? user.username) : "You"}</div>
          <div class="status">{headline}</div>
        </div>
        <Segmented options={[...headlines]} bind:value={draft.statusDisplay} label="Status headline" />
      </div>
      <p class="fine">Buttons only appear to other people, never on your own profile.</p>
    </section>

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
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
  }
  .inner {
    max-width: 640px;
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
    letter-spacing: -0.01em;
    color: var(--text-strong);
    background: none;
    border-color: transparent;
  }
  .go {
    min-width: 96px;
  }
  .note {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 0 0 16px;
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
  section {
    margin-top: 26px;
  }
  .head {
    display: flex;
    gap: 10px;
    align-items: baseline;
    margin-bottom: 10px;
  }
  h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .head span {
    font-size: 12px;
    color: var(--muted);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .var {
    height: 26px;
    padding: 0 10px;
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 12px;
    color: var(--text);
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    transition:
      transform 120ms var(--ease-out),
      border-color 150ms var(--ease-out),
      color 150ms var(--ease-out);
  }
  .var:active {
    transform: scale(0.95);
  }
  @media (hover: hover) and (pointer: fine) {
    .var:hover {
      color: var(--text-strong);
      border-color: var(--accent);
    }
  }
  .member-card {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 10px 12px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .member-card img,
  .av {
    flex: none;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--bg-3);
  }
  .who {
    flex: 1;
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
  .fine {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--muted);
  }
  details {
    margin-top: 26px;
    border-top: 1px solid var(--border);
    padding-top: 14px;
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
</style>
