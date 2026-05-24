import { writable, get } from 'svelte/store';
import { ipc } from '$lib/ipc';
import type { Project, Timeline } from '$lib/types';

export interface ProjectState {
  project: Project | null;
  path: string | null;
  dirty: boolean;
}

const initial: ProjectState = { project: null, path: null, dirty: false };

export const projectStore = writable<ProjectState>(initial);

export const projectActions = {
  reset(): void {
    projectStore.set(initial);
  },

  async newProject(name: string): Promise<void> {
    const project = await ipc.newProject(name);
    projectStore.set({ project, path: null, dirty: false });
  },

  async openProject(path: string): Promise<void> {
    const project = await ipc.openProject(path);
    projectStore.set({ project, path, dirty: false });
  },

  async save(targetPath?: string): Promise<void> {
    const state = get(projectStore);
    if (!state.project) {
      throw new Error('no project loaded');
    }
    const path = targetPath ?? state.path;
    if (!path) {
      throw new Error('no path provided and project has no associated path');
    }
    await ipc.saveProject(state.project, path);
    projectStore.update((s) => ({ ...s, path, dirty: false }));
  },

  markDirty(): void {
    projectStore.update((s) => ({ ...s, dirty: true }));
  },

  setProject(project: Project): void {
    projectStore.update((s) => ({ ...s, project, dirty: true }));
  },

  setTimeline(timeline: Timeline): void {
    projectStore.update((s) =>
      s.project ? { ...s, project: { ...s.project, timeline }, dirty: true } : s,
    );
  },
};
