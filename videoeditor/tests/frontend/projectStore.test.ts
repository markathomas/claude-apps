import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

import { projectStore, projectActions } from '$lib/stores/projectStore';

const sampleProject = {
  version: '1',
  name: 'S',
  created_at: '',
  modified_at: '',
  output_settings: { resolution: { width: 1920, height: 1080 }, framerate: 30, audio_sample_rate: 48000 },
  media_pool: [],
  timeline: { duration_ms: 0, video_track: [], audio_track: [], text_track: [] },
};

describe('projectStore', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    projectActions.reset();
  });

  it('starts empty', () => {
    expect(get(projectStore)).toEqual({ project: null, path: null, dirty: false });
  });

  it('newProject populates the store and clears path', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    const s = get(projectStore);
    expect(s.project?.name).toBe('S');
    expect(s.path).toBeNull();
    expect(s.dirty).toBe(false);
  });

  it('openProject sets project and path', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.openProject('/abs/p.vproj');
    const s = get(projectStore);
    expect(s.project).not.toBeNull();
    expect(s.path).toBe('/abs/p.vproj');
    expect(s.dirty).toBe(false);
  });

  it('save writes via ipc and clears dirty', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    projectActions.markDirty();
    expect(get(projectStore).dirty).toBe(true);

    mockInvoke.mockResolvedValueOnce(null);
    await projectActions.save('/abs/new.vproj');
    expect(mockInvoke).toHaveBeenCalledWith('save_project_cmd', expect.objectContaining({
      path: '/abs/new.vproj',
    }));
    const s = get(projectStore);
    expect(s.path).toBe('/abs/new.vproj');
    expect(s.dirty).toBe(false);
  });

  it('save throws when no project loaded', async () => {
    await expect(projectActions.save('/abs/p.vproj')).rejects.toThrow(/no project/i);
  });

  it('save without path throws when project has no associated path', async () => {
    mockInvoke.mockResolvedValueOnce(sampleProject);
    await projectActions.newProject('S');
    await expect(projectActions.save()).rejects.toThrow(/path/i);
  });
});
