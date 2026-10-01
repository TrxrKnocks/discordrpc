<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { quartOut } from "svelte/easing";
  import { app } from "./lib/state.svelte";
  import Titlebar from "./lib/Titlebar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import Editor from "./lib/Editor.svelte";
  import Settings from "./lib/Settings.svelte";
  import Logo from "./lib/Logo.svelte";
  import { templates } from "./lib/templates";

  onMount(() => {
    app.init().catch((e) => app.notify(`Startup failed: ${e}`));
  });

  function readableOn(hex: string): string {
    const n = parseInt(hex.slice(1), 16);
    const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
    return 0.299 * r + 0.587 * g + 0.114 * b > 160 ? "#0d0e10" : "#fff";
  }

  $effect(() => {
    const root = document.documentElement;
    root.style.setProperty("--accent", app.settings.accent);
    root.style.setProperty("--accent-text", readableOn(app.settings.accent));

    const query = matchMedia("(prefers-color-scheme: light)");
    const apply = () => {
      const t = app.settings.theme;
      root.dataset.theme = t === "system" ? (query.matches ? "light" : "dark") : t;
    };
    apply();
    query.addEventListener("change", apply);
    return () => query.removeEventListener("change", apply);
  });
</script>

<div class="frame">
  <Titlebar />
  {#if app.ready}
    <div class="shell">
      <Sidebar />
      <main>
        {#if app.view === "settings"}
          <Settings />
        {:else if app.selected}
          {#key app.selectedId}
            <Editor profile={app.selected} />
          {/key}
        {:else}
          <div class="blank enter">
            <Logo size={72} />
            <h2>Make your first profile</h2>
            <p>A profile is one look for your Discord activity. Pick a starting point, then click the text to make it yours.</p>
            <div class="starts">
              {#each templates as t}
                <button class="start" onclick={() => app.add(t.make())}>
                  <b>{t.label}</b>
                  <span>{t.blurb}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </main>
    </div>
  {/if}
</div>

{#if app.toast}
  <div class="toast" role="status" transition:fly={{ y: 12, duration: 220, easing: quartOut }}>{app.toast}</div>
{/if}

<style>
  .frame {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .shell {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  main {
    flex: 1;
    min-width: 0;
    height: 100%;
  }
  .blank {
    display: grid;
    gap: 10px;
    place-content: center;
    justify-items: center;
    height: 100%;
    padding: 24px;
    text-align: center;
  }
  .blank h2 {
    margin: 6px 0 0;
    font-size: 18px;
    color: var(--text-strong);
  }
  .blank p {
    max-width: 380px;
    margin: 0 0 8px;
    color: var(--muted);
  }
  .starts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    width: min(100%, 480px);
    margin-top: 6px;
  }
  .start {
    display: grid;
    gap: 2px;
    padding: 12px 14px;
    text-align: left;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    transition:
      transform 140ms var(--ease-out),
      border-color 150ms var(--ease-out);
  }
  .start:active {
    transform: scale(0.98);
  }
  @media (hover: hover) and (pointer: fine) {
    .start:hover {
      border-color: var(--accent);
    }
  }
  .start b {
    font-weight: 600;
    color: var(--text-strong);
  }
  .start span {
    font-size: 12px;
    color: var(--muted);
  }
  .toast {
    position: fixed;
    right: 18px;
    bottom: 18px;
    max-width: 380px;
    padding: 10px 14px;
    border-radius: var(--radius);
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    color: var(--text-strong);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  }
</style>
