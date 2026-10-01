<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    align = "left",
    width = 320,
    padding = 14,
    fit = false,
    children,
  }: { open: boolean; align?: "left" | "right"; width?: number; padding?: number; fit?: boolean; children: Snippet } = $props();

  // Closes when the pointer goes down outside the anchor (the popover's parent) or on Escape.
  function dismiss(node: HTMLElement) {
    const anchor = node.parentElement as HTMLElement;
    if (getComputedStyle(anchor).position === "static") anchor.style.position = "relative";

    const down = (e: PointerEvent) => {
      if (!anchor.contains(e.target as Node)) open = false;
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") open = false;
    };
    window.addEventListener("pointerdown", down, true);
    window.addEventListener("keydown", key);
    return {
      destroy() {
        window.removeEventListener("pointerdown", down, true);
        window.removeEventListener("keydown", key);
      },
    };
  }
</script>

{#if open}
  <div class="pop" class:right={align === "right"} style:width={fit ? "max-content" : `${width}px`}
    style:min-width={fit ? "100%" : undefined}
    style:padding={`${padding}px`} use:dismiss role="dialog">
    {@render children()}
  </div>
{/if}

<style>
  .pop {
    position: absolute;
    z-index: 30;
    top: calc(100% + 6px);
    left: 0;
    max-width: calc(100vw - 32px);
    background: var(--bg-pop);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow:
      var(--hi),
      0 14px 40px rgba(0, 0, 0, 0.45);
    transform-origin: top left;
    animation: pop 150ms var(--ease-out);
  }
  .pop.right {
    left: auto;
    right: 0;
    transform-origin: top right;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
  }
</style>
