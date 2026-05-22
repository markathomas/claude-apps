import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

import { ipc } from '$lib/ipc';

describe('ipc', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it('newProject calls new_project with name', async () => {
    mockInvoke.mockResolvedValueOnce({
      version: '1', name: 'Hi', created_at: '', modified_at: '',
      output_settings: { resolution: { width: 1920, height: 1080 }, framerate: 30, audio_sample_rate: 48000 },
      media_pool: [], timeline: { duration_ms: 0, video_track: [], audio_track: [], text_track: [] },
    });
    const p = await ipc.newProject('Hi');
    expect(mockInvoke).toHaveBeenCalledWith('new_project', { name: 'Hi' });
    expect(p.name).toBe('Hi');
  });

  it('openProject calls open_project with path', async () => {
    mockInvoke.mockResolvedValueOnce({
      version: '1', name: 'X', created_at: '', modified_at: '',
      output_settings: { resolution: { width: 1920, height: 1080 }, framerate: 30, audio_sample_rate: 48000 },
      media_pool: [], timeline: { duration_ms: 0, video_track: [], audio_track: [], text_track: [] },
    });
    await ipc.openProject('/abs/p.vproj');
    expect(mockInvoke).toHaveBeenCalledWith('open_project', { path: '/abs/p.vproj' });
  });

  it('saveProject calls save_project_cmd with project and path', async () => {
    mockInvoke.mockResolvedValueOnce(null);
    const project = {
      version: '1', name: 'Y', created_at: '', modified_at: '',
      output_settings: { resolution: { width: 1920, height: 1080 }, framerate: 30, audio_sample_rate: 48000 },
      media_pool: [], timeline: { duration_ms: 0, video_track: [], audio_track: [], text_track: [] },
    };
    await ipc.saveProject(project, '/abs/p.vproj');
    expect(mockInvoke).toHaveBeenCalledWith('save_project_cmd', { project, path: '/abs/p.vproj' });
  });

  it('getRecentProjects calls get_recent_projects', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const r = await ipc.getRecentProjects();
    expect(mockInvoke).toHaveBeenCalledWith('get_recent_projects', undefined);
    expect(r).toEqual([]);
  });
});
