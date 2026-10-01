<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api, type Profile } from "./api";
  import { app } from "./state.svelte";
  import Icon from "./Icon.svelte";
  import { flip } from "svelte/animate";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Popover from "./Popover.svelte";
  import { templates, type Template } from "./templates";

  let query = $state("");
  let menuOpen = $state(false);

  async function fromTemplate(t: Template) {
    menuOpen = false;
    await app.add(t.make());
  }

  const shown = $derived(
    app.profiles.filter((p) => {
      const q = query.trim().toLowerCase();
      return !q || p.name.toLowerCase().includes(q) || p.tags.some((t) => t.toLowerCase().includes(q));
    }),
  );

  const live = $derived(app.profiles.find((p) => p.id === app.activeId) ?? null);

  const verbs: Record<number, string> = { 0: "Playing", 2: "Listening to", 3: "Watching", 5: "Competing in" };

  /** A stable, muted colour per profile so the letter tiles are easy to tell apart. */
  function hue(name: string): number {
    let h = 0;
    for (const c of name) h = (h * 31 + c.charCodeAt(0)) % 360;
    return h;
  }

  let broken = $state<Record<string, boolean>>({});

  function subtitle(p: Profile): string {
    const name = p.nameOverride.trim();
    return name ? `${verbs[p.activityType]} ${name}` : p.details.trim() || verbs[p.activityType];
  }

  async function doImport() {
    const path = await open({ filters: [{ name: "Profiles", extensions: ["json"] }] });
    if (typeof path !== "string") return;
    try {
      const added = await api.importProfiles(path);
      app.profiles.push(...added);
      app.selectedId = added[0].id;
      app.view = "editor";
      app.notify(`Imported ${added.length} ${added.length === 1 ? "profile" : "profiles"}`);
    } catch (e) {
      app.notify(`Import failed: ${e}`);
    }
  }

  async function doExport() {
    const path = await save({ defaultPath: "profiles.json", filters: [{ name: "Profiles", extensions: ["json"] }] });
    if (!path) return;
    try {
      const n = await api.exportProfiles(path, null);
      app.notify(`Exported ${n} ${n === 1 ? "profile" : "profiles"}`);
    } catch (e) {
      app.notify(`Export failed: ${e}`);
    }
  }

  function select(id: string) {
    app.selectedId = id;
    app.view = "editor";
  }

  async function remove(e: MouseEvent, id: string, name: string) {
    e.stopPropagation();
    if (confirm(`Delete "${name}"?`)) await app.remove(id);
  }
</script>

<nav>
  <div class="top">
    <h2>Profiles</h2>
    <div class="anchor">
      <button class="icon-btn" title="New profile" aria-label="New profile" onclick={() => (menuOpen = !menuOpen)}><Icon name="plus" size={17} /></button>
      <Popover bind:open={menuOpen} width={290}>
        <div class="menu-title">Start from</div>
        <ul class="templates">
          {#each templates as t}
            <li>
              <button onclick={() => fromTemplate(t)}>
                <b>{t.label}</b>
                <span>{t.blurb}</span>
              </button>
            </li>
          {/each}
        </ul>
      </Popover>
    </div>
  </div>

  <div class="search">
    <Icon name="search" size={14} />
    <input type="text" placeholder="Search" aria-label="Search profiles" bind:value={query} />
  </div>

  <ul>
    {#each shown as p (p.id)}
      <li animate:flip={{ duration: 220, easing: cubicOut }} in:fly={{ x: -10, duration: 220, easing: cubicOut }} out:fade={{ duration: 120 }}>
        <div
          class="item"
          class:sel={app.view === "editor" && app.selectedId === p.id}
          role="button"
          tabindex="0"
          onclick={() => select(p.id)}
          onkeydown={(e) => (e.key === "Enter" || e.key === " ") && select(p.id)}
        >
          {#if p.largeImage.trim() && !broken[p.id]}
            <img class="tile" src={p.largeImage.trim()} alt="" onerror={() => (broken[p.id] = true)} />
          {:else}
            <span class="tile letter" style:--h={hue(p.name)}>{(p.name.trim()[0] ?? "?").toUpperCase()}</span>
          {/if}
          <div class="text">
            <div class="name">{p.name}</div>
            <div class="sub">{subtitle(p)}</div>
          </div>
          {#if app.activeId === p.id}<span class="live" title="Currently shown on Discord">Live</span>{/if}
          <button class="icon-btn del" title="Delete" aria-label={`Delete ${p.name}`} onclick={(e) => remove(e, p.id, p.name)}>
            <Icon name="trash" size={14} />
          </button>
        </div>
      </li>
    {:else}
      <li class="empty">{query ? "No matches." : "Nothing here yet."}</li>
    {/each}
  </ul>

  <div class="now" class:on={live}>
    {#if live}
      <span class="pulse"></span>
      <div class="nt">
        <b>Showing now</b>
        <span>{live.name}</span>
      </div>
      <button class="btn danger small" onclick={() => api.deactivate()}><Icon name="stop" size={11} /> Stop</button>
    {:else}
      <span class="idle">Nothing showing on Discord</span>
    {/if}
  </div>

  <div class="foot">
    <button class="btn ghost small" onclick={doImport}><Icon name="import" size={14} /> Import</button>
    <button class="btn ghost small" onclick={doExport} disabled={app.profiles.length === 0}><Icon name="export" size={14} /> Export</button>
    <button
      class="icon-btn gear"
      class:on={app.view === "settings"}
      title="Settings"
      aria-label="Settings"
      onclick={() => (app.view = app.view === "settings" ? "editor" : "settings")}
    >
      <Icon name="gear" size={16} />
    </button>
  </div>
</nav>

<style>
  nav {
    display: flex;
    flex-direction: column;
    width: 244px;
    flex: none;
    background: var(--bg-0);
    border-right: 1px solid var(--border);
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 14px 8px 18px;
  }
  h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .menu-title {
    margin-bottom: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .templates {
    margin: 0 -6px -6px;
    padding: 0;
    list-style: none;
  }
  .templates button {
    display: grid;
    width: 100%;
    padding: 7px 8px;
    border-radius: 6px;
    text-align: left;
    transition: background-color 120ms var(--ease-out);
  }
  .templates b {
    font-weight: 500;
    color: var(--text-strong);
  }
  .templates span {
    font-size: 12px;
    color: var(--muted);
  }
  @media (hover: hover) and (pointer: fine) {
    .templates button:hover {
      background: var(--bg-3);
    }
  }
  .search {
    position: relative;
    margin: 0 12px 8px;
    color: var(--muted);
  }
  .search :global(svg) {
    position: absolute;
    left: 11px;
    top: 10px;
    pointer-events: none;
  }
  .search input {
    height: 34px;
    padding-left: 32px;
  }
  ul {
    flex: 1;
    margin: 0;
    padding: 2px 8px;
    list-style: none;
    overflow-y: auto;
  }
  .item {
    display: flex;
    gap: 8px;
    align-items: center;
    min-height: 54px;
    padding: 7px 6px 7px 8px;
    margin-bottom: 2px;
    border-radius: var(--radius);
    cursor: pointer;
    transition:
      background-color 120ms var(--ease-out),
      transform 120ms var(--ease-out);
  }
  .item:active {
    transform: scale(0.99);
  }
  @media (hover: hover) and (pointer: fine) {
    .item:hover {
      background: var(--bg-2);
    }
  }
  .item.sel {
    background: var(--bg-3);
  }
  .tile {
    flex: none;
    width: 38px;
    height: 38px;
    border-radius: 10px;
    object-fit: cover;
  }
  .tile.letter {
    display: grid;
    place-items: center;
    font-weight: 700;
    font-size: 15px;
    color: hsl(var(--h) 70% 80%);
    background: hsl(var(--h) 32% 24%);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  }
  :global(:root[data-theme="light"]) .tile.letter {
    color: hsl(var(--h) 55% 28%);
    background: hsl(var(--h) 60% 88%);
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .name,
  .sub {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name {
    font-weight: 500;
    color: var(--text-strong);
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }
  .live {
    flex: none;
    padding: 1px 7px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--ok);
    background: color-mix(in srgb, var(--ok), transparent 86%);
  }
  .del {
    display: none;
  }
  .item:hover .del,
  .item:focus-within .del {
    display: grid;
  }
  .item:hover .live,
  .item:focus-within .live {
    display: none;
  }
  .empty {
    padding: 18px 12px;
    color: var(--muted);
  }
  .now {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 48px;
    margin: 0 8px 8px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-2);
    transition: border-color 250ms var(--ease-out);
  }
  .now.on {
    border-color: color-mix(in srgb, var(--ok), transparent 65%);
  }
  .idle {
    font-size: 12px;
    color: var(--muted);
  }
  .pulse {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    animation: pulse 2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }
  .nt {
    display: grid;
    flex: 1;
    min-width: 0;
  }
  .nt b {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .nt span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--muted);
  }
  .foot {
    display: flex;
    gap: 2px;
    align-items: center;
    padding: 8px;
    border-top: 1px solid var(--border);
  }
  .gear {
    margin-left: auto;
  }
  .gear.on {
    background: var(--bg-3);
    color: var(--text-strong);
  }
</style>
