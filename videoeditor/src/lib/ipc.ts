import { invoke } from '@tauri-apps/api/core';
import type { MediaItem, Project, RecentProject } from './types';

export const ipc = {
  newProject(name: string): Promise<Project> {
    return invoke('new_project', { name });
  },
  openProject(path: string): Promise<Project> {
    return invoke('open_project', { path });
  },
  saveProject(project: Project, path: string): Promise<void> {
    return invoke('save_project_cmd', { project, path });
  },
  getRecentProjects(): Promise<RecentProject[]> {
    return invoke('get_recent_projects', undefined);
  },
  importMedia(paths: string[]): Promise<MediaItem[]> {
    return invoke('import_media', { paths });
  },
  deleteMedia(id: string): Promise<void> {
    return invoke('delete_media', { id });
  },
  listMedia(): Promise<MediaItem[]> {
    return invoke('list_media', undefined);
  },
};
