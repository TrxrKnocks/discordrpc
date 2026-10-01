<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { app } from "./state.svelte";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  const win = getCurrentWindow();
  const mac = navigator.userAgent.includes("Mac");

  let maximized = $state(false);

  onMount(() => {
    const sync = async () => (maximized = await win.isMaximized());
    sync();
    const off = win.onResized(sync);
    return () => {
      off.then((fn) => fn());
    };
  });

  const label = $derived(
    app.status.connected
      ? `Connected${app.status.user ? ` as ${app.status.user.globalName ?? app.status.user.username}` : ""}`
      : "Discord not detected",
  );
</script>

<header data-tauri-drag-region class:mac>
  <div class="brand" data-tauri-drag-region>
    <Logo size={20} />
    DiscordRPC
  </div>

  <div class="status" data-tauri-drag-region title={app.status.error ?? ""}>
    <span class="dot" class:on={app.status.connected}></span>
    {label}
  </div>

  {#if !mac}
    <div class="controls">
      <button aria-label="Minimize" onclick={() => win.minimize()}><Icon name="minus" size={14} /></button>
      <button aria-label={maximized ? "Restore" : "Maximize"} onclick={() => win.toggleMaximize()}>
        <Icon name={maximized ? "restore" : "square"} size={12} />
      </button>
      <button class="close" aria-label="Close" onclick={() => win.close()}><Icon name="x" size={15} /></button>
    </div>
  {/if}
</header>

<style>
  header {
    display: flex;
    align-items: center;
    height: 38px;
    flex: none;
    padding-left: 12px;
    background: var(--bg-0);
    border-bottom: 1px solid var(--border);
  }
  header.mac {
    padding-left: 80px;
    padding-right: 12px;
  }
  .brand {
    display: flex;
    gap: 8px;
    align-items: center;
    font-weight: 600;
    color: var(--text-strong);
  }
  .status {
    display: flex;
    gap: 7px;
    align-items: center;
    margin-left: auto;
    padding-right: 12px;
    font-size: 12px;
    color: var(--muted);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--muted);
    transition: background-color 200ms var(--ease-out);
  }
  .dot.on {
    background: var(--ok);
  }
  .controls {
    display: flex;
    height: 100%;
  }
  .controls button {
    display: grid;
    place-items: center;
    width: 46px;
    height: 100%;
    color: var(--muted);
    transition: background-color 120ms var(--ease-out);
  }
  @media (hover: hover) and (pointer: fine) {
    .controls button:hover {
      background: var(--bg-3);
      color: var(--text-strong);
    }
    .controls .close:hover {
      background: var(--danger);
      color: #fff;
    }
  }
</style>
