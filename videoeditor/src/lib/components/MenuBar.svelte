<script lang="ts">
  import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { projectStore, projectActions } from '$lib/stores/projectStore';
  import { ipc } from '$lib/ipc';
  import type { RecentProject } from '$lib/types';
  import NewProjectDialog from '$lib/dialogs/NewProjectDialog.svelte';
  import RecentProjectsDialog from '$lib/dialogs/RecentProjectsDialog.svelte';

  let showNewDialog = $state(false);
  let showRecentDialog = $state(false);
  let recent: RecentProject[] = $state([]);

  async function handleNewClick() {
    showNewDialog = true;
  }

  async function handleNewCreate(name: string) {
    showNewDialog = false;
    await projectActions.newProject(name);
  }

  async function handleOpen() {
    const picked = await openDialog({
      multiple: false,
      filters: [{ name: 'VideoEditor Project', extensions: ['vproj'] }],
    });
    if (typeof picked !== 'string') return;
    await projectActions.openProject(picked);
  }

  async function handleSave() {
    const state = $projectStore;
    if (!state.project) return;
    if (state.path) {
      await projectActions.save();
      return;
    }
    await handleSaveAs();
  }

  async function handleSaveAs() {
    const state = $projectStore;
    if (!state.project) return;
    const picked = await saveDialog({
      defaultPath: `${state.project.name}.vproj`,
      filters: [{ name: 'VideoEditor Project', extensions: ['vproj'] }],
    });
    if (typeof picked !== 'string') return;
    await projectActions.save(picked);
  }

  async function handleRecentClick() {
    recent = await ipc.getRecentProjects();
    showRecentDialog = true;
  }

  async function handleRecentPick(path: string) {
    showRecentDialog = false;
    await projectActions.openProject(path);
  }
</script>

<nav class="menubar" aria-label="Main menu">
  <div class="group">
    <span class="label">File</span>
    <button type="button" onclick={handleNewClick}>New</button>
    <button type="button" onclick={handleOpen}>Open…</button>
    <button type="button" onclick={handleRecentClick}>Recent…</button>
    <button type="button" onclick={handleSave} disabled={!$projectStore.project}>Save</button>
    <button type="button" onclick={handleSaveAs} disabled={!$projectStore.project}>Save As…</button>
  </div>
  <div class="status">
    {#if $projectStore.project}
      <span>{$projectStore.project.name}{$projectStore.dirty ? ' •' : ''}</span>
    {:else}
      <span class="hint">No project</span>
    {/if}
  </div>
</nav>

<NewProjectDialog
  open={showNewDialog}
  onCreate={handleNewCreate}
  onCancel={() => (showNewDialog = false)}
/>
<RecentProjectsDialog
  open={showRecentDialog}
  items={recent}
  onPick={handleRecentPick}
  onCancel={() => (showRecentDialog = false)}
/>

<style>
  .menubar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #161616;
    border-bottom: 1px solid #2a2a2a;
    padding: 0.25rem 0.5rem;
    height: 36px;
  }
  .group { display: flex; align-items: center; gap: 0.25rem; }
  .label { font-size: 0.75rem; opacity: 0.6; padding-right: 0.5rem; text-transform: uppercase; letter-spacing: 0.05em; }
  button {
    background: transparent;
    border: 0;
    color: inherit;
    padding: 0.3rem 0.6rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
  }
  button:hover:not(:disabled) { background: #2a2a2a; }
  button:disabled { opacity: 0.4; cursor: not-allowed; }
  .status { font-size: 0.85rem; opacity: 0.8; }
  .hint { opacity: 0.5; }
</style>
