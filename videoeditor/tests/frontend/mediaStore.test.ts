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

import { mediaStore, mediaActions } from '$lib/stores/mediaStore';

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
});
