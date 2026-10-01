<script lang="ts">
  import { app } from "./state.svelte";

  let {
    value = $bindable(),
    placeholder = "",
    max = 128,
    label,
    kind = "line",
    picker = true,
  }: {
    value: string;
    placeholder?: string;
    max?: number;
    label: string;
    kind?: "title" | "line";
    picker?: boolean;
  } = $props();

  let el: HTMLInputElement;
  const tooShort = $derived(value.trim().length === 1);
</script>

<input
  bind:this={el}
  bind:value
  type="text"
  class={kind}
  {placeholder}
  maxlength={max}
  aria-label={label}
  title={tooShort ? "Discord needs at least 2 characters, so this line is skipped" : ""}
  class:short={tooShort}
  onfocus={() => picker && (app.lastField = el)}
/>

<style>
  input[type="text"] {
    width: calc(100% + 12px);
    height: auto;
    padding: 3px 6px;
    margin: 0 -6px;
    background: transparent;
    border: 1px dashed transparent;
    border-radius: 6px;
    text-overflow: ellipsis;
  }
  input[type="text"].title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-strong);
  }
  input[type="text"].line {
    font-size: 13.5px;
  }
  @media (hover: hover) and (pointer: fine) {
    input[type="text"]:hover:not(:focus) {
      border-color: var(--border-strong);
    }
  }
  input[type="text"]:focus {
    background: var(--bg-input);
    border: 1px solid var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent), transparent 80%);
  }
  input[type="text"].short {
    border-color: var(--warn);
  }
</style>
