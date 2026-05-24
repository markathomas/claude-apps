import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

import { projectStore, projectActions } from '$lib/stores/projectStore';
import { mediaStore, mediaActions } from '$lib/stores/mediaStore';
import {
  timelineStore,
  timelineActions,
  TIMELINE_HISTORY_LIMIT,
} from '$lib/stores/timelineStore';
import type { Timeline, MediaItem } from '$lib/types';

function emptyTimeline(): Timeline {
  return { duration_ms: 0, video_track: [], audio_track: [], text_track: [] };
}

function timelineWithDuration(ms: number): Timeline {
  return { duration_ms: ms, video_track: [], audio_track: [], text_track: [] };
}

const sampleProject = {
  version: '1',
  name: 'S',
  created_at: '',
  modified_at: '',
  output_settings: {
    resolution: { width: 1920, height: 1080 },
    framerate: 30,
    audio_sample_rate: 48000,
  },
  media_pool: [],
  timeline: emptyTimeline(),
};

describe('timelineStore', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    projectActions.reset();
    timelineActions.reset();
  });

  it('starts empty with no history', () => {
    const s = get(timelineStore);
    expect(s.timeline).toEqual(emptyTimeline());
    expect(s.canUndo).toBe(false);
    expect(s.canRedo).toBe(false);
  });

  it('reset(timeline) replaces current timeline and clears history', () => {
    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.reset(timelineWithDuration(5000));
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(5000);
    expect(s.canUndo).toBe(false);
    expect(s.canRedo).toBe(false);
  });

  it('apply pushes previous timeline to undo stack', () => {
    timelineActions.reset(emptyTimeline());
    timelineActions.apply(timelineWithDuration(1000));
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(1000);
    expect(s.canUndo).toBe(true);
    expect(s.canRedo).toBe(false);
  });

  it('undo restores the previous timeline and enables redo', () => {
    timelineActions.reset(emptyTimeline());
    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.apply(timelineWithDuration(2000));
    timelineActions.undo();
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(1000);
    expect(s.canUndo).toBe(true);
    expect(s.canRedo).toBe(true);
  });

  it('redo replays an undone change', () => {
    timelineActions.reset(emptyTimeline());
    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.apply(timelineWithDuration(2000));
    timelineActions.undo();
    timelineActions.redo();
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(2000);
    expect(s.canUndo).toBe(true);
    expect(s.canRedo).toBe(false);
  });

  it('apply after undo clears the redo stack', () => {
    timelineActions.reset(emptyTimeline());
    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.apply(timelineWithDuration(2000));
    timelineActions.undo();
    expect(get(timelineStore).canRedo).toBe(true);

    timelineActions.apply(timelineWithDuration(3000));
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(3000);
    expect(s.canRedo).toBe(false);
  });

  it('undo is a no-op when there is no history', () => {
    timelineActions.reset(timelineWithDuration(500));
    timelineActions.undo();
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(500);
    expect(s.canUndo).toBe(false);
  });

  it('redo is a no-op when there is nothing to redo', () => {
    timelineActions.reset(emptyTimeline());
    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.redo();
    const s = get(timelineStore);
    expect(s.timeline.duration_ms).toBe(1000);
    expect(s.canRedo).toBe(false);
  });

  it('caps undo history at TIMELINE_HISTORY_LIMIT entries', () => {
    timelineActions.reset(emptyTimeline());
    const total = TIMELINE_HISTORY_LIMIT + 25;
    for (let i = 1; i <= total; i++) {
      timelineActions.apply(timelineWithDuration(i));
    }

    let depth = 0;
    while (get(timelineStore).canUndo) {
      timelineActions.undo();
      depth++;
    }
    expect(depth).toBe(TIMELINE_HISTORY_LIMIT);
    // Earliest reachable timeline is the one at slot (total - LIMIT)
    expect(get(timelineStore).timeline.duration_ms).toBe(total - TIMELINE_HISTORY_LIMIT);
  });

  it('apply syncs the project timeline and marks the project dirty', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    timelineActions.reset(get(projectStore).project!.timeline);

    timelineActions.apply(timelineWithDuration(1234));

    const s = get(projectStore);
    expect(s.project?.timeline.duration_ms).toBe(1234);
    expect(s.dirty).toBe(true);
  });

  it('undo and redo also sync project.timeline', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    timelineActions.reset(get(projectStore).project!.timeline);

    timelineActions.apply(timelineWithDuration(1000));
    timelineActions.apply(timelineWithDuration(2000));
    timelineActions.undo();
    expect(get(projectStore).project?.timeline.duration_ms).toBe(1000);

    timelineActions.redo();
    expect(get(projectStore).project?.timeline.duration_ms).toBe(2000);
  });
});

describe('timelineActions.insertClipFromMedia', () => {
  const readyMedia: MediaItem = {
    id: 'media-1',
    source_path: '/x.mp4',
    proxy_path: '/proxies/x.mp4',
    proxy_status: 'ready',
    probe: {
      duration_ms: 4000,
      width: 1920,
      height: 1080,
      fps: 30,
      video_codec: 'h264',
      audio_codec: 'aac',
      has_audio: true,
    },
  };

  const insertedTimeline: Timeline = {
    duration_ms: 4000,
    video_track: [
      {
        id: 'clip-1',
        media_id: 'media-1',
        source_in_ms: 0,
        source_out_ms: 4000,
        timeline_start_ms: 1000,
        volume: 1,
        muted: false,
        transition_in: { type: 'cut', duration_ms: 0 },
        transition_out: { type: 'cut', duration_ms: 0 },
      },
    ],
    audio_track: [],
    text_track: [],
  };

  beforeEach(() => {
    mockInvoke.mockReset();
    projectActions.reset();
    timelineActions.reset();
    mediaActions.reset();
  });

  it('invokes timeline_insert_clip with [0, duration_ms] source range', async () => {
    mediaStore.update((s) => ({ ...s, items: [readyMedia] }));
    mockInvoke.mockResolvedValueOnce(insertedTimeline);

    await timelineActions.insertClipFromMedia('media-1', 1000);

    expect(mockInvoke).toHaveBeenCalledWith('timeline_insert_clip', {
      timeline: emptyTimeline(),
      track: 'video',
      mediaId: 'media-1',
      timelineStartMs: 1000,
      sourceInMs: 0,
      sourceOutMs: 4000,
    });
    expect(get(timelineStore).timeline).toEqual(insertedTimeline);
  });

  it('clamps drop position to >= 0', async () => {
    mediaStore.update((s) => ({ ...s, items: [readyMedia] }));
    mockInvoke.mockResolvedValueOnce(insertedTimeline);

    await timelineActions.insertClipFromMedia('media-1', -500);

    expect(mockInvoke).toHaveBeenCalledWith(
      'timeline_insert_clip',
      expect.objectContaining({ timelineStartMs: 0 }),
    );
  });

  it('is a no-op for unknown media id', async () => {
    await timelineActions.insertClipFromMedia('missing', 0);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('is a no-op when media is not ready', async () => {
    mediaStore.update((s) => ({
      ...s,
      items: [{ ...readyMedia, proxy_status: 'generating' }],
    }));
    await timelineActions.insertClipFromMedia('media-1', 0);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('is a no-op when probe is missing', async () => {
    mediaStore.update((s) => ({
      ...s,
      items: [{ ...readyMedia, probe: null }],
    }));
    await timelineActions.insertClipFromMedia('media-1', 0);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

describe('projectActions.setTimeline', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    projectActions.reset();
  });

  it('replaces the project timeline and marks dirty', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    expect(get(projectStore).dirty).toBe(false);

    projectActions.setTimeline(timelineWithDuration(7777));

    const s = get(projectStore);
    expect(s.project?.timeline.duration_ms).toBe(7777);
    expect(s.dirty).toBe(true);
  });

  it('is a no-op when no project is loaded', () => {
    projectActions.setTimeline(timelineWithDuration(1));
    const s = get(projectStore);
    expect(s.project).toBeNull();
    expect(s.dirty).toBe(false);
  });
});
