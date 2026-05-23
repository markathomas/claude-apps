import { writable, derived, get, type Readable } from 'svelte/store';
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
  selectedId: string | null;
}

const initial: MediaState = { items: [], initialized: false, selectedId: null };

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

export interface PreviewSource {
  id: string;
  proxyPath: string;
}

function samePreview(a: PreviewSource | null, b: PreviewSource | null): boolean {
  if (a === null || b === null) return a === b;
  return a.id === b.id && a.proxyPath === b.proxyPath;
}

// Emits only when the selected clip's ready proxy path changes.
// Importing or generating proxies for other clips must not change this value
// (otherwise the preview restarts mid-playback).
export const selectedPreviewSource: Readable<PreviewSource | null> = (() => {
  let last: PreviewSource | null = null;
  return derived<typeof mediaStore, PreviewSource | null>(
    mediaStore,
    ($s, set) => {
      let next: PreviewSource | null = null;
      if ($s.selectedId) {
        const item = $s.items.find((i) => i.id === $s.selectedId);
        if (item && item.proxy_status === 'ready' && item.proxy_path) {
          next = { id: item.id, proxyPath: item.proxy_path };
        }
      }
      if (!samePreview(last, next)) {
        last = next;
        set(next);
      }
    },
    null,
  );
})();

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
    mediaStore.update((s) => ({ ...s, items, initialized: true }));
  },

  async importMedia(paths: string[]): Promise<void> {
    const newItems = await ipc.importMedia(paths);
    mediaStore.update((s) => ({ ...s, items: [...s.items, ...newItems] }));
  },

  selectItem(id: string | null): void {
    mediaStore.update((s) => ({ ...s, selectedId: id }));
  },

  async deleteMedia(id: string): Promise<void> {
    await ipc.deleteMedia(id);
    mediaStore.update((s) => ({
      ...s,
      items: s.items.filter((i) => i.id !== id),
      selectedId: s.selectedId === id ? null : s.selectedId,
    }));
  },
};
