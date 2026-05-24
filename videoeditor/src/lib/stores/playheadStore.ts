import { writable, get } from 'svelte/store';
import type { Timeline, VideoClip, AudioClip } from '$lib/types';

export type ActiveClipKind = 'video' | 'audio';

export interface ActiveClip {
  kind: ActiveClipKind;
  clip: VideoClip | AudioClip;
}

export interface PlayheadState {
  playheadMs: number;
  playing: boolean;
}

const initial: PlayheadState = { playheadMs: 0, playing: false };

export const playheadStore = writable<PlayheadState>(initial);

let rafHandle: number | null = null;
let lastTickMs = 0;
let durationMsCap = 0;

function clamp(value: number, min: number, max: number): number {
  if (value < min) return min;
  if (value > max) return max;
  return value;
}

function containsMs(
  clip: VideoClip | AudioClip,
  ms: number,
): boolean {
  const length = clip.source_out_ms - clip.source_in_ms;
  const end = clip.timeline_start_ms + length;
  return ms >= clip.timeline_start_ms && ms < end;
}

export function activeClipAt(timeline: Timeline, ms: number): ActiveClip | null {
  const v = timeline.video_track.find((c) => containsMs(c, ms));
  if (v) return { kind: 'video', clip: v };
  const a = timeline.audio_track.find((c) => containsMs(c, ms));
  if (a) return { kind: 'audio', clip: a };
  return null;
}

function stopRaf(): void {
  if (rafHandle !== null) {
    cancelAnimationFrame(rafHandle);
    rafHandle = null;
  }
}

function tick(now: number): void {
  const delta = now - lastTickMs;
  lastTickMs = now;
  let stopped = false;
  playheadStore.update((s) => {
    if (!s.playing) return s;
    const next = s.playheadMs + delta;
    if (next >= durationMsCap) {
      stopped = true;
      return { playheadMs: durationMsCap, playing: false };
    }
    return { ...s, playheadMs: next };
  });
  if (stopped) {
    stopRaf();
    return;
  }
  rafHandle = requestAnimationFrame(tick);
}

export const playheadActions = {
  reset(): void {
    stopRaf();
    playheadStore.set(initial);
  },

  seek(ms: number, durationMs?: number): void {
    const max = durationMs ?? Number.POSITIVE_INFINITY;
    const clamped = clamp(ms, 0, max);
    playheadStore.update((s) => ({ ...s, playheadMs: clamped }));
  },

  play(durationMs: number): void {
    if (get(playheadStore).playing) return;
    if (durationMs <= 0) return;
    durationMsCap = durationMs;
    lastTickMs = performance.now();
    playheadStore.update((s) => {
      const start = s.playheadMs >= durationMs ? 0 : s.playheadMs;
      return { playheadMs: start, playing: true };
    });
    rafHandle = requestAnimationFrame(tick);
  },

  pause(): void {
    stopRaf();
    playheadStore.update((s) => (s.playing ? { ...s, playing: false } : s));
  },
};
