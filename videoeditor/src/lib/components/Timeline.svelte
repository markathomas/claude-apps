<script lang="ts">
  import { timelineStore, timelineActions } from '$lib/stores/timelineStore';
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

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    const tag = target.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
    if (target.isContentEditable) return true;
    return false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (isEditableTarget(e.target)) return;

    const cmdOrCtrl = e.metaKey || e.ctrlKey;

    if (cmdOrCtrl && (e.key === 'z' || e.key === 'Z')) {
      e.preventDefault();
      if (e.shiftKey) {
        timelineActions.redo();
      } else {
        timelineActions.undo();
      }
      return;
    }

    if (cmdOrCtrl && (e.key === 'y' || e.key === 'Y')) {
      e.preventDefault();
      timelineActions.redo();
      return;
    }

    if (cmdOrCtrl) return;

    if (e.key === 's' || e.key === 'S') {
      e.preventDefault();
      void timelineActions.splitSelectedAt(PLAYHEAD_MS);
      return;
    }

    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      void timelineActions.deleteSelected();
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="pane timeline"
  style="--px-per-sec: {PX_PER_SEC}px"
  tabindex="0"
  role="application"
  aria-label="Timeline"
  onkeydown={handleKeydown}
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
</div>

<style>
  .pane {
    background: #111;
    height: 240px;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .pane:focus {
    outline: none;
  }
  .pane:focus-visible {
    outline: 1px solid #ffcb47;
    outline-offset: -1px;
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
