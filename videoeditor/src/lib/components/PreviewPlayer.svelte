<script lang="ts">
  import { untrack } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { mediaStore, selectedPreviewSource } from '$lib/stores/mediaStore';

  let blobUrl = $state<string | null>(null);
  let loadErr = $state<string>('');
  let status = $state<string>('');

  let assetSrc = $derived($selectedPreviewSource ? convertFileSrc($selectedPreviewSource.proxyPath) : null);

  $effect(() => {
    const src = assetSrc;
    untrack(() => {
      loadErr = '';
      status = '';
      if (blobUrl) {
        URL.revokeObjectURL(blobUrl);
        blobUrl = null;
      }
    });
    if (!src) return;

    let cancelled = false;
    void (async () => {
      try {
        status = 'fetching proxy…';
        const res = await fetch(src);
        if (!res.ok) throw new Error(`fetch ${res.status} ${res.statusText}`);
        const blob = await res.blob();
        if (cancelled) return;
        const typed = blob.type ? blob : new Blob([blob], { type: 'video/mp4' });
        const url = URL.createObjectURL(typed);
        untrack(() => {
          blobUrl = url;
          status = `loaded ${typed.size} bytes (${typed.type})`;
        });
      } catch (e) {
        if (!cancelled) loadErr = e instanceof Error ? e.message : String(e);
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  function onVideoError(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    const codes = ['', 'ABORTED', 'NETWORK', 'DECODE', 'SRC_NOT_SUPPORTED'];
    const codeName = codes[v.error?.code ?? 0] ?? String(v.error?.code);
    loadErr = `video error code=${v.error?.code} (${codeName}) msg=${v.error?.message ?? ''}`;
  }
</script>

<section class="pane preview">
  {#if $selectedPreviewSource && blobUrl}
    {#key blobUrl}
      <video class="video" src={blobUrl} controls onerror={onVideoError}>
        <track kind="captions" />
      </video>
    {/key}
  {:else if $selectedPreviewSource}
    <div class="canvas">
      <p class="placeholder">{loadErr || status || 'Loading proxy…'}</p>
    </div>
  {:else}
    <div class="canvas">
      <p class="placeholder">
        {#if $mediaStore.selectedId}Proxy not ready yet.{:else}Select a clip to preview.{/if}
      </p>
    </div>
  {/if}
  {#if $selectedPreviewSource && assetSrc}
    <div class="diag">
      <div>proxy_path: {$selectedPreviewSource.proxyPath}</div>
      <div>asset src: {assetSrc}</div>
      {#if status}<div>{status}</div>{/if}
      {#if loadErr}<div class="err">{loadErr}</div>{/if}
    </div>
  {/if}
</section>

<style>
  .pane { display: flex; flex-direction: column; min-height: 0; }
  .video { width: 100%; height: 100%; background: #000; object-fit: contain; }
  .canvas { flex: 1; display: grid; place-items: center; background: #0a0a0a; }
  .placeholder { opacity: 0.7; font-size: 0.875rem; color: #ccc; padding: 1rem; text-align: center; }
  .diag { font-family: ui-monospace, monospace; font-size: 0.72rem; padding: 0.5rem; color: #ccc; background: #111; word-break: break-all; }
  .diag .err { color: #f88; margin-top: 0.25rem; }
</style>
