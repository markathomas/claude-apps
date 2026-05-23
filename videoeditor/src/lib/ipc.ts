import { invoke } from '@tauri-apps/api/core';
import type { MediaItem, Project, RecentProject, Timeline } from './types';

export type TimelineTrack = 'video' | 'audio';

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
  timelineInsertClip(
    timeline: Timeline,
    track: TimelineTrack,
    mediaId: string,
    timelineStartMs: number,
    sourceInMs: number,
    sourceOutMs: number,
  ): Promise<Timeline> {
    return invoke('timeline_insert_clip', {
      timeline,
      track,
      mediaId,
      timelineStartMs,
      sourceInMs,
      sourceOutMs,
    });
  },
  timelineMoveClip(
    timeline: Timeline,
    track: TimelineTrack,
    clipId: string,
    newStartMs: number,
    snapEnabled: boolean,
    snapThresholdMs?: number,
  ): Promise<Timeline> {
    return invoke('timeline_move_clip', {
      timeline,
      track,
      clipId,
      newStartMs,
      snapEnabled,
      snapThresholdMs,
    });
  },
  timelineTrimClip(
    timeline: Timeline,
    track: TimelineTrack,
    clipId: string,
    newSourceInMs: number,
    newSourceOutMs: number,
    snapEnabled: boolean,
    snapThresholdMs?: number,
  ): Promise<Timeline> {
    return invoke('timeline_trim_clip', {
      timeline,
      track,
      clipId,
      newSourceInMs,
      newSourceOutMs,
      snapEnabled,
      snapThresholdMs,
    });
  },
  timelineSplitClip(
    timeline: Timeline,
    track: TimelineTrack,
    clipId: string,
    atTimelineMs: number,
  ): Promise<Timeline> {
    return invoke('timeline_split_clip', {
      timeline,
      track,
      clipId,
      atTimelineMs,
    });
  },
  timelineDeleteClip(
    timeline: Timeline,
    track: TimelineTrack,
    clipId: string,
  ): Promise<Timeline> {
    return invoke('timeline_delete_clip', { timeline, track, clipId });
  },
};
