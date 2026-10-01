<script lang="ts">
  import { app } from "./state.svelte";
  import Icon from "./Icon.svelte";

  let {
    label,
    value = $bindable(),
    max = 128,
    placeholder = "",
    picker = true,
    hint = "",
    span = false,
  }: {
    label: string;
    value: string;
    max?: number;
    placeholder?: string;
    picker?: boolean;
    hint?: string;
    span?: boolean;
  } = $props();

  let open = $state(false);
  let input: HTMLInputElement;
  let root: HTMLElement;

  const tooShort = $derived(value.trim().length === 1);

  function insert(name: string) {
    const start = input.selectionStart ?? value.length;
    const end = input.selectionEnd ?? start;
    const token = `{${name}}`;
    value = value.slice(0, start) + token + value.slice(end);
    open = false;
    queueMicrotask(() => {
      input.focus();
      input.setSelectionRange(start + token.length, start + token.length);
    });
  }

  function outside(e: MouseEvent) {
    if (open && !root.contains(e.target as Node)) open = false;
  }

  function keys(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      open = false;
      input.focus();
    }
  }
</script>

<svelte:window onclick={outside} onkeydown={keys} />

<div class="field" class:span bind:this={root}>
  <span class="label">{label}</span>
  <div class="control">
    <input type="text" aria-label={label} bind:this={input} bind:value {placeholder} maxlength={max} class:padded={picker} />
    {#if picker}
      <button class="icon-btn pick" class:on={open} title="Insert a variable" aria-label="Insert a variable" onclick={() => (open = !open)}>
        <Icon name="var" size={14} />
      </button>
    {/if}
    {#if open}
      <ul class="menu" role="menu">
        {#each app.variables as [name, desc]}
          <li>
            <button role="menuitem" onclick={() => insert(name)}>
              <code>{`{${name}}`}</code>
              <span>{desc}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
  {#if tooShort}
    <p class="hint warn">Discord needs at least 2 characters, so this line is skipped.</p>
  {:else if hint}
    <p class="hint">{hint}</p>
  {/if}
</div>

<style>
  .span {
    grid-column: 1 / -1;
  }
  .control {
    position: relative;
  }
  input.padded {
    padding-right: 38px;
  }
  .pick {
    position: absolute;
    right: 3px;
    top: 3px;
  }
  .pick.on {
    background: var(--bg-3);
    color: var(--text-strong);
  }
  .menu {
    position: absolute;
    z-index: 20;
    right: 0;
    top: 40px;
    width: 300px;
    max-height: 260px;
    margin: 0;
    padding: 4px;
    overflow-y: auto;
    list-style: none;
    background: var(--bg-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
    transform-origin: top right;
    animation: pop 140ms var(--ease-out);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
  }
  .menu button {
    display: flex;
    gap: 12px;
    align-items: baseline;
    width: 100%;
    padding: 6px 9px;
    border-radius: 6px;
    text-align: left;
  }
  @media (hover: hover) and (pointer: fine) {
    .menu button:hover {
      background: var(--bg-3);
    }
  }
  code {
    min-width: 88px;
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 12px;
    color: var(--text-strong);
  }
  .menu span {
    color: var(--muted);
    font-size: 12px;
  }
</style>
