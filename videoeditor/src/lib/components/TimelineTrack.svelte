<script lang="ts">
  import type { VideoClip, AudioClip } from '$lib/types';
  import TimelineClip from './TimelineClip.svelte';

  interface Props {
    kind: 'video' | 'audio';
    clips: VideoClip[] | AudioClip[];
    pxPerSec: number;
    widthPx: number;
  }

  const { kind, clips, pxPerSec, widthPx }: Props = $props();
</script>

<div
  class="track"
  class:video={kind === 'video'}
  class:audio={kind === 'audio'}
  style="width: {widthPx}px"
  data-track={kind}
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
