<script lang="ts" generics="T extends string | number">
  import Popover from "./Popover.svelte";
  import Icon from "./Icon.svelte";
  import { cue } from "./sound";

  let {
    options,
    value = $bindable(),
    label,
    variant = "field",
  }: {
    options: readonly { value: T; label: string }[];
    value: T;
    label: string;
    variant?: "field" | "inline";
  } = $props();

  let open = $state(false);
  let active = $state(0);
  let trigger: HTMLButtonElement;

  const current = $derived(options.find((o) => o.value === value));

  function show() {
    active = Math.max(0, options.findIndex((o) => o.value === value));
    open = true;
  }

  function choose(v: T) {
    if (v !== value) cue("tick");
    value = v;
    open = false;
    trigger.focus();
  }

  function keys(e: KeyboardEvent) {
    if (!open) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        show();
      }
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      active = (active + 1) % options.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active = (active - 1 + options.length) % options.length;
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      choose(options[active].value);
    } else if (e.key === "Tab") {
      open = false;
    }
  }
</script>

<div class="anchor">
  <button
    bind:this={trigger}
    class="trigger {variant}"
    class:open
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={label}
    onclick={() => (open ? (open = false) : show())}
    onkeydown={keys}
  >
    <span>{current?.label}</span>
    <Icon name="chevron" size={14} />
  </button>

  <Popover bind:open fit padding={4}>
    <ul role="listbox" aria-label={label}>
      {#each options as o, i (o.value)}
        <li>
          <button
            role="option"
            aria-selected={o.value === value}
            class:active={i === active}
            tabindex="-1"
            onmouseenter={() => (active = i)}
            onclick={() => choose(o.value)}
          >
            <span>{o.label}</span>
            {#if o.value === value}<Icon name="check" size={14} />{/if}
          </button>
        </li>
      {/each}
    </ul>
  </Popover>
</div>

<style>
  .anchor {
    position: relative;
  }
  .trigger {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    transition:
      background-color 150ms var(--ease-out),
      border-color 150ms var(--ease-out),
      transform 140ms var(--ease-out);
  }
  .trigger:active {
    transform: scale(0.98);
  }
  .trigger :global(svg) {
    flex: none;
    color: var(--muted);
    transition: transform 180ms var(--ease-out);
  }
  .trigger.open :global(svg) {
    transform: rotate(180deg);
  }
  .field {
    width: 100%;
    height: 34px;
    padding: 0 10px 0 11px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .inline {
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    font-weight: 700;
    color: var(--text-strong);
    border-radius: 6px;
  }
  @media (hover: hover) and (pointer: fine) {
    .field:hover {
      border-color: var(--border-strong);
    }
    .inline:hover {
      background: var(--bg-3);
    }
  }
  .trigger.open {
    border-color: var(--accent);
  }
  .inline.open {
    background: var(--bg-3);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li button {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-width: 150px;
    padding: 7px 10px;
    border-radius: 6px;
    text-align: left;
    white-space: nowrap;
  }
  li button.active {
    background: var(--bg-3);
    color: var(--text-strong);
  }
  li button[aria-selected="true"] {
    color: var(--text-strong);
    font-weight: 500;
  }
  li button :global(svg) {
    color: var(--accent);
  }
</style>
