# VideoEditor — Design Spec

**Date:** 2026-05-22
**Status:** Approved (design phase)
**App name:** VideoEditor

## 1. Purpose

A Linux desktop video editor scoped for **quick clip work**: trim, splice, lightly enhance, and export a handful of clips end-to-end. The editor is for the user's own use — mixed source material (screen recordings, phone video, camera footage), short-to-medium projects (2–10 clips on a single timeline). It is not a general-purpose NLE; multi-track video, color grading, keyframes, and effects beyond simple transitions and text are explicit non-goals for v1.

## 2. Tech stack

- **Shell:** Tauri 2 (Rust backend, system webview frontend)
- **Frontend:** Svelte (with TypeScript and Vite)
- **Backend language:** Rust
- **Video engine:** FFmpeg as a bundled external binary, invoked as a subprocess; `ffprobe` for media inspection
- **Packaging:** AppImage primary; `.deb` and Flatpak as follow-ups
- **Testing:** `cargo test` (Rust), Vitest (frontend logic), Playwright (E2E against built binary)

Rationale: Tauri keeps the binary lean and gives Rust ownership of disk, processes, and large data. FFmpeg-as-subprocess is more than fast enough for the target workload (few clips, single video track) and avoids linking against `libav*` directly.

## 3. Architecture

Three-layer split:

1. **Rust core (Tauri backend)** — source of truth. Owns project file IO, FFmpeg orchestration, proxy generation, export pipeline, media probing.
2. **Frontend (Tauri webview)** — editor surface. Owns timeline UI, playback, edit operations on a local copy of project state, sync to Rust on commit.
3. **FFmpeg (bundled binary)** — does all real video work.

Communication between frontend and Rust is via Tauri commands (request/response) and Tauri events (push notifications for progress and async results).

### Tauri commands (initial set)

| Command | Args | Returns |
|---|---|---|
| `new_project` | `name`, `output_settings` | `Project` |
| `open_project` | `path` | `Project` |
| `save_project` | `project`, `path?` | `()` |
| `import_media` | `paths: string[]` | `MediaItem[]` (with `proxy_status: pending`) |
| `delete_media` | `media_id` | `()` |
| `start_export` | `project`, `output_path`, `preset` | `export_id` |
| `cancel_export` | `export_id` | `()` |
| `get_recent_projects` | — | `RecentProject[]` |

### Tauri events

| Event | Payload |
|---|---|
| `proxy_progress` | `{ media_id, percent }` |
| `proxy_ready` | `{ media_id, proxy_path, thumbnails_dir, waveform_path }` |
| `proxy_failed` | `{ media_id, reason }` |
| `export_progress` | `{ export_id, percent, eta_seconds }` |
| `export_complete` | `{ export_id, output_path }` |
| `export_failed` | `{ export_id, reason, stderr_tail }` |
| `autosave_written` | `{ path, timestamp }` |

## 4. Data model

Project file format: `.vproj` (JSON, versioned).

```text
Project {
  version: "1"
  name: string
  created_at, modified_at: ISO 8601 timestamps
  output_settings: {
    resolution: { width, height }
    framerate: number
    audio_sample_rate: number
  }
  media_pool: MediaItem[]
  timeline: Timeline
}

MediaItem {
  id: uuid
  source_path: absolute path to original
  proxy_path: absolute path to generated proxy, or null
  proxy_status: "pending" | "generating" | "ready" | "failed"
  probe: {
    duration_ms, width, height, fps,
    video_codec, audio_codec, has_audio
  }
}

Timeline {
  duration_ms: number          // computed; cached for UI
  video_track: VideoClip[]
  audio_track: AudioClip[]     // music bed
  text_track: TextClip[]
}

VideoClip {
  id: uuid
  media_id: ref to MediaItem
  source_in_ms, source_out_ms  // trim points within source
  timeline_start_ms
  volume: 0.0..1.0
  muted: bool
  transition_in:  { type, duration_ms }
  transition_out: { type, duration_ms }
}
// transition.type ∈ "cut" | "fade" | "crossfade" | "dip-black" | "dip-white" | "slide"

AudioClip {
  id, media_id, source_in_ms, source_out_ms,
  timeline_start_ms, volume,
  fade_in_ms, fade_out_ms
}

TextClip {
  id
  text: string
  timeline_start_ms, duration_ms
  style: {
    font_family, size_px, color,
    weight, bg_color?, bg_opacity?
  }
  position: { x_pct, y_pct, anchor }
  // anchor ∈ "tl"|"tc"|"tr"|"ml"|"mc"|"mr"|"bl"|"bc"|"br"
  kind: "title" | "caption"    // UX hint; identical render path
}
```

### Decisions

- **Single video track** matching the "handful of clips" scope. Crossfades are expressed by overlapping adjacent clips on this single track, not by a second track.
- **One audio track** (music bed). Per-clip volume and mute on `VideoClip` covers source audio. Voiceover, if needed later, fits the same `AudioClip` model.
- **Text is its own track**, rendered on top of video. Captions and titles share data; `kind` is a UX hint for default style and inspector behavior.
- **Times are integer milliseconds** to avoid float drift. Frame-accuracy is computed from the project framerate at export time.
- **Source paths are absolute.** If a file moves, the project marks it missing; user can relink. No bundled-media mode in v1.
- **Proxy paths live outside the project file**, in `$XDG_CACHE_HOME/<app>/proxies/<media_id>.mp4`. Same for thumbnails (`thumbnails/<media_id>/`) and waveforms (`waveforms/<media_id>.json`).

## 5. UI layout

```
┌──────────────────────────────────────────────────────────────┐
│ Menu bar: File | Edit | View | Export | Help                 │
├──────────────────┬───────────────────────────────────────────┤
│  Media Pool      │            Preview Player                 │
│  (thumbnails,    │       (video output, scrubbable)          │
│   proxy status,  │                                           │
│   drag handles)  │   ▶ ⏸  00:01:23 / 00:05:42  [vol] [⛶]     │
├──────────────────┴───────────────────────────────────────────┤
│  Inspector / Properties (context-sensitive to selection)     │
├──────────────────────────────────────────────────────────────┤
│  Timeline                                                    │
│   Ruler ─────────────────────────────────────────────────    │
│   V:  [clip1][clip2 ──crossfade── ][clip3]      [clip4]      │
│   A:  ════════════════ music bed ═══════════                 │
│   T:           [Title]              [Caption block]          │
│   Toolbar: ✂ split  🗑 delete  ⤺ undo  ⤻ redo  🔍± zoom        │
└──────────────────────────────────────────────────────────────┘
```

### Components

| Component | Responsibility |
|---|---|
| `MediaPool` | List imported clips, show proxy-ready status and thumbnail, drag handles |
| `PreviewPlayer` | Single `<video>` element + transport; switches source as playhead crosses clip boundaries; renders text overlays in a layered DOM/canvas above |
| `Timeline` | Render three tracks (video with thumbnail strip, audio with waveform, text); handle clip drag/trim/split; own the playhead |
| `Inspector` | Edit properties of the currently selected clip on any track |
| `Toolbar` | Tool actions; mostly keyboard-shortcut equivalents |
| `ExportDialog` | Preset picker + custom panel; progress bar; cancel |
| `ImportDialog` | File picker; kicks off `import_media` |
| `MenuBar` | Native menu via Tauri menu API |

### Keyboard shortcuts

- **Space** — play/pause
- **J / K / L** — reverse / pause / forward
- **I / O** — set in/out points on selected clip
- **S** or **Ctrl+K** — split at playhead
- **Delete** — delete selected clip
- **Ctrl+Z / Ctrl+Shift+Z** — undo/redo
- **+ / -** — zoom timeline in/out
- **Home / End** — jump to start/end
- **Arrow keys** — nudge selected clip by 1 frame; **Shift+Arrow** by 10 frames

### Inspector behavior

Content depends on selection: empty state when nothing's selected; `VideoClip` props when a video clip is selected; `AudioClip` props for music; `TextClip` props for text. No separate "effects browser" panel — transitions are picked from a dropdown in the Inspector when a video clip is selected.

## 6. v1 feature set

### Included

- Project lifecycle: new, open, save, save-as, recent projects list, autosave every 30s to `<project>.vproj.autosave`
- Media import: file picker, multi-select, auto-probe, auto-kick proxy generation
- **Proxy pipeline:** background worker generates 540p H.264 proxies; status surfaced in MediaPool; export always uses originals
- Single video track, single music track, single text track
- Timeline editing: drag from MediaPool, trim either edge (rolling), split at playhead, delete, reorder by drag, snap-to-edges with hold-to-disable modifier
- Per-clip volume + mute
- Music track: drop audio, trim, fade-in/fade-out, volume
- Transitions library: cut, fade, crossfade, dip-to-black, dip-to-white, slide; default duration 500ms; selected via Inspector dropdown
- Text overlays: title and manual caption blocks; font, size, color, weight, optional background; 9-point anchor positioning; preview reflects them
- Preview playback uses proxies; play/pause/scrub; respects transitions and overlays at "good enough" fidelity
- Export: presets (YouTube 1080p, YouTube 4K, Twitter/X, Instagram vertical, "small file") + custom panel (resolution, fps, video codec, video bitrate, audio bitrate, container); progress bar; cancel
- Undo/redo for all timeline operations
- Keyboard shortcuts as listed
- **Audio waveform display on the timeline** (both clip audio and music bed)
- **Thumbnail strip on video clips in the timeline** (lazily generated from proxy)
- Linux desktop integration: `.desktop` entry, `.vproj` file association, AppImage packaging

### Explicitly excluded from v1

- Multi-track video / picture-in-picture
- Color grading, LUTs, scopes
- Speed ramps, slow-mo, reverse playback
- Animated keyframes for any property
- Auto-captions / speech-to-text
- Plugins, extensions, scripting
- Cloud sync, collaboration
- Bundled media / portable projects
- Effects beyond transitions and basic text
- Multiple selection of clips for batch edit (single-select only in v1)

## 7. Export pipeline

The timeline is compiled into a single FFmpeg invocation with a filter graph.

### Compile step

1. Walk the timeline; build an FFmpeg filter graph:
   - Each video clip → `[N:v]trim=start:end,setpts=...[vN]`
   - Crossfades / fades → `xfade` filter between adjacent video segments
   - Text overlays → `drawtext` filter with timed `enable=between(t,...)` expressions
   - Per-clip audio gain → `volume` filter on each clip's audio
   - Music bed → mixed in via `amix`, with `afade` for fade-in/out
2. Apply codec, bitrate, resolution, framerate from the chosen preset
3. Spawn `ffmpeg` as a subprocess with the assembled command
4. Parse stderr for progress (`out_time_ms` lines) and emit `export_progress` events
5. On cancel: send SIGTERM, then SIGKILL after a short grace period; clean up partial output (deletion is opt-in via UI confirmation; preserve by default for inspection)

### Why a single-shot command

Simpler error model, no intermediate files to clean up, FFmpeg handles transitions natively. The compiler produces a deterministic graph string for a given project, which is the basis for snapshot tests.

## 8. Error handling

| Failure | Detected at | UX |
|---|---|---|
| Source file moved/deleted | Project load and on access | Mark `MediaItem` as missing; placeholder on timeline; "Relink media" file picker |
| Codec unsupported by bundled FFmpeg | Probe at import | Reject import with clear error and a suggested re-encode command |
| Proxy generation fails | Background worker | Mark proxy `failed`; preview falls back to original (best-effort, may stutter); export still works (uses original) |
| FFmpeg crashes mid-export | Export subprocess | Surface last 50 lines of stderr in a modal; "Copy error" button; partial output preserved by default |
| Disk full during export | Filesystem (ENOSPC from FFmpeg) | Clear error message; offer to delete partial output |
| Project file corrupted on load | `open_project` | Show "couldn't load this project" with parse error; offer most recent autosave if present |
| App crash mid-edit | — (covered by autosave) | Autosave every 30s; on next launch, offer to recover from `<project>.vproj.autosave` |

### Principles

1. **Never silently drop work.** Every failure surfaces in the UI with cause and next action.
2. **Don't make the user debug FFmpeg.** Show human-readable cause first; raw stderr behind a "Show details" toggle.
3. **Failures during background work shouldn't block foreground work.** A failed proxy doesn't stop editing; a failed export doesn't take down the project.

## 9. Testing strategy

### Rust core (heaviest coverage; target 80%+)

- **Project file IO:** round-trip preservation; old-version migration; corrupt-file failure modes
- **Filter graph compiler:** corpus of frozen `Project` → expected filter graph (snapshot tests)
- **Media probe parsing:** `ffprobe` JSON fixtures including pathological cases (no audio, VFR, unusual color spaces)
- **Export progress parser:** stderr fixtures → progress events
- **Subprocess lifecycle:** spawn, cancel, cleanup; no zombie processes; cancel path verified explicitly

### Frontend logic (target 80%+ on reducer/logic modules)

- **Timeline state reducer:** trim, split, delete, reorder, transition application — pure functions, snapshot before/after
- **Undo/redo:** N operations + N undos returns to start state
- **Snap-to-edge math:** unit-tested directly
- **Transition compatibility rules:** e.g., crossfade requires overlap; tested directly
- **Component tests:** Vitest + Testing Library for `MediaPool`, `Inspector`, `ExportDialog` — behavioral, not snapshot churn

### End-to-end (Playwright, small set: 5–10 tests)

- Smoke: launch, open project, timeline renders
- Import → trim → export → verify output exists, expected duration (probed), non-empty
- Add title → export → extract a frame, verify text presence (perceptual hash or OCR)
- Open autosaved project → recovery works
- Cancel export mid-flight → no zombies, no orphaned partials

### Not tested in CI

- Pixel-perfect render output of transitions (visual diff in dev only — FFmpeg version drift breaks pinned hashes)
- Real-time playback frame rate (manual smoke)
- Every export preset combo — exhaustive on the compile step, full export on one or two presets

### Test fixtures

- `tests/fixtures/media/` — small MP4s (10–30s, low bitrate), checked in
- `tests/fixtures/probes/` — canned `ffprobe` JSON
- `tests/fixtures/projects/` — golden `.vproj` files for migration and compile tests

## 10. Open questions / future work

These are deliberately deferred but worth recording so they're not lost:

- **Audio waveform performance:** if rendering waveforms blocks the UI on long music tracks, move waveform extraction to the Rust side and stream peak data to the frontend.
- **Hardware-accelerated decoding:** add VAAPI decode hint to FFmpeg invocations once a target Linux baseline is set; deferred because it varies by hardware.
- **Multi-track video:** likely the first big post-v1 expansion. The current data model has room (`video_track` is already an array; would become `video_tracks: VideoClip[][]`).
- **Auto-captions:** Whisper integration as an optional component. Would slot into the existing `text_track` data model.
- **Project portability:** "copy media into project folder" mode for moving projects between machines.
- **App icon:** TBD before first packaged release.
