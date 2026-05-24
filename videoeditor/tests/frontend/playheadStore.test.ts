import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  playheadStore,
  playheadActions,
  activeClipAt,
} from '$lib/stores/playheadStore';
import type { Timeline, VideoClip, AudioClip } from '$lib/types';

function videoClip(
  id: string,
  startMs: number,
  durationMs: number,
  sourceInMs = 0,
): VideoClip {
  return {
    id,
    media_id: 'media-1',
    source_in_ms: sourceInMs,
    source_out_ms: sourceInMs + durationMs,
    timeline_start_ms: startMs,
    volume: 1,
    muted: false,
    transition_in: { type: 'cut', duration_ms: 0 },
    transition_out: { type: 'cut', duration_ms: 0 },
  };
}

function audioClip(id: string, startMs: number, durationMs: number): AudioClip {
  return {
    id,
    media_id: 'media-2',
    source_in_ms: 0,
    source_out_ms: durationMs,
    timeline_start_ms: startMs,
    volume: 1,
    fade_in_ms: 0,
    fade_out_ms: 0,
  };
}

function timelineWith(
  durationMs: number,
  video: VideoClip[] = [],
  audio: AudioClip[] = [],
): Timeline {
  return {
    duration_ms: durationMs,
    video_track: video,
    audio_track: audio,
    text_track: [],
  };
}

describe('activeClipAt', () => {
  it('returns null on empty timeline', () => {
    expect(activeClipAt(timelineWith(0), 0)).toBeNull();
  });

  it('returns the video clip whose range contains ms', () => {
    const c = videoClip('a', 1000, 2000);
    const result = activeClipAt(timelineWith(3000, [c]), 1500);
    expect(result?.clip.id).toBe('a');
    expect(result?.kind).toBe('video');
  });

  it('treats clip range as half-open [start, end)', () => {
    const c = videoClip('a', 1000, 1000);
    expect(activeClipAt(timelineWith(2000, [c]), 1000)?.clip.id).toBe('a');
    expect(activeClipAt(timelineWith(2000, [c]), 2000)).toBeNull();
  });

  it('returns null when ms falls in a gap between clips', () => {
    const a = videoClip('a', 0, 500);
    const b = videoClip('b', 1000, 500);
    expect(activeClipAt(timelineWith(2000, [a, b]), 700)).toBeNull();
  });

  it('prefers video over audio when both contain ms', () => {
    const v = videoClip('v', 0, 1000);
    const a = audioClip('a', 0, 1000);
    const r = activeClipAt(timelineWith(1000, [v], [a]), 500);
    expect(r?.kind).toBe('video');
    expect(r?.clip.id).toBe('v');
  });

  it('falls back to audio when only audio contains ms', () => {
    const a = audioClip('a', 0, 1000);
    const r = activeClipAt(timelineWith(1000, [], [a]), 500);
    expect(r?.kind).toBe('audio');
    expect(r?.clip.id).toBe('a');
  });
});

describe('playheadStore actions', () => {
  beforeEach(() => {
    playheadActions.pause();
    playheadActions.seek(0);
  });

  afterEach(() => {
    playheadActions.pause();
  });

  it('starts paused at 0', () => {
    const s = get(playheadStore);
    expect(s.playheadMs).toBe(0);
    expect(s.playing).toBe(false);
  });

  it('seek clamps to [0, duration_ms]', () => {
    playheadActions.seek(-100, 5000);
    expect(get(playheadStore).playheadMs).toBe(0);

    playheadActions.seek(10000, 5000);
    expect(get(playheadStore).playheadMs).toBe(5000);

    playheadActions.seek(2500, 5000);
    expect(get(playheadStore).playheadMs).toBe(2500);
  });

  it('seek without max keeps non-negative value', () => {
    playheadActions.seek(-50);
    expect(get(playheadStore).playheadMs).toBe(0);

    playheadActions.seek(1234);
    expect(get(playheadStore).playheadMs).toBe(1234);
  });

  it('play sets playing flag and pause clears it', () => {
    playheadActions.play(10000);
    expect(get(playheadStore).playing).toBe(true);
    playheadActions.pause();
    expect(get(playheadStore).playing).toBe(false);
  });

  it('play advances playheadMs by performance.now delta on each tick', () => {
    let now = 1000;
    const nowSpy = vi.spyOn(performance, 'now').mockImplementation(() => now);

    let pendingTick: FrameRequestCallback | null = null;
    const rafSpy = vi
      .spyOn(globalThis, 'requestAnimationFrame')
      .mockImplementation((cb: FrameRequestCallback) => {
        pendingTick = cb;
        return 1 as unknown as number;
      });
    const cancelSpy = vi
      .spyOn(globalThis, 'cancelAnimationFrame')
      .mockImplementation(() => {});

    try {
      playheadActions.seek(0);
      playheadActions.play(10000);
      expect(pendingTick).not.toBeNull();

      now = 1100;
      (pendingTick as FrameRequestCallback | null)?.(now);
      expect(get(playheadStore).playheadMs).toBe(100);

      now = 1250;
      (pendingTick as FrameRequestCallback | null)?.(now);
      expect(get(playheadStore).playheadMs).toBe(250);
    } finally {
      nowSpy.mockRestore();
      rafSpy.mockRestore();
      cancelSpy.mockRestore();
    }
  });

  it('play caps playheadMs at duration_ms and auto-pauses', () => {
    let now = 1000;
    const nowSpy = vi.spyOn(performance, 'now').mockImplementation(() => now);
    let pendingTick: FrameRequestCallback | null = null;
    const rafSpy = vi
      .spyOn(globalThis, 'requestAnimationFrame')
      .mockImplementation((cb: FrameRequestCallback) => {
        pendingTick = cb;
        return 1 as unknown as number;
      });
    const cancelSpy = vi
      .spyOn(globalThis, 'cancelAnimationFrame')
      .mockImplementation(() => {});

    try {
      playheadActions.seek(0);
      playheadActions.play(500);
      now = 2000;
      (pendingTick as FrameRequestCallback | null)?.(now);
      const s = get(playheadStore);
      expect(s.playheadMs).toBe(500);
      expect(s.playing).toBe(false);
    } finally {
      nowSpy.mockRestore();
      rafSpy.mockRestore();
      cancelSpy.mockRestore();
    }
  });

  it('play() while already playing is a no-op', () => {
    const rafSpy = vi
      .spyOn(globalThis, 'requestAnimationFrame')
      .mockImplementation(() => 1 as unknown as number);
    const cancelSpy = vi
      .spyOn(globalThis, 'cancelAnimationFrame')
      .mockImplementation(() => {});

    try {
      playheadActions.play(10000);
      const callsAfterFirst = rafSpy.mock.calls.length;
      playheadActions.play(10000);
      expect(rafSpy.mock.calls.length).toBe(callsAfterFirst);
    } finally {
      rafSpy.mockRestore();
      cancelSpy.mockRestore();
    }
  });
});
