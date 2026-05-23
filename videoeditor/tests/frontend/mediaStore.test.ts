import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const mockInvoke = vi.fn();
const mockListen = vi.fn();
const eventCallbacks: Record<string, Array<(payload: unknown) => void>> = {};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (name: string, cb: (e: { payload: unknown }) => void) => {
    eventCallbacks[name] ??= [];
    eventCallbacks[name].push((p) => cb({ payload: p }));
    mockListen(name);
    return Promise.resolve(() => {});
  },
}));

import { mediaStore, mediaActions, selectedPreviewSource } from '$lib/stores/mediaStore';

const mediaItem = {
  id: 'm1',
  source_path: '/a.mp4',
  proxy_path: null,
  proxy_status: 'pending' as const,
  probe: null,
};

function fireEvent(name: string, payload: unknown) {
  for (const cb of eventCallbacks[name] ?? []) cb(payload);
}

describe('mediaStore', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockListen.mockReset();
    for (const k of Object.keys(eventCallbacks)) delete eventCallbacks[k];
    mediaActions.reset();
  });

  it('starts empty', () => {
    expect(get(mediaStore).items).toEqual([]);
  });

  it('initialize subscribes to events and loads list', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    expect(mockListen).toHaveBeenCalledWith('proxy_progress');
    expect(mockListen).toHaveBeenCalledWith('proxy_ready');
    expect(mockListen).toHaveBeenCalledWith('proxy_failed');
    expect(get(mediaStore).items).toHaveLength(1);
  });

  it('importMedia appends items', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.importMedia(['/a.mp4']);
    expect(mockInvoke).toHaveBeenCalledWith('import_media', { paths: ['/a.mp4'] });
    expect(get(mediaStore).items).toEqual([mediaItem]);
  });

  it('proxy_progress updates progress for matching item', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_progress', { media_id: 'm1', percent: 42 });
    const item = get(mediaStore).items[0];
    expect(item.progress).toBe(42);
  });

  it('proxy_ready updates status to ready and stores paths', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_ready', {
      media_id: 'm1',
      proxy_path: '/p.mp4',
      thumbnails_dir: '/t',
      waveform_path: '/w.json',
    });
    const item = get(mediaStore).items[0];
    expect(item.proxy_status).toBe('ready');
    expect(item.proxy_path).toBe('/p.mp4');
    expect(item.thumbnails_dir).toBe('/t');
    expect(item.waveform_path).toBe('/w.json');
  });

  it('proxy_failed updates status to failed', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_failed', { media_id: 'm1', reason: 'boom' });
    const item = get(mediaStore).items[0];
    expect(item.proxy_status).toBe('failed');
    expect(item.error).toBe('boom');
  });

  it('deleteMedia removes from store', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mockInvoke.mockResolvedValueOnce(null);
    await mediaActions.deleteMedia('m1');
    expect(mockInvoke).toHaveBeenCalledWith('delete_media', { id: 'm1' });
    expect(get(mediaStore).items).toEqual([]);
  });

  it('selectItem sets selectedId', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    expect(get(mediaStore).selectedId).toBe('m1');
  });

  it('selectItem can be cleared to null', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    mediaActions.selectItem(null);
    expect(get(mediaStore).selectedId).toBeNull();
  });

  it('deleteMedia clears selectedId when the selected item is deleted', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    mockInvoke.mockResolvedValueOnce(null);
    await mediaActions.deleteMedia('m1');
    expect(get(mediaStore).selectedId).toBeNull();
  });

  it('deleteMedia preserves selectedId when a different item is deleted', async () => {
    const other = { ...mediaItem, id: 'm2', source_path: '/b.mp4' };
    mockInvoke.mockResolvedValueOnce([mediaItem, other]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    mockInvoke.mockResolvedValueOnce(null);
    await mediaActions.deleteMedia('m2');
    expect(get(mediaStore).selectedId).toBe('m1');
  });
});

describe('selectedPreviewSource', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockListen.mockReset();
    for (const k of Object.keys(eventCallbacks)) delete eventCallbacks[k];
    mediaActions.reset();
  });

  const ready = (id: string, proxyPath: string) => ({
    id,
    source_path: `/${id}.mp4`,
    proxy_path: proxyPath,
    proxy_status: 'ready' as const,
    probe: null,
  });

  it('is null when no item is selected', () => {
    expect(get(selectedPreviewSource)).toBeNull();
  });

  it('is null when the selected item has no ready proxy', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    expect(get(selectedPreviewSource)).toBeNull();
  });

  it('emits the proxy path once the selected item becomes ready', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');
    fireEvent('proxy_ready', {
      media_id: 'm1',
      proxy_path: '/p1.webm',
      thumbnails_dir: '/t',
      waveform_path: '/w.json',
    });
    expect(get(selectedPreviewSource)).toEqual({ id: 'm1', proxyPath: '/p1.webm' });
  });

  it('does not re-emit when an unrelated item is imported while playing', async () => {
    mockInvoke.mockResolvedValueOnce([ready('m1', '/p1.webm')]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');

    const seen: Array<unknown> = [];
    const unsub = selectedPreviewSource.subscribe((v) => seen.push(v));
    expect(seen).toHaveLength(1);
    expect(seen[0]).toEqual({ id: 'm1', proxyPath: '/p1.webm' });

    // Import a new item — should not affect the playing preview source.
    mockInvoke.mockResolvedValueOnce([
      { ...mediaItem, id: 'm2', source_path: '/b.mp4' },
    ]);
    await mediaActions.importMedia(['/b.mp4']);

    // Progress events for the new item must not change the preview source.
    fireEvent('proxy_progress', { media_id: 'm2', percent: 25 });
    fireEvent('proxy_progress', { media_id: 'm2', percent: 75 });
    fireEvent('proxy_ready', {
      media_id: 'm2',
      proxy_path: '/p2.webm',
      thumbnails_dir: '/t',
      waveform_path: '/w.json',
    });

    expect(seen).toHaveLength(1);
    unsub();
  });

  it('does not re-emit when the same selected item gets a progress update', async () => {
    mockInvoke.mockResolvedValueOnce([ready('m1', '/p1.webm')]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');

    const seen: Array<unknown> = [];
    const unsub = selectedPreviewSource.subscribe((v) => seen.push(v));
    expect(seen).toHaveLength(1);

    fireEvent('proxy_progress', { media_id: 'm1', percent: 99 });
    expect(seen).toHaveLength(1);
    unsub();
  });

  it('emits when the user selects a different ready clip', async () => {
    mockInvoke.mockResolvedValueOnce([
      ready('m1', '/p1.webm'),
      ready('m2', '/p2.webm'),
    ]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');

    const seen: Array<unknown> = [];
    const unsub = selectedPreviewSource.subscribe((v) => seen.push(v));
    expect(seen).toHaveLength(1);
    expect(seen[0]).toEqual({ id: 'm1', proxyPath: '/p1.webm' });

    mediaActions.selectItem('m2');
    expect(seen).toHaveLength(2);
    expect(seen[1]).toEqual({ id: 'm2', proxyPath: '/p2.webm' });
    unsub();
  });

  it('emits null when the selection is cleared', async () => {
    mockInvoke.mockResolvedValueOnce([ready('m1', '/p1.webm')]);
    await mediaActions.initialize();
    mediaActions.selectItem('m1');

    const seen: Array<unknown> = [];
    const unsub = selectedPreviewSource.subscribe((v) => seen.push(v));
    mediaActions.selectItem(null);
    expect(seen[seen.length - 1]).toBeNull();
    unsub();
  });
});
