<script lang="ts">
  import type { VideoClip, AudioClip } from '$lib/types';
  import { msToPx } from '$lib/lib/time';

  interface Props {
    clip: VideoClip | AudioClip;
    pxPerSec: number;
    kind: 'video' | 'audio';
  }

  const { clip, pxPerSec, kind }: Props = $props();

  const left = $derived(msToPx(clip.timeline_start_ms, pxPerSec));
  const width = $derived(
    msToPx(clip.source_out_ms - clip.source_in_ms, pxPerSec),
  );
</script>

<div
  class="clip"
  class:video={kind === 'video'}
  class:audio={kind === 'audio'}
  style="left: {left}px; width: {width}px"
  data-clip-id={clip.id}
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
  }
  .clip.audio {
    background: #2a6a4d;
    border-color: #3b9b6c;
  }
  .clip:hover {
    filter: brightness(1.15);
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
