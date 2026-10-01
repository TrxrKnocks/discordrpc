<script lang="ts">
  import type { Profile, Resolved } from "./api";
  import InlineField from "./InlineField.svelte";
  import Popover from "./Popover.svelte";
  import Segmented from "./Segmented.svelte";
  import Select from "./Select.svelte";
  import Icon from "./Icon.svelte";
  import { app } from "./state.svelte";

  let { draft = $bindable(), resolved, live = false }: { draft: Profile; resolved: Resolved | null; live?: boolean } = $props();

  const user = $derived(app.status.user);
  const avatar = $derived(
    user?.avatar ? `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64` : null,
  );

  let imagesOpen = $state(false);
  let timerOpen = $state(false);
  let partyOpen = $state(false);
  let largeBroken = $state(false);
  let smallBroken = $state(false);

  $effect(() => {
    draft.largeImage;
    largeBroken = false;
  });
  $effect(() => {
    draft.smallImage;
    smallBroken = false;
  });

  const types = [
    { value: 0, label: "Playing" },
    { value: 2, label: "Listening to" },
    { value: 3, label: "Watching" },
    { value: 5, label: "Competing in" },
  ] as const;

  const timers = [
    { value: "none", label: "None" },
    { value: "elapsed", label: "Elapsed" },
    { value: "since", label: "Since" },
    { value: "countdown", label: "Countdown" },
  ] as const;

  const large = $derived(draft.largeImage.trim());
  const small = $derived(draft.smallImage.trim());

  const timerLabel = $derived(
    {
      none: "No timer",
      elapsed: "Elapsed time",
      since: "Elapsed since a time",
      countdown: `Countdown, ${Math.round(draft.timestamp.value / 60) || 0} min`,
    }[draft.timestamp.kind],
  );

  const partyLabel = $derived(
    draft.partyMax > 0 ? `Party ${Math.min(Math.max(draft.partyCurrent, 1), draft.partyMax)} of ${draft.partyMax}` : "No party",
  );

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

  function addButton() {
    if (draft.buttons.length < 2) draft.buttons.push({ label: "", url: "" });
  }

  function removeButton(i: number) {
    draft.buttons.splice(i, 1);
  }

  /** Shows what a templated line resolves to right now. */
  function resolvedLine(raw: string, out: string | undefined): string {
    return raw.includes("{") && out !== undefined && out !== raw.trim() ? out : "";
  }
</script>

<div class="stage">
  <div class="wrap">
    <div class="me">
      {#if avatar}<img src={avatar} alt="" />{:else}<span class="av"></span>{/if}
      <span class="who">{user ? (user.globalName ?? user.username) : "You"}</span>
      <span class="online"></span>
    </div>

  <div class="card" class:live>
    <div class="kind">
      <Select variant="inline" options={types} bind:value={draft.activityType} label="Activity type" />
      {#if live}<span class="livetag"><i></i>Live</span>{/if}
    </div>

    <div class="row">
      <div class="art-wrap">
        <button class="art" class:empty={!large || largeBroken} title="Set images" aria-label="Set images" onclick={() => (imagesOpen = !imagesOpen)}>
          {#if large && !largeBroken}
            <img src={large} alt="" onerror={() => (largeBroken = true)} />
          {:else}
            <Icon name="image" size={26} />
            <span>Add image</span>
          {/if}
        </button>
        {#if small && !smallBroken}
          <img class="badge" src={small} alt="" onerror={() => (smallBroken = true)} />
        {/if}

        <Popover bind:open={imagesOpen} width={340}>
          <div class="pop-title">Large image</div>
          <div class="stack">
            <input type="text" aria-label="Large image URL" placeholder="Image link (https://...)" maxlength="512" bind:value={draft.largeImage} />
            <input type="text" aria-label="Large image hover text" placeholder="Hover text (optional)" maxlength="128" bind:value={draft.largeText} />
          </div>
          <div class="pop-title second">Small badge</div>
          <div class="stack">
            <input type="text" aria-label="Small image URL" placeholder="Image link (https://...)" maxlength="512" bind:value={draft.smallImage} />
            <input type="text" aria-label="Small image hover text" placeholder="Hover text (optional)" maxlength="128" bind:value={draft.smallText} />
          </div>
          <p class="hint">Use a direct link to a PNG, JPG or GIF. Discord loads it for you, so it has to be public.</p>
        </Popover>
      </div>

      <div class="text">
        <InlineField kind="title" label="Activity name" placeholder="Activity name" max={64} bind:value={draft.nameOverride} />
        {#if resolvedLine(draft.nameOverride, resolved?.nameOverride)}<div class="resolved">{resolved?.nameOverride}</div>{/if}

        <InlineField label="Details" placeholder="Details" bind:value={draft.details} />
        {#if resolvedLine(draft.details, resolved?.details)}<div class="resolved">{resolved?.details}</div>{/if}

        <InlineField label="State" placeholder="State" bind:value={draft.state} />
        {#if resolvedLine(draft.state, resolved?.state)}<div class="resolved">{resolved?.state}</div>{/if}

        <div class="chips">
          <div class="anchor">
            <button class="chip" class:on={draft.timestamp.kind !== "none"} onclick={() => (timerOpen = !timerOpen)}>{timerLabel}</button>
            <Popover bind:open={timerOpen} width={320}>
              <Segmented options={[...timers]} bind:value={draft.timestamp.kind} label="Timer" fill />
              {#if draft.timestamp.kind === "since"}
                <div class="stack pad">
                  <span class="label">Started at</span>
                  <input type="datetime-local" aria-label="Started at" value={toLocalInput(draft.timestamp.value)} onchange={(e) => (draft.timestamp.value = fromLocalInput(e.currentTarget.value))} />
                </div>
              {:else if draft.timestamp.kind === "countdown"}
                <div class="stack pad">
                  <span class="label">Minutes until it ends</span>
                  <input type="number" min="1" aria-label="Minutes" value={Math.round(draft.timestamp.value / 60) || ""} onchange={(e) => (draft.timestamp.value = Math.max(0, Math.round(Number(e.currentTarget.value) * 60)))} />
                </div>
              {:else}
                <p class="hint pad">{draft.timestamp.kind === "elapsed" ? "Counts up from the moment you press Start." : "Hides the timer."}</p>
              {/if}
            </Popover>
          </div>

          <div class="anchor">
            <button class="chip" class:on={draft.partyMax > 0} onclick={() => (partyOpen = !partyOpen)}>{partyLabel}</button>
            <Popover bind:open={partyOpen} width={260}>
              <div class="two">
                <label class="field">
                  <span class="label">Current</span>
                  <input type="number" min="0" bind:value={draft.partyCurrent} />
                </label>
                <label class="field">
                  <span class="label">Maximum</span>
                  <input type="number" min="0" bind:value={draft.partyMax} />
                </label>
              </div>
              <p class="hint pad">Set the maximum to 0 to hide the party.</p>
            </Popover>
          </div>
        </div>
      </div>
    </div>

    {#each draft.buttons as b, i}
      <div class="btn-row">
        <input type="text" aria-label={`Button ${i + 1} label`} placeholder="Button label" maxlength="32" bind:value={b.label} />
        <input type="url" aria-label={`Button ${i + 1} link`} placeholder="https://..." bind:value={b.url} />
        <button class="icon-btn" title="Remove button" aria-label="Remove button" onclick={() => removeButton(i)}><Icon name="x" size={14} /></button>
      </div>
    {/each}
    {#if draft.buttons.length < 2}
      <button class="add" onclick={addButton}><Icon name="plus" size={14} /> Add a button</button>
    {/if}
  </div>
  </div>
</div>

<style>
  .stage {
    display: grid;
    place-items: center;
    padding: 36px 24px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background-color: var(--bg-0);
    background-image: radial-gradient(color-mix(in srgb, var(--border-strong), transparent 40%) 1px, transparent 1px);
    background-size: 18px 18px;
  }
  .card {
    padding: 18px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow:
      var(--hi),
      0 10px 30px rgba(0, 0, 0, 0.22);
    transition:
      border-color 250ms var(--ease-out),
      box-shadow 250ms var(--ease-out);
  }
  .card.live {
    border-color: color-mix(in srgb, var(--accent), transparent 35%);
    box-shadow:
      var(--hi),
      0 0 0 4px color-mix(in srgb, var(--accent), transparent 88%),
      0 10px 30px rgba(0, 0, 0, 0.22);
  }
  .wrap {
    width: min(100%, 470px);
  }
  .me {
    display: flex;
    gap: 9px;
    align-items: center;
    margin: 0 2px 12px;
  }
  .me img,
  .av {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--bg-3);
  }
  .who {
    font-weight: 600;
    color: var(--text-strong);
  }
  .online {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
  }
  .kind {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  .livetag {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    padding: 2px 9px;
    border-radius: 11px;
    font-size: 11px;
    font-weight: 600;
    color: var(--ok);
    background: color-mix(in srgb, var(--ok), transparent 86%);
  }
  .livetag i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ok);
    animation: pulse 2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }
  .row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }
  .art-wrap {
    position: relative;
    flex: none;
    width: 100px;
    height: 100px;
  }
  .art {
    display: grid;
    gap: 4px;
    place-content: center;
    justify-items: center;
    width: 100px;
    height: 100px;
    padding: 0;
    overflow: hidden;
    border-radius: 12px;
    transition: transform 140ms var(--ease-out);
  }
  .art:active {
    transform: scale(0.97);
  }
  .art.empty {
    background: var(--bg-input);
    border: 1px dashed var(--border-strong);
    color: var(--muted);
    font-size: 12px;
  }
  @media (hover: hover) and (pointer: fine) {
    .art.empty:hover {
      border-color: var(--accent);
      color: var(--text-strong);
    }
  }
  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .badge {
    position: absolute;
    right: -8px;
    bottom: -8px;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    object-fit: cover;
    border: 3px solid var(--bg-2);
    pointer-events: none;
  }
  .text {
    display: grid;
    flex: 1;
    gap: 3px;
    min-width: 0;
  }
  .resolved {
    margin: -1px 0 2px;
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .resolved::before {
    content: "shows as  ";
    opacity: 0.7;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  .anchor {
    position: relative;
  }
  .chip {
    height: 26px;
    padding: 0 10px;
    font-size: 12px;
    color: var(--muted);
    border: 1px solid var(--border);
    border-radius: 13px;
    transition:
      transform 140ms var(--ease-out),
      background-color 150ms var(--ease-out),
      color 150ms var(--ease-out);
  }
  .chip:active {
    transform: scale(0.96);
  }
  .chip.on {
    color: var(--text);
    background: var(--bg-3);
  }
  @media (hover: hover) and (pointer: fine) {
    .chip:hover {
      color: var(--text-strong);
      border-color: var(--border-strong);
    }
  }
  .btn-row {
    display: grid;
    grid-template-columns: 1fr 1.4fr 28px;
    gap: 6px;
    margin-top: 12px;
  }
  .add {
    display: flex;
    gap: 6px;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 34px;
    margin-top: 12px;
    color: var(--muted);
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius);
    transition:
      transform 140ms var(--ease-out),
      color 150ms var(--ease-out),
      border-color 150ms var(--ease-out);
  }
  .add:active {
    transform: scale(0.99);
  }
  @media (hover: hover) and (pointer: fine) {
    .add:hover {
      color: var(--text-strong);
      border-color: var(--accent);
    }
  }
  .pop-title {
    margin-bottom: 8px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .pop-title.second {
    margin-top: 16px;
  }
  .stack {
    display: grid;
    gap: 8px;
  }
  .stack.pad,
  .hint.pad {
    margin-top: 12px;
  }
  .two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .hint {
    margin-top: 12px;
  }
</style>
