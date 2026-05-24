import { writable, get } from 'svelte/store';
import type { Timeline } from '$lib/types';
import { ipc } from '$lib/ipc';
import { mediaStore } from './mediaStore';
import { projectActions } from './projectStore';

export const TIMELINE_HISTORY_LIMIT = 100;

export interface TimelineState {
  timeline: Timeline;
  canUndo: boolean;
  canRedo: boolean;
}

function emptyTimeline(): Timeline {
  return { duration_ms: 0, video_track: [], audio_track: [], text_track: [] };
}

const undoStack: Timeline[] = [];
const redoStack: Timeline[] = [];

const internal = writable<TimelineState>({
  timeline: emptyTimeline(),
  canUndo: false,
  canRedo: false,
});

export const timelineStore = internal;

function publish(timeline: Timeline): void {
  internal.set({
    timeline,
    canUndo: undoStack.length > 0,
    canRedo: redoStack.length > 0,
  });
}

export const timelineActions = {
  reset(timeline?: Timeline): void {
    undoStack.length = 0;
    redoStack.length = 0;
    publish(timeline ?? emptyTimeline());
  },

  apply(next: Timeline): void {
    let current: Timeline = emptyTimeline();
    internal.update((s) => {
      current = s.timeline;
      return s;
    });
    undoStack.push(current);
    if (undoStack.length > TIMELINE_HISTORY_LIMIT) {
      undoStack.shift();
    }
    redoStack.length = 0;
    publish(next);
    projectActions.setTimeline(next);
  },

  undo(): void {
    if (undoStack.length === 0) return;
    let current: Timeline = emptyTimeline();
    internal.update((s) => {
      current = s.timeline;
      return s;
    });
    const prev = undoStack.pop() as Timeline;
    redoStack.push(current);
    publish(prev);
    projectActions.setTimeline(prev);
  },

  redo(): void {
    if (redoStack.length === 0) return;
    let current: Timeline = emptyTimeline();
    internal.update((s) => {
      current = s.timeline;
      return s;
    });
    const next = redoStack.pop() as Timeline;
    undoStack.push(current);
    publish(next);
    projectActions.setTimeline(next);
  },

  async insertClipFromMedia(mediaId: string, dropMs: number): Promise<void> {
    const item = get(mediaStore).items.find((i) => i.id === mediaId);
    if (!item || item.proxy_status !== 'ready' || !item.probe) return;

    const startMs = Math.max(0, Math.round(dropMs));
    const current = get(internal).timeline;

    try {
      const next = await ipc.timelineInsertClip(
        current,
        'video',
        mediaId,
        startMs,
        0,
        item.probe.duration_ms,
      );
      timelineActions.apply(next);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('insertClipFromMedia failed:', message);
    }
  },
};
