<script lang="ts" generics="T extends string | number">
  let {
    options,
    value = $bindable(),
    label = "",
    fill = false,
  }: {
    options: { value: T; label: string }[];
    value: T;
    label?: string;
    fill?: boolean;
  } = $props();
</script>

<div class="seg" class:fill role="radiogroup" aria-label={label}>
  {#each options as o (o.value)}
    <button role="radio" aria-checked={value === o.value} class:on={value === o.value} onclick={() => (value = o.value)}>
      {o.label}
    </button>
  {/each}
</div>

<style>
  .seg {
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
  button {
    flex: 1;
    height: 28px;
    padding: 0 13px;
    border-radius: 6px;
    font-weight: 500;
    white-space: nowrap;
    color: var(--muted);
    transition:
      background-color 150ms var(--ease-out),
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
    background: var(--bg-3);
    color: var(--text-strong);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
  }
</style>
