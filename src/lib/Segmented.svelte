<script lang="ts" generics="T extends string | number">
  import { onMount } from "svelte";
  import { cue } from "./sound";

  let {
    options,
    value = $bindable(),
    label = "",
    fill = false,
    quiet = false,
  }: {
    options: readonly { value: T; label: string }[];
    value: T;
    label?: string;
    fill?: boolean;
    quiet?: boolean;
  } = $props();

  let root: HTMLElement;
  let buttons = $state<HTMLButtonElement[]>([]);
  // Kept as separate primitives: measure() writes them from inside an effect,
  // so it must never read state it also assigns.
  let x = $state(0);
  let w = $state(0);
  let ready = $state(false);

  function measure() {
    const b = buttons[options.findIndex((o) => o.value === value)];
    if (!b) return;
    x = b.offsetLeft;
    w = b.offsetWidth;
  }

  $effect(() => {
    value;
    options;
    measure();
  });

  onMount(() => {
    measure();
    // Let the first placement happen without sliding in from zero.
    requestAnimationFrame(() => (ready = true));
    const ro = new ResizeObserver(measure);
    ro.observe(root);
    return () => ro.disconnect();
  });
</script>

<div class="seg" class:fill role="radiogroup" aria-label={label} bind:this={root}>
  <span class="pill" class:ready style:width={`${w}px`} style:transform={`translateX(${x}px)`}></span>
  {#each options as o, i (o.value)}
    <button bind:this={buttons[i]} role="radio" aria-checked={value === o.value} class:on={value === o.value} onclick={() => {
        if (!quiet && value !== o.value) cue("tick");
        value = o.value;
      }}>
      {o.label}
    </button>
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    background: var(--bg-0);
    border: 1px solid var(--border);
    border-radius: 9px;
  }
  .seg.fill {
    display: flex;
  }
  .pill {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 0;
    border-radius: 6px;
    background: var(--bg-3);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.06),
      0 1px 2px rgba(0, 0, 0, 0.3);
  }
  .pill.ready {
    transition:
      transform 240ms var(--ease-out),
      width 240ms var(--ease-out);
  }
  button {
    position: relative;
    z-index: 1;
    flex: 1;
    height: 28px;
    padding: 0 13px;
    border-radius: 6px;
    font-weight: 500;
    white-space: nowrap;
    color: var(--muted);
    transition:
      color 150ms var(--ease-out),
      transform 120ms var(--ease-out);
  }
  button:active {
    transform: scale(0.97);
  }
  @media (hover: hover) and (pointer: fine) {
    button:not(.on):hover {
      color: var(--text-strong);
    }
  }
  button.on {
    color: var(--text-strong);
  }
</style>
