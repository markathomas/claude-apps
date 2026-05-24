import { writable, get } from 'svelte/store';
import type { Timeline, VideoClip, AudioClip } from '$lib/types';
import { ipc, type TimelineTrack } from '$lib/ipc';
import { mediaStore } from './mediaStore';
import { projectActions } from './projectStore';

export const TIMELINE_HISTORY_LIMIT = 100;

export interface TimelineState {
  timeline: Timeline;
  canUndo: boolean;
  canRedo: boolean;
  selectedClipId: string | null;
}

function emptyTimeline(): Timeline {
  return { duration_ms: 0, video_track: [], audio_track: [], text_track: [] };
}

const undoStack: Timeline[] = [];
const redoStack: Timeline[] = [];

let selectedClipId: string | null = null;

const internal = writable<TimelineState>({
  timeline: emptyTimeline(),
  canUndo: false,
  canRedo: false,
  selectedClipId: null,
});

export const timelineStore = internal;

function publish(timeline: Timeline): void {
  internal.set({
    timeline,
    canUndo: undoStack.length > 0,
    canRedo: redoStack.length > 0,
    selectedClipId,
  });
}

function findClipTrack(
  timeline: Timeline,
  clipId: string,
): { track: TimelineTrack; clip: VideoClip | AudioClip } | null {
  const v = timeline.video_track.find((c) => c.id === clipId);
  if (v) return { track: 'video', clip: v };
  const a = timeline.audio_track.find((c) => c.id === clipId);
  if (a) return { track: 'audio', clip: a };
  return null;
}

export const timelineActions = {
  reset(timeline?: Timeline): void {
    undoStack.length = 0;
    redoStack.length = 0;
    selectedClipId = null;
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

  selectClip(id: string | null): void {
    selectedClipId = id;
    internal.update((s) => ({ ...s, selectedClipId: id }));
  },

  async moveClip(
    track: TimelineTrack,
    clipId: string,
    newStartMs: number,
  ): Promise<void> {
    const startMs = Math.max(0, Math.round(newStartMs));
    const current = get(internal).timeline;
    try {
      const next = await ipc.timelineMoveClip(
        current,
        track,
        clipId,
        startMs,
        true,
        undefined,
      );
      timelineActions.apply(next);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('moveClip failed:', message);
    }
  },

  async trimClip(
    track: TimelineTrack,
    clipId: string,
    newSourceInMs: number,
    newSourceOutMs: number,
  ): Promise<void> {
    const inMs = Math.max(0, Math.round(newSourceInMs));
    const outMs = Math.max(0, Math.round(newSourceOutMs));
    if (outMs <= inMs) return;
    const current = get(internal).timeline;
    try {
      const next = await ipc.timelineTrimClip(
        current,
        track,
        clipId,
        inMs,
        outMs,
        true,
        undefined,
      );
      timelineActions.apply(next);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('trimClip failed:', message);
    }
  },

  async splitSelectedAt(playheadMs: number): Promise<void> {
    if (selectedClipId === null) return;
    const current = get(internal).timeline;
    const found = findClipTrack(current, selectedClipId);
    if (!found) return;
    const atMs = Math.max(0, Math.round(playheadMs));
    try {
      const next = await ipc.timelineSplitClip(
        current,
        found.track,
        selectedClipId,
        atMs,
      );
      timelineActions.apply(next);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('splitSelectedAt failed:', message);
    }
  },

  async deleteSelected(): Promise<void> {
    if (selectedClipId === null) return;
    const current = get(internal).timeline;
    const found = findClipTrack(current, selectedClipId);
    if (!found) return;
    const targetId = selectedClipId;
    try {
      const next = await ipc.timelineDeleteClip(current, found.track, targetId);
      selectedClipId = null;
      timelineActions.apply(next);
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error('deleteSelected failed:', message);
    }
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
