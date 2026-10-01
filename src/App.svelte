<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "./lib/state.svelte";
  import Titlebar from "./lib/Titlebar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import Editor from "./lib/Editor.svelte";
  import Settings from "./lib/Settings.svelte";
  import Icon from "./lib/Icon.svelte";

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
          <div class="blank">
            <span class="mark"><Icon name="signal" size={26} /></span>
            <h2>Make your first profile</h2>
            <p>A profile is one look for your Discord activity: a name, two lines of text, images and a timer. Switch between them any time.</p>
            <button class="btn primary" onclick={() => app.create()}><Icon name="plus" size={14} /> New profile</button>
          </div>
        {/if}
      </main>
    </div>
  {/if}
</div>

{#if app.toast}
  <div class="toast" role="status">{app.toast}</div>
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
  .mark {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 14px;
    background: var(--accent);
    color: var(--accent-text);
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
    animation: toast 200ms var(--ease-out);
  }
  @keyframes toast {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
</style>
