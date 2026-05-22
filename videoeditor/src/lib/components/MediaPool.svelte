<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { mediaStore, mediaActions } from '$lib/stores/mediaStore';

  async function handleImport() {
    const picked = await openDialog({
      multiple: true,
      filters: [{ name: 'Video', extensions: ['mp4', 'mov', 'mkv', 'webm', 'm4v'] }],
    });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    if (paths.length === 0) return;
    await mediaActions.importMedia(paths);
  }

  async function handleDelete(id: string) {
    await mediaActions.deleteMedia(id);
  }

  function basename(p: string): string {
    const i = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
    return i >= 0 ? p.slice(i + 1) : p;
  }

  function formatDuration(ms: number | undefined): string {
    if (!ms) return '--:--';
    const total = Math.round(ms / 1000);
    const m = Math.floor(total / 60);
    const s = total % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<aside class="pane media-pool">
  <header>
    <h2>Media</h2>
    <button type="button" onclick={handleImport}>+ Import</button>
  </header>

  {#if $mediaStore.items.length === 0}
    <p class="placeholder">No media imported yet.</p>
  {:else}
    <ul>
      {#each $mediaStore.items as item (item.id)}
        <li class="item" data-status={item.proxy_status}>
          <div class="row">
            <span class="name" title={item.source_path}>{basename(item.source_path)}</span>
            <button type="button" class="delete" onclick={() => handleDelete(item.id)} aria-label="Remove">×</button>
          </div>
          <div class="meta">
            {#if item.probe}
              <span>{item.probe.width}×{item.probe.height}</span>
              <span>{formatDuration(item.probe.duration_ms)}</span>
              {#if !item.probe.has_audio}<span class="muted-tag">no audio</span>{/if}
            {/if}
          </div>
          <div class="status">
            {#if item.proxy_status === 'pending'}
              <span class="badge pending">queued</span>
            {:else if item.proxy_status === 'generating'}
              <span class="badge generating">proxy {Math.round(item.progress ?? 0)}%</span>
            {:else if item.proxy_status === 'ready'}
              <span class="badge ready">ready</span>
            {:else if item.proxy_status === 'failed'}
              <span class="badge failed" title={item.error ?? ''}>failed</span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .pane { padding: 0.5rem 0.75rem; overflow: auto; }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
  h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.7; margin: 0; }
  header button {
    background: #2563eb; color: white; border: 0; padding: 0.3rem 0.6rem;
    border-radius: 4px; font-size: 0.8rem; cursor: pointer;
  }
  header button:hover { background: #1d4ed8; }
  .placeholder { font-size: 0.875rem; opacity: 0.6; }
  ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .item {
    background: #1f1f1f; padding: 0.5rem; border-radius: 4px;
    border-left: 3px solid #555;
  }
  .item[data-status="ready"] { border-left-color: #22c55e; }
  .item[data-status="failed"] { border-left-color: #ef4444; }
  .item[data-status="generating"] { border-left-color: #2563eb; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
  .name { font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .delete {
    background: transparent; border: 0; color: inherit; opacity: 0.5;
    font-size: 1rem; line-height: 1; cursor: pointer; padding: 0 0.25rem;
  }
  .delete:hover { opacity: 1; }
  .meta { display: flex; gap: 0.5rem; font-size: 0.75rem; opacity: 0.6; margin-top: 0.25rem; }
  .muted-tag { color: #f59e0b; }
  .status { margin-top: 0.4rem; }
  .badge {
    display: inline-block; font-size: 0.7rem; padding: 0.1rem 0.4rem;
    border-radius: 2px; background: #2a2a2a;
  }
  .badge.ready { background: #15803d; }
  .badge.failed { background: #991b1b; }
  .badge.generating { background: #1d4ed8; }
</style>
