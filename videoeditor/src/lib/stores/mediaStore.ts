import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { ipc } from '$lib/ipc';
import type { MediaItem } from '$lib/types';

export interface MediaItemView extends MediaItem {
  progress?: number;
  thumbnails_dir?: string;
  waveform_path?: string;
  error?: string;
}

interface MediaState {
  items: MediaItemView[];
  initialized: boolean;
}

const initial: MediaState = { items: [], initialized: false };

export const mediaStore = writable<MediaState>(initial);

interface ProxyProgressPayload { media_id: string; percent: number }
interface ProxyReadyPayload {
  media_id: string;
  proxy_path: string;
  thumbnails_dir: string;
  waveform_path: string;
}
interface ProxyFailedPayload { media_id: string; reason: string }

function update(id: string, patch: Partial<MediaItemView>) {
  mediaStore.update((s) => ({
    ...s,
    items: s.items.map((i) => (i.id === id ? { ...i, ...patch } : i)),
  }));
}

export const mediaActions = {
  reset(): void {
    mediaStore.set(initial);
  },

  async initialize(): Promise<void> {
    if (get(mediaStore).initialized) return;

    await listen<ProxyProgressPayload>('proxy_progress', (e) => {
      update(e.payload.media_id, { progress: e.payload.percent });
    });
    await listen<ProxyReadyPayload>('proxy_ready', (e) => {
      update(e.payload.media_id, {
        proxy_status: 'ready',
        proxy_path: e.payload.proxy_path,
        thumbnails_dir: e.payload.thumbnails_dir,
        waveform_path: e.payload.waveform_path,
        progress: 100,
      });
    });
    await listen<ProxyFailedPayload>('proxy_failed', (e) => {
      update(e.payload.media_id, {
        proxy_status: 'failed',
        error: e.payload.reason,
      });
    });

    const items = await ipc.listMedia();
    mediaStore.set({ items, initialized: true });
  },

  async importMedia(paths: string[]): Promise<void> {
    const newItems = await ipc.importMedia(paths);
    mediaStore.update((s) => ({ ...s, items: [...s.items, ...newItems] }));
  },

  async deleteMedia(id: string): Promise<void> {
    await ipc.deleteMedia(id);
    mediaStore.update((s) => ({ ...s, items: s.items.filter((i) => i.id !== id) }));
  },
};
