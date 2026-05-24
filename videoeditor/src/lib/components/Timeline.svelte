<script lang="ts">
  import { timelineStore } from '$lib/stores/timelineStore';
  import TimelineRuler from './TimelineRuler.svelte';
  import TimelineTrack from './TimelineTrack.svelte';

  const PX_PER_SEC = 60;
  const PLAYHEAD_MS = 0;

  const timeline = $derived($timelineStore.timeline);
  const totalSeconds = $derived(
    Math.max(10, Math.ceil(timeline.duration_ms / 1000) + 5),
  );
  const widthPx = $derived(totalSeconds * PX_PER_SEC);
  const playheadPx = $derived((PLAYHEAD_MS / 1000) * PX_PER_SEC);
</script>

<section
  class="pane timeline"
  style="--px-per-sec: {PX_PER_SEC}px"
>
  <div class="scroll">
    <div class="content" style="width: {widthPx}px">
      <TimelineRuler durationMs={timeline.duration_ms} pxPerSec={PX_PER_SEC} />
      <TimelineTrack
        kind="video"
        clips={timeline.video_track}
        pxPerSec={PX_PER_SEC}
        {widthPx}
      />
      <TimelineTrack
        kind="audio"
        clips={timeline.audio_track}
        pxPerSec={PX_PER_SEC}
        {widthPx}
      />
      <div class="playhead" style="left: {playheadPx}px"></div>
    </div>
  </div>
</section>

<style>
  .pane {
    background: #111;
    height: 240px;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .scroll {
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
  }
  .content {
    position: relative;
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }
  .playhead {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: #ef4444;
    pointer-events: none;
    box-shadow: 0 0 4px rgba(239, 68, 68, 0.5);
  }
</style>
