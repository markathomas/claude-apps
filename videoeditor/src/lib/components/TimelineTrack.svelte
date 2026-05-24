<script lang="ts">
  import type { VideoClip, AudioClip } from '$lib/types';
  import TimelineClip from './TimelineClip.svelte';
  import { msToPx, pxToMs } from '$lib/lib/time';
  import type { SnapEdge } from '$lib/lib/snap';
  import { timelineActions } from '$lib/stores/timelineStore';
  import { mediaStore } from '$lib/stores/mediaStore';

  interface Props {
    kind: 'video' | 'audio';
    clips: VideoClip[] | AudioClip[];
    pxPerSec: number;
    widthPx: number;
  }

  const { kind, clips, pxPerSec, widthPx }: Props = $props();

  const mediaItems = $derived($mediaStore.items);

  function mediaDurationFor(mediaId: string): number {
    const item = mediaItems.find((i) => i.id === mediaId);
    return item?.probe?.duration_ms ?? 0;
  }

  function hasAudioFor(mediaId: string): boolean {
    const item = mediaItems.find((i) => i.id === mediaId);
    return item?.probe?.has_audio ?? false;
  }

  const MEDIA_MIME = 'application/x-videoeditor-media-id';

  let snapPreviewMs = $state<number | null>(null);

  function edgesExcluding(clipId: string): SnapEdge[] {
    const edges: SnapEdge[] = [];
    for (const c of clips) {
      if (c.id === clipId) continue;
      edges.push({ ms: c.timeline_start_ms, source: 'start' });
      edges.push({
        ms: c.timeline_start_ms + (c.source_out_ms - c.source_in_ms),
        source: 'end',
      });
    }
    return edges;
  }

  function handleDragOver(e: DragEvent) {
    if (kind !== 'video') return;
    if (!e.dataTransfer) return;
    if (!e.dataTransfer.types.includes(MEDIA_MIME)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }

  function handleDrop(e: DragEvent) {
    if (kind !== 'video') return;
    if (!e.dataTransfer) return;
    const mediaId = e.dataTransfer.getData(MEDIA_MIME);
    if (!mediaId) return;
    e.preventDefault();
    const target = e.currentTarget as HTMLDivElement;
    const rect = target.getBoundingClientRect();
    const dropPx = e.clientX - rect.left;
    const dropMs = pxToMs(dropPx, pxPerSec);
    void timelineActions.insertClipFromMedia(mediaId, dropMs);
  }
</script>

<div
  class="track"
  class:video={kind === 'video'}
  class:audio={kind === 'audio'}
  style="width: {widthPx}px"
  data-track={kind}
  role="region"
  aria-label={kind === 'video' ? 'Video track' : 'Audio track'}
  ondragover={handleDragOver}
  ondrop={handleDrop}
>
  {#each clips as clip (clip.id)}
    <TimelineClip
      {clip}
      {pxPerSec}
      {kind}
      track={kind}
      siblingEdges={edgesExcluding(clip.id)}
      mediaDurationMs={mediaDurationFor(clip.media_id)}
      hasAudio={hasAudioFor(clip.media_id)}
      onSnapPreview={(ms) => (snapPreviewMs = ms)}
    />
  {/each}
  {#if snapPreviewMs !== null}
    <div
      class="snap-line"
      style="left: {msToPx(snapPreviewMs, pxPerSec)}px"
      aria-hidden="true"
    ></div>
  {/if}
</div>

<style>
  .track {
    position: relative;
    height: 56px;
    background: #161616;
    border-bottom: 1px solid #2a2a2a;
    flex-shrink: 0;
  }
  .track.audio {
    height: 44px;
    background: #131715;
  }
  .snap-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: #ffcb47;
    box-shadow: 0 0 4px rgba(255, 203, 71, 0.7);
    pointer-events: none;
  }
</style>
