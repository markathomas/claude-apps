<script lang="ts">
  import { untrack } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { mediaStore, selectedPreviewSource } from '$lib/stores/mediaStore';
  import { timelineStore } from '$lib/stores/timelineStore';
  import { playheadStore, activeClipAt } from '$lib/stores/playheadStore';
  import type { VideoClip, AudioClip, MediaItem } from '$lib/types';

  let videoEl: HTMLVideoElement | null = $state(null);
  let blobUrl = $state<string | null>(null);
  let loadErr = $state<string>('');
  let status = $state<string>('');

  const timeline = $derived($timelineStore.timeline);
  const playheadMs = $derived($playheadStore.playheadMs);
  const playing = $derived($playheadStore.playing);

  const active = $derived(activeClipAt(timeline, playheadMs));

  function findMedia(mediaId: string): MediaItem | undefined {
    return $mediaStore.items.find((i) => i.id === mediaId);
  }

  const activeMedia = $derived(
    active ? findMedia(active.clip.media_id) : undefined,
  );

  const fallbackPreview = $derived(active ? null : $selectedPreviewSource);

  const activeProxyPath = $derived(
    activeMedia && activeMedia.proxy_status === 'ready' && activeMedia.proxy_path
      ? activeMedia.proxy_path
      : null,
  );

  const proxyPath = $derived(activeProxyPath ?? fallbackPreview?.proxyPath ?? null);
  const assetSrc = $derived(proxyPath ? convertFileSrc(proxyPath) : null);

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

  $effect(() => {
    const v = videoEl;
    if (!v || !active) return;
    const clip = active.clip as VideoClip | AudioClip;
    const targetSec =
      (playheadMs - clip.timeline_start_ms + clip.source_in_ms) / 1000;
    if (Math.abs(v.currentTime - targetSec) > 0.05) {
      try {
        v.currentTime = targetSec;
      } catch {
        // ignore — happens before metadata is loaded
      }
    }
  });

  $effect(() => {
    const v = videoEl;
    if (!v) return;
    if (playing && active) {
      void v.play().catch(() => {});
    } else {
      v.pause();
    }
  });

  function onVideoError(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    const codes = ['', 'ABORTED', 'NETWORK', 'DECODE', 'SRC_NOT_SUPPORTED'];
    const codeName = codes[v.error?.code ?? 0] ?? String(v.error?.code);
    loadErr = `video error code=${v.error?.code} (${codeName}) msg=${v.error?.message ?? ''}`;
  }
</script>

<section class="pane preview">
  {#if proxyPath && blobUrl}
    {#key blobUrl}
      <video
        class="video"
        src={blobUrl}
        bind:this={videoEl}
        onerror={onVideoError}
      >
        <track kind="captions" />
      </video>
    {/key}
  {:else if proxyPath}
    <div class="canvas">
      <p class="placeholder">{loadErr || status || 'Loading proxy…'}</p>
    </div>
  {:else}
    <div class="canvas">
      <p class="placeholder">
        {#if timeline.video_track.length > 0 || timeline.audio_track.length > 0}
          Move the playhead onto a clip to preview.
        {:else if $mediaStore.selectedId}
          Proxy not ready yet.
        {:else}
          Add a clip to the timeline to preview.
        {/if}
      </p>
    </div>
  {/if}
  {#if proxyPath && assetSrc}
    <div class="diag">
      <div>proxy_path: {proxyPath}</div>
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
