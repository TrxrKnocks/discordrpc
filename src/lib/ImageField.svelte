<script lang="ts">
  import TextField from "./TextField.svelte";
  import Icon from "./Icon.svelte";

  let {
    title,
    url = $bindable(),
    tooltip = $bindable(),
    round = false,
  }: { title: string; url: string; tooltip: string; round?: boolean } = $props();

  let failed = $state(false);
  const src = $derived(url.trim());

  $effect(() => {
    src;
    failed = false;
  });
</script>

<div class="image">
  <div class="thumb" class:round>
    {#if src && !failed}
      <img {src} alt="" onerror={() => (failed = true)} />
    {:else}
      <Icon name="image" size={22} />
    {/if}
  </div>
  <div class="inputs">
    <div class="name">{title}</div>
    <TextField label="Image URL" bind:value={url} max={512} picker={false} placeholder="https://example.com/image.png" hint={failed ? "That link doesn't load as an image." : ""} />
    <TextField label="Hover text" bind:value={tooltip} />
  </div>
</div>

<style>
  .image {
    display: flex;
    gap: 16px;
    padding: 16px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .thumb {
    display: grid;
    flex: none;
    place-items: center;
    width: 84px;
    height: 84px;
    overflow: hidden;
    border-radius: 10px;
    background: var(--bg-input);
    border: 1px dashed var(--border-strong);
    color: var(--muted);
  }
  .thumb.round {
    border-radius: 50%;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .inputs {
    display: grid;
    flex: 1;
    gap: 12px;
    min-width: 0;
  }
  .name {
    font-weight: 600;
    color: var(--text-strong);
  }
</style>
