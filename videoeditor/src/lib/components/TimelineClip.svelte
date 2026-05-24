<script lang="ts">
  import type { VideoClip, AudioClip } from '$lib/types';
  import { msToPx, pxToMs } from '$lib/lib/time';
  import { snap, type SnapEdge } from '$lib/lib/snap';
  import { timelineActions } from '$lib/stores/timelineStore';

  interface Props {
    clip: VideoClip | AudioClip;
    pxPerSec: number;
    kind: 'video' | 'audio';
    track: 'video' | 'audio';
    siblingEdges: readonly SnapEdge[];
    onSnapPreview?: (ms: number | null) => void;
  }

  const SNAP_THRESHOLD_MS = 166;

  const {
    clip,
    pxPerSec,
    kind,
    track,
    siblingEdges,
    onSnapPreview,
  }: Props = $props();

  const baseLeft = $derived(msToPx(clip.timeline_start_ms, pxPerSec));
  const width = $derived(
    msToPx(clip.source_out_ms - clip.source_in_ms, pxPerSec),
  );

  let dragging = $state(false);
  let dragOffsetPx = $state(0);
  let pointerStartX = 0;
  let originalStartMs = 0;

  function candidateStartMs(deltaPx: number): number {
    const deltaMs = pxToMs(deltaPx, pxPerSec);
    const candidate = Math.max(0, originalStartMs + deltaMs);
    const result = snap(candidate, siblingEdges, SNAP_THRESHOLD_MS);
    return result.snapped;
  }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    const target = e.currentTarget as HTMLDivElement;
    target.setPointerCapture(e.pointerId);
    pointerStartX = e.clientX;
    originalStartMs = clip.timeline_start_ms;
    dragOffsetPx = 0;
    dragging = true;
    e.preventDefault();
  }

  function handlePointerMove(e: PointerEvent) {
    if (!dragging) return;
    const deltaPx = e.clientX - pointerStartX;
    const snappedMs = candidateStartMs(deltaPx);
    dragOffsetPx = msToPx(snappedMs - originalStartMs, pxPerSec);
    onSnapPreview?.(snappedMs);
  }

  async function handlePointerUp(e: PointerEvent) {
    if (!dragging) return;
    const target = e.currentTarget as HTMLDivElement;
    if (target.hasPointerCapture(e.pointerId)) {
      target.releasePointerCapture(e.pointerId);
    }
    const deltaPx = e.clientX - pointerStartX;
    const finalMs = candidateStartMs(deltaPx);
    dragging = false;
    dragOffsetPx = 0;
    onSnapPreview?.(null);
    if (finalMs !== originalStartMs) {
      await timelineActions.moveClip(track, clip.id, finalMs);
    }
  }

  function handlePointerCancel(e: PointerEvent) {
    if (!dragging) return;
    const target = e.currentTarget as HTMLDivElement;
    if (target.hasPointerCapture(e.pointerId)) {
      target.releasePointerCapture(e.pointerId);
    }
    dragging = false;
    dragOffsetPx = 0;
    onSnapPreview?.(null);
  }
</script>

<div
  class="clip"
  class:video={kind === 'video'}
  class:audio={kind === 'audio'}
  class:dragging
  style="left: {baseLeft}px; width: {width}px; transform: translateX({dragOffsetPx}px)"
  data-clip-id={clip.id}
  role="button"
  tabindex="0"
  aria-label="Clip {clip.id.slice(0, 6)}"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerCancel}
>
  <span class="label">{clip.id.slice(0, 6)}</span>
</div>

<style>
  .clip {
    position: absolute;
    top: 4px;
    bottom: 4px;
    border-radius: 3px;
    background: #2a4d8f;
    border: 1px solid #3b6bc9;
    overflow: hidden;
    cursor: grab;
    user-select: none;
    touch-action: none;
  }
  .clip.audio {
    background: #2a6a4d;
    border-color: #3b9b6c;
  }
  .clip:hover {
    filter: brightness(1.15);
  }
  .clip.dragging {
    cursor: grabbing;
    z-index: 2;
    filter: brightness(1.2);
  }
  .label {
    display: block;
    padding: 0.15rem 0.35rem;
    font-size: 0.7rem;
    color: #e6edf3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }
</style>
