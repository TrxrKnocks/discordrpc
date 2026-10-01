<script lang="ts">
  import { open, ask } from "@tauri-apps/plugin-dialog";
  import { app } from "./state.svelte";
  import { cue } from "./sound";
  import Icon from "./Icon.svelte";

  let {
    title,
    url = $bindable(),
    text = $bindable(),
  }: { title: string; url: string; text: string } = $props();

  let busy = $state(false);

  async function upload() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp"] }],
    });
    if (typeof path !== "string") return;

    if (!app.settings.uploadConsent) {
      const agreed = await ask(
        "The image is uploaded to catbox.moe, a free public file host, so Discord can load it from a link. Anyone who has the link can see it. Upload it?",
        { title: "Upload image", kind: "info", okLabel: "Upload", cancelLabel: "Cancel" },
      );
      if (!agreed) return;
      await app.updateSettings({ uploadConsent: true });
    }

    busy = true;
    try {
      url = (await app.uploadImage(path)).url;
      cue("add");
    } catch (e) {
      app.notify(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="picker">
  <div class="head">
    <span class="title">{title}</span>
    <button class="btn ghost small" onclick={upload} disabled={busy}>
      <Icon name="export" size={13} />
      {busy ? "Uploading…" : "Upload"}
    </button>
  </div>

  <div class="stack">
    <input type="text" aria-label={`${title} link`} placeholder="Image link (https://...)" maxlength="512" bind:value={url} />
    <input type="text" aria-label={`${title} hover text`} placeholder="Hover text (optional)" maxlength="128" bind:value={text} />
  </div>

  {#if app.images.length}
    <div class="lib" aria-label="Uploaded images">
      {#each app.images.slice(0, 12) as img (img.url)}
        <div class="thumb" class:on={url === img.url}>
          <button class="pick" title={img.name} aria-label={`Use ${img.name}`} onclick={() => (url = img.url)}>
            <img src={img.url} alt="" loading="lazy" />
          </button>
          <button class="del" title="Remove from this list" aria-label={`Remove ${img.name} from the list`} onclick={() => app.removeImage(img.url)}>
            <Icon name="x" size={10} />
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .title {
    font-weight: 600;
    color: var(--text-strong);
  }
  .stack {
    display: grid;
    gap: 8px;
  }
  .lib {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 10px;
  }
  .thumb {
    position: relative;
    width: 40px;
    height: 40px;
  }
  .pick {
    width: 100%;
    height: 100%;
    padding: 0;
    overflow: hidden;
    border: 2px solid transparent;
    border-radius: 8px;
    background: var(--bg-input);
    transition:
      transform 120ms var(--ease-out),
      border-color 150ms var(--ease-out);
  }
  .pick:active {
    transform: scale(0.95);
  }
  .thumb.on .pick {
    border-color: var(--accent);
  }
  .pick img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .del {
    position: absolute;
    top: -5px;
    right: -5px;
    display: none;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--bg-3);
    border: 1px solid var(--border-strong);
    color: var(--text-strong);
  }
  .thumb:hover .del,
  .thumb:focus-within .del {
    display: grid;
  }
</style>
