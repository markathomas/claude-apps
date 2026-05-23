<script lang="ts">
  import { derived } from 'svelte/store';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { mediaStore } from '$lib/stores/mediaStore';

  const preview = derived(mediaStore, ($s) => {
    if (!$s.selectedId) return null;
    const item = $s.items.find((i) => i.id === $s.selectedId);
    if (!item || item.proxy_status !== 'ready' || !item.proxy_path) return null;
    return { src: convertFileSrc(item.proxy_path) };
  });
</script>

<section class="pane preview">
  {#if $preview}
    {#key $preview.src}
      <video class="video" src={$preview.src} controls>
        <track kind="captions" />
      </video>
    {/key}
  {:else}
    <div class="canvas">
      <p class="placeholder">
        {#if $mediaStore.selectedId}
          Proxy not ready yet.
        {:else}
          Select a clip to preview.
        {/if}
      </p>
    </div>
  {/if}
</section>

<style>
  .pane { display: flex; flex-direction: column; min-height: 0; }
  .video { width: 100%; height: 100%; background: #000; object-fit: contain; }
  .canvas { flex: 1; display: grid; place-items: center; background: #0a0a0a; }
  .placeholder { opacity: 0.5; font-size: 0.875rem; }
</style>
