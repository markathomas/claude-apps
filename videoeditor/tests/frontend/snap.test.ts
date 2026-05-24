import { describe, it, expect } from 'vitest';
import { snap, type SnapEdge } from '$lib/lib/snap';

describe('snap', () => {
  it('snaps to nearest start edge within threshold', () => {
    const edges: SnapEdge[] = [
      { ms: 1000, source: 'start' },
      { ms: 5000, source: 'end' },
    ];
    const result = snap(1050, edges, 100);
    expect(result.snapped).toBe(1000);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('start');
  });

  it('snaps to nearest end edge within threshold', () => {
    const edges: SnapEdge[] = [
      { ms: 1000, source: 'start' },
      { ms: 5000, source: 'end' },
    ];
    const result = snap(4970, edges, 100);
    expect(result.snapped).toBe(5000);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('end');
  });

  it('snaps to zero when candidate is within threshold of 0', () => {
    const edges: SnapEdge[] = [{ ms: 5000, source: 'end' }];
    const result = snap(50, edges, 100);
    expect(result.snapped).toBe(0);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('zero');
  });

  it('returns candidate unchanged when nothing within threshold', () => {
    const edges: SnapEdge[] = [
      { ms: 1000, source: 'start' },
      { ms: 5000, source: 'end' },
    ];
    const result = snap(3000, edges, 100);
    expect(result.snapped).toBe(3000);
    expect(result.hit).toBe(false);
    expect(result.source).toBe(null);
  });

  it('returns candidate unchanged with empty edge list when far from zero', () => {
    const result = snap(2000, [], 100);
    expect(result.snapped).toBe(2000);
    expect(result.hit).toBe(false);
    expect(result.source).toBe(null);
  });

  it('snaps to zero with empty edge list when within threshold of 0', () => {
    const result = snap(80, [], 100);
    expect(result.snapped).toBe(0);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('zero');
  });

  it('picks the closest edge when multiple are within threshold', () => {
    const edges: SnapEdge[] = [
      { ms: 1000, source: 'start' },
      { ms: 1080, source: 'end' },
    ];
    const result = snap(1050, edges, 100);
    expect(result.snapped).toBe(1080);
    expect(result.source).toBe('end');
  });

  it('treats threshold as inclusive at exactly threshold distance', () => {
    const edges: SnapEdge[] = [{ ms: 1000, source: 'start' }];
    const result = snap(1100, edges, 100);
    expect(result.snapped).toBe(1000);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('start');
  });

  it('does not snap when distance exceeds threshold', () => {
    const edges: SnapEdge[] = [{ ms: 1000, source: 'start' }];
    const result = snap(1101, edges, 100);
    expect(result.snapped).toBe(1101);
    expect(result.hit).toBe(false);
    expect(result.source).toBe(null);
  });

  it('snaps to zero when zero is closer than any edge', () => {
    const edges: SnapEdge[] = [{ ms: 90, source: 'start' }];
    const result = snap(40, edges, 100);
    expect(result.snapped).toBe(0);
    expect(result.source).toBe('zero');
  });

  it('snaps to edge when edge is closer than zero', () => {
    const edges: SnapEdge[] = [{ ms: 90, source: 'start' }];
    const result = snap(80, edges, 100);
    expect(result.snapped).toBe(90);
    expect(result.source).toBe('start');
  });

  it('handles negative candidate by snapping toward zero when within threshold', () => {
    const edges: SnapEdge[] = [{ ms: 1000, source: 'start' }];
    const result = snap(-50, edges, 100);
    expect(result.snapped).toBe(0);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('zero');
  });

  it('handles candidate exactly on an edge', () => {
    const edges: SnapEdge[] = [{ ms: 2000, source: 'start' }];
    const result = snap(2000, edges, 100);
    expect(result.snapped).toBe(2000);
    expect(result.hit).toBe(true);
    expect(result.source).toBe('start');
  });
});
