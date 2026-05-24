<script lang="ts">
  import type { VideoClip, AudioClip } from '$lib/types';
  import TimelineClip from './TimelineClip.svelte';
  import { pxToMs } from '$lib/lib/time';
  import { timelineActions } from '$lib/stores/timelineStore';

  interface Props {
    kind: 'video' | 'audio';
    clips: VideoClip[] | AudioClip[];
    pxPerSec: number;
    widthPx: number;
  }

  const { kind, clips, pxPerSec, widthPx }: Props = $props();

  const MEDIA_MIME = 'application/x-videoeditor-media-id';

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
    <TimelineClip {clip} {pxPerSec} {kind} />
  {/each}
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
</style>
