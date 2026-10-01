<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Profile, type Resolved } from "./api";
  import { app } from "./state.svelte";
  import Icon from "./Icon.svelte";

  let { profile }: { profile: Profile } = $props();

  const verbs: Record<number, string> = { 0: "Playing", 2: "Listening to", 3: "Watching", 5: "Competing in" };

  let resolved = $state<Resolved | null>(null);
  let largeBroken = $state(false);
  let smallBroken = $state(false);
  let now = $state(Date.now());
  const mountedAt = Date.now();

  async function refresh() {
    try {
      resolved = await api.preview($state.snapshot(profile));
    } catch {
      // Only fails while the window is closing.
    }
  }

  $effect(() => {
    profile.largeImage;
    largeBroken = false;
  });
  $effect(() => {
    profile.smallImage;
    smallBroken = false;
  });

  let debounce: ReturnType<typeof setTimeout>;
  $effect(() => {
    JSON.stringify($state.snapshot(profile));
    clearTimeout(debounce);
    debounce = setTimeout(refresh, 100);
  });

  onMount(() => {
    const tick = setInterval(() => (now = Date.now()), 1000);
    const poll = setInterval(refresh, 5000);
    return () => {
      clearInterval(tick);
      clearInterval(poll);
      clearTimeout(debounce);
    };
  });

  function clock(totalSecs: number): string {
    const s = Math.max(0, Math.floor(totalSecs));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const mm = h > 0 ? String(m).padStart(2, "0") : String(m);
    return `${h > 0 ? h + ":" : ""}${mm}:${String(sec).padStart(2, "0")}`;
  }

  const elapsedText = $derived.by(() => {
    const t = profile.timestamp;
    if (t.kind === "elapsed") return `${clock((now - mountedAt) / 1000)} elapsed`;
    if (t.kind === "since" && t.value > 0) return `${clock(now / 1000 - t.value)} elapsed`;
    if (t.kind === "countdown" && t.value > 0) return `${clock(t.value - (now - mountedAt) / 1000)} left`;
    return "";
  });

  const title = $derived(resolved?.nameOverride.trim() || "Default application");
  const headline = $derived(
    profile.statusDisplay === 1 && resolved?.state
      ? resolved.state
      : profile.statusDisplay === 2 && resolved?.details
        ? resolved.details
        : `${verbs[profile.activityType]} ${title}`,
  );
  const buttons = $derived(
    (resolved?.buttons ?? []).filter((b) => b.label.trim() && /^https?:\/\//.test(b.url.trim())).slice(0, 2),
  );
  const party = $derived(
    profile.partyMax > 0 ? `${Math.min(Math.max(profile.partyCurrent, 1), profile.partyMax)} of ${profile.partyMax}` : "",
  );
  const live = $derived(app.activeId === profile.id);
  const user = $derived(app.status.user);
  const avatar = $derived(
    user?.avatar ? `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64` : null,
  );
  const me = $derived(user ? (user.globalName ?? user.username) : "You");
</script>

<aside>
  <div class="heading">
    <h3>Preview</h3>
    {#if live}<span class="live">Live</span>{/if}
  </div>

  <div class="card">
    <div class="kind">{verbs[profile.activityType]}</div>
    <div class="row">
      <div class="art">
        {#if profile.largeImage.trim() && !largeBroken}
          <img class="large" src={profile.largeImage.trim()} alt="" title={resolved?.largeText} onerror={() => (largeBroken = true)} />
        {:else}
          <div class="large empty"><Icon name="image" size={22} /></div>
        {/if}
        {#if profile.smallImage.trim() && !smallBroken}
          <img class="small" src={profile.smallImage.trim()} alt="" title={resolved?.smallText} onerror={() => (smallBroken = true)} />
        {/if}
      </div>
      <div class="text">
        <div class="title">{title}</div>
        {#if resolved?.details}<div class="line">{resolved.details}</div>{/if}
        {#if resolved?.state}<div class="line">{resolved.state}{party ? ` (${party})` : ""}</div>{/if}
        {#if elapsedText}<div class="line time">{elapsedText}</div>{/if}
      </div>
    </div>
    {#each buttons as b}
      <div class="btn-fake">{b.label}</div>
    {/each}
  </div>

  <h3 class="second">Member list</h3>
  <div class="member">
    {#if avatar}
      <img class="av" src={avatar} alt="" />
    {:else}
      <span class="av"></span>
    {/if}
    <div class="who">
      <div class="name">{me}</div>
      <div class="status">{headline}</div>
    </div>
  </div>

  <p class="note">Buttons only appear to other people, never on your own profile.</p>
</aside>

<style>
  aside {
    width: 308px;
    flex: none;
    padding: 20px;
    overflow-y: auto;
    background: var(--bg-0);
    border-left: 1px solid var(--border);
  }
  .heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  h3.second {
    margin: 22px 0 10px;
  }
  .live {
    padding: 1px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--ok);
    background: color-mix(in srgb, var(--ok), transparent 86%);
  }
  .card {
    padding: 14px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .kind {
    margin-bottom: 10px;
    font-size: 12px;
    font-weight: 700;
    color: var(--text-strong);
  }
  .row {
    display: flex;
    gap: 12px;
  }
  .art {
    position: relative;
    flex: none;
    width: 64px;
    height: 64px;
  }
  .large {
    width: 64px;
    height: 64px;
    border-radius: 8px;
    object-fit: cover;
  }
  .large.empty {
    display: grid;
    place-items: center;
    background: var(--bg-input);
    border: 1px dashed var(--border-strong);
    color: var(--muted);
  }
  .small {
    position: absolute;
    right: -6px;
    bottom: -6px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    object-fit: cover;
    border: 3px solid var(--bg-2);
    background: var(--bg-3);
  }
  .text {
    min-width: 0;
    font-size: 12.5px;
    line-height: 1.4;
  }
  .title {
    font-weight: 700;
    color: var(--text-strong);
  }
  .line {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .time {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .btn-fake {
    margin-top: 10px;
    padding: 7px;
    text-align: center;
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
    background: var(--bg-3);
    color: var(--text-strong);
  }
  .member {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 9px 10px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .av {
    flex: none;
    width: 32px;
    height: 32px;
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
  .note {
    margin: 16px 0 0;
    font-size: 12px;
    color: var(--muted);
  }
</style>
