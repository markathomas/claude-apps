<script lang="ts">
  import { onDestroy } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { ipc } from '$lib/ipc';
  import type { ThumbEntry } from '$lib/types';
  import { msToPx } from '$lib/lib/time';

  interface Props {
    mediaId: string;
    sourceInMs: number;
    sourceOutMs: number;
    pxPerSec: number;
    heightPx?: number;
  }

  const THUMB_INTERVAL_MS = 1000;
  const DEFAULT_HEIGHT = 48;

  const {
    mediaId,
    sourceInMs,
    sourceOutMs,
    pxPerSec,
    heightPx = DEFAULT_HEIGHT,
  }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let entries = $state<ThumbEntry[]>([]);
  let loadedMediaId = $state<string | null>(null);
  const images = new Map<string, HTMLImageElement>();
  let rafHandle = 0;
  let cancelled = false;

  const widthPx = $derived(
    Math.max(1, Math.round(msToPx(sourceOutMs - sourceInMs, pxPerSec))),
  );

  function requestRedraw(): void {
    if (rafHandle !== 0) return;
    rafHandle = requestAnimationFrame(() => {
      rafHandle = 0;
      draw();
    });
  }

  function draw(): void {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);
    if (entries.length === 0) return;
    const thumbWidthPx = Math.max(1, msToPx(THUMB_INTERVAL_MS, pxPerSec));
    for (const entry of entries) {
      const endMs = entry.time_ms + THUMB_INTERVAL_MS;
      if (endMs <= sourceInMs) continue;
      if (entry.time_ms >= sourceOutMs) continue;
      const img = images.get(entry.path);
      if (!img || !img.complete || img.naturalWidth === 0) continue;
      const x = msToPx(entry.time_ms - sourceInMs, pxPerSec);
      ctx.drawImage(img, x, 0, thumbWidthPx, h);
    }
  }

  async function load(id: string): Promise<void> {
    if (loadedMediaId === id) return;
    loadedMediaId = id;
    images.clear();
    entries = [];
    try {
      const next = await ipc.listThumbnails(id);
      if (cancelled || loadedMediaId !== id) return;
      entries = next;
      for (const entry of next) {
        const img = new Image();
        img.onload = () => requestRedraw();
        img.src = convertFileSrc(entry.path);
        images.set(entry.path, img);
      }
      requestRedraw();
    } catch {
      entries = [];
    }
  }

  $effect(() => {
    void load(mediaId);
  });

  $effect(() => {
    void widthPx;
    void heightPx;
    void sourceInMs;
    void sourceOutMs;
    void pxPerSec;
    requestRedraw();
  });

  onDestroy(() => {
    cancelled = true;
    if (rafHandle !== 0) {
      cancelAnimationFrame(rafHandle);
      rafHandle = 0;
    }
  });
</script>

<canvas
  bind:this={canvas}
  width={widthPx}
  height={heightPx}
  class="filmstrip"
  aria-hidden="true"
></canvas>

<style>
  .filmstrip {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 0;
    image-rendering: -webkit-optimize-contrast;
  }
</style>
