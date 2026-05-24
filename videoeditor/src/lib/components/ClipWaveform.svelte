<script lang="ts">
  import { onDestroy } from 'svelte';
  import { ipc } from '$lib/ipc';
  import { msToPx } from '$lib/lib/time';

  interface Props {
    mediaId: string;
    sourceInMs: number;
    sourceOutMs: number;
    pxPerSec: number;
    heightPx?: number;
  }

  const DEFAULT_HEIGHT = 44;

  const {
    mediaId,
    sourceInMs,
    sourceOutMs,
    pxPerSec,
    heightPx = DEFAULT_HEIGHT,
  }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let peaks = $state<number[]>([]);
  let bucketMs = $state(0);
  let loadedMediaId = $state<string | null>(null);
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
    if (peaks.length === 0 || bucketMs <= 0) return;

    const halfH = h / 2;
    const baselineY = h - halfH / 2;
    const durationMs = sourceOutMs - sourceInMs;
    if (durationMs <= 0) return;

    ctx.strokeStyle = 'rgba(255, 255, 255, 0.7)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let col = 0; col < w; col += 1) {
      const colStartMs = sourceInMs + (col / w) * durationMs;
      const colEndMs = sourceInMs + ((col + 1) / w) * durationMs;
      const startBucket = Math.max(0, Math.floor(colStartMs / bucketMs));
      const endBucket = Math.min(
        peaks.length - 1,
        Math.max(startBucket, Math.floor(colEndMs / bucketMs)),
      );
      let peak = 0;
      for (let b = startBucket; b <= endBucket; b += 1) {
        const v = peaks[b] ?? 0;
        if (v > peak) peak = v;
      }
      const lineH = Math.max(0, Math.min(halfH, peak * halfH));
      const x = col + 0.5;
      ctx.moveTo(x, baselineY - lineH / 2);
      ctx.lineTo(x, baselineY + lineH / 2);
    }
    ctx.stroke();
  }

  async function load(id: string): Promise<void> {
    if (loadedMediaId === id) return;
    loadedMediaId = id;
    peaks = [];
    bucketMs = 0;
    try {
      const wf = await ipc.readWaveform(id);
      if (cancelled || loadedMediaId !== id) return;
      peaks = wf.peaks;
      bucketMs = wf.bucket_ms;
      requestRedraw();
    } catch {
      peaks = [];
      bucketMs = 0;
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
  class="waveform"
  aria-hidden="true"
></canvas>

<style>
  .waveform {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 0;
  }
</style>
