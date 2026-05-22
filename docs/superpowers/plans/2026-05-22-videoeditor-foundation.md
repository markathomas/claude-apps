# VideoEditor — Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the VideoEditor app skeleton — a Tauri 2 + Svelte + TypeScript desktop app with a Rust core that can create, save, open, and list `.vproj` project files. UI shows the menu bar and three-pane layout (empty placeholders). No video features yet.

**Architecture:** Tauri 2 shell. Rust backend owns the `Project` data model, JSON serialization, file IO, and a recent-projects registry stored in `$XDG_CONFIG_HOME/videoeditor/recent.json`. Svelte frontend renders the menu bar and pane layout, calls Tauri commands for project lifecycle, and holds a single source-of-truth `projectStore` (Svelte writable store). Tests: Rust unit/integration with `cargo test`, frontend logic tests with Vitest.

**Tech Stack:** Tauri 2, Rust (stable), Svelte 5 + TypeScript, Vite, Vitest, `serde` / `serde_json`, `uuid`, `chrono`, `dirs`.

**Project layout (created by this plan):**

```
videoeditor/
├── Cargo.toml                          (workspace)
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs                     (Tauri entrypoint, command registration)
│   │   ├── lib.rs                      (re-exports)
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── project.rs              (Project, OutputSettings, MediaItem stubs)
│   │   │   ├── timeline.rs             (Timeline, VideoClip, AudioClip, TextClip)
│   │   │   └── transition.rs           (TransitionType, TransitionSpec)
│   │   ├── project_io.rs               (load/save .vproj, autosave path helpers)
│   │   ├── recent.rs                   (recent-projects registry)
│   │   ├── paths.rs                    (XDG dir resolution)
│   │   ├── error.rs                    (AppError enum)
│   │   └── commands.rs                 (Tauri command handlers)
│   └── tests/
│       ├── project_roundtrip.rs
│       ├── recent_registry.rs
│       └── fixtures/
│           ├── projects/empty_v1.vproj
│           └── projects/corrupt.vproj
├── src/                                (Svelte frontend)
│   ├── app.html
│   ├── main.ts
│   ├── App.svelte
│   ├── lib/
│   │   ├── ipc.ts                      (typed wrappers for Tauri invoke)
│   │   ├── types.ts                    (TS mirrors of Rust types)
│   │   ├── stores/
│   │   │   └── projectStore.ts         (current project + persistence helpers)
│   │   ├── components/
│   │   │   ├── MenuBar.svelte
│   │   │   ├── PaneLayout.svelte
│   │   │   ├── MediaPool.svelte        (placeholder)
│   │   │   ├── PreviewPlayer.svelte    (placeholder)
│   │   │   ├── Inspector.svelte        (placeholder)
│   │   │   └── Timeline.svelte         (placeholder)
│   │   └── dialogs/
│   │       ├── NewProjectDialog.svelte
│   │       └── RecentProjectsDialog.svelte
│   └── styles/global.css
├── tests/
│   └── frontend/
│       ├── ipc.test.ts
│       └── projectStore.test.ts
├── package.json
├── tsconfig.json
├── vite.config.ts
├── svelte.config.js
└── README.md
```

---

## Prerequisites (one-time, do once before Task 1)

- [ ] **Verify toolchain installed.**

Run:
```bash
rustc --version
cargo --version
node --version
pnpm --version || npm --version
```

Expected: Rust 1.78+, Node 20+, a package manager available. If `pnpm` isn't installed, this plan uses `npm` — substitute everywhere.

- [ ] **Install Tauri prerequisites for Linux.**

Run:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Expected: packages install. If on a non-Debian distro, find the equivalents (Tauri docs list them).

- [ ] **Confirm working directory.**

Run:
```bash
pwd
```

Expected: `/home/mthomas/data/git/claude-apps`. All paths in this plan are relative to that root.

---

### Task 1: Initialize the workspace

**Files:**
- Create: `videoeditor/package.json`
- Create: `videoeditor/tsconfig.json`
- Create: `videoeditor/vite.config.ts`
- Create: `videoeditor/svelte.config.js`
- Create: `videoeditor/.gitignore`
- Create: `videoeditor/index.html`
- Create: `videoeditor/src/main.ts`
- Create: `videoeditor/src/App.svelte`
- Create: `videoeditor/src/styles/global.css`
- Create: `videoeditor/README.md`

- [ ] **Step 1: Create the directory and `package.json`.**

```bash
mkdir -p videoeditor/src/lib/components videoeditor/src/lib/dialogs videoeditor/src/lib/stores videoeditor/src/styles videoeditor/tests/frontend
```

Write `videoeditor/package.json`:

```json
{
  "name": "videoeditor",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest run",
    "test:watch": "vitest",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "@testing-library/svelte": "^5.2.0",
    "@tsconfig/svelte": "^5.0.4",
    "@types/node": "^22.0.0",
    "jsdom": "^25.0.0",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "tslib": "^2.7.0",
    "typescript": "^5.6.0",
    "vite": "^5.4.0",
    "vitest": "^2.1.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0"
  }
}
```

- [ ] **Step 2: Write `videoeditor/tsconfig.json`.**

```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitOverride": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "types": ["vitest/globals"],
    "baseUrl": ".",
    "paths": {
      "$lib/*": ["src/lib/*"]
    }
  },
  "include": ["src/**/*", "tests/**/*"]
}
```

- [ ] **Step 3: Write `videoeditor/svelte.config.js`.**

```js
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
};
```

- [ ] **Step 4: Write `videoeditor/vite.config.ts`.**

```ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['tests/frontend/**/*.test.ts'],
  },
});
```

- [ ] **Step 5: Write `videoeditor/.gitignore`.**

```
node_modules
dist
src-tauri/target
src-tauri/gen
.DS_Store
*.log
```

- [ ] **Step 6: Write `videoeditor/index.html`.**

Vite expects `index.html` at the project root and uses it as the entrypoint.

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>VideoEditor</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 7: Write `videoeditor/src/main.ts`.**

```ts
import { mount } from 'svelte';
import App from './App.svelte';
import './styles/global.css';

const app = mount(App, { target: document.getElementById('app')! });

export default app;
```

- [ ] **Step 8: Write `videoeditor/src/App.svelte`.**

```svelte
<script lang="ts">
  // Foundation shell — replaced in subsequent tasks.
</script>

<main>
  <h1>VideoEditor</h1>
  <p>Foundation skeleton.</p>
</main>

<style>
  main {
    font-family: system-ui, sans-serif;
    padding: 2rem;
  }
</style>
```

- [ ] **Step 9: Write `videoeditor/src/styles/global.css`.**

```css
:root {
  color-scheme: light dark;
  font-family: system-ui, -apple-system, sans-serif;
}

html, body, #app {
  height: 100%;
  margin: 0;
}

body {
  background: var(--bg, #1a1a1a);
  color: var(--fg, #e5e5e5);
}
```

- [ ] **Step 10: Write `videoeditor/README.md`.**

```markdown
# VideoEditor

A Linux desktop video editor for quick clip work.

See `docs/superpowers/specs/2026-05-22-videoeditor-design.md` for the design.

## Development

```bash
cd videoeditor
npm install
npm run tauri dev
```

## Tests

- Frontend logic: `npm test`
- Rust core: `cd src-tauri && cargo test`
```

- [ ] **Step 11: Install dependencies.**

Run:
```bash
cd videoeditor && npm install
```

Expected: dependencies install. Warnings about peer deps are fine.

- [ ] **Step 12: Verify TypeScript compiles.**

Run:
```bash
cd videoeditor && npx svelte-check
```

Expected: 0 errors. Warnings are OK.

- [ ] **Step 13: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/package.json videoeditor/tsconfig.json videoeditor/svelte.config.js videoeditor/vite.config.ts videoeditor/.gitignore videoeditor/index.html videoeditor/src/main.ts videoeditor/src/App.svelte videoeditor/src/styles/global.css videoeditor/README.md
git commit -m "feat(videoeditor): scaffold svelte+ts+vite frontend"
```

---

### Task 2: Initialize the Tauri 2 backend

**Files:**
- Create: `videoeditor/src-tauri/Cargo.toml`
- Create: `videoeditor/src-tauri/build.rs`
- Create: `videoeditor/src-tauri/tauri.conf.json`
- Create: `videoeditor/src-tauri/src/main.rs`
- Create: `videoeditor/src-tauri/src/lib.rs`
- Create: `videoeditor/src-tauri/capabilities/default.json`

- [ ] **Step 1: Initialize the Tauri Rust crate.**

Write `videoeditor/src-tauri/Cargo.toml`:

```toml
[package]
name = "videoeditor"
version = "0.1.0"
description = "Linux desktop video editor"
authors = ["mthomas"]
edition = "2021"
rust-version = "1.78"

[lib]
name = "videoeditor_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
thiserror = "1"

[dev-dependencies]
tempfile = "3"
pretty_assertions = "1"
```

- [ ] **Step 2: Write `videoeditor/src-tauri/build.rs`.**

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 3: Write `videoeditor/src-tauri/tauri.conf.json`.**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "VideoEditor",
  "version": "0.1.0",
  "identifier": "dev.videoeditor.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "VideoEditor",
        "width": 1400,
        "height": 900,
        "minWidth": 1024,
        "minHeight": 640
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": false,
    "targets": "all",
    "icon": ["icons/icon.png"]
  },
  "plugins": {}
}
```

- [ ] **Step 4: Write `videoeditor/src-tauri/capabilities/default.json`.**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default"
  ]
}
```

- [ ] **Step 5: Write `videoeditor/src-tauri/src/main.rs`.**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    videoeditor_lib::run();
}
```

- [ ] **Step 6: Write `videoeditor/src-tauri/src/lib.rs`.**

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Verify Rust builds.**

Run:
```bash
cd videoeditor/src-tauri && cargo build
```

Expected: build succeeds. First build will take 2–5 minutes (Tauri pulls a lot of crates).

- [ ] **Step 8: Verify the dev server boots (smoke test).**

Run:
```bash
cd videoeditor && npm run tauri dev
```

Expected: a window opens showing "VideoEditor / Foundation skeleton." Close it (Ctrl+C in the terminal).

If the window opens, success. If `webkit2gtk` errors appear, re-check the Prerequisites section.

- [ ] **Step 9: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/Cargo.toml videoeditor/src-tauri/build.rs videoeditor/src-tauri/tauri.conf.json videoeditor/src-tauri/capabilities/default.json videoeditor/src-tauri/src/main.rs videoeditor/src-tauri/src/lib.rs videoeditor/src-tauri/Cargo.lock
git commit -m "feat(videoeditor): bootstrap tauri 2 rust shell"
```

---

### Task 3: Define the data model (transition + clip types)

**Files:**
- Create: `videoeditor/src-tauri/src/model/mod.rs`
- Create: `videoeditor/src-tauri/src/model/transition.rs`
- Test: `videoeditor/src-tauri/src/model/transition.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write the failing test for transitions.**

Create `videoeditor/src-tauri/src/model/transition.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionType {
    Cut,
    Fade,
    Crossfade,
    DipBlack,
    DipWhite,
    Slide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionSpec {
    #[serde(rename = "type")]
    pub kind: TransitionType,
    pub duration_ms: u64,
}

impl Default for TransitionSpec {
    fn default() -> Self {
        Self { kind: TransitionType::Cut, duration_ms: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_type_serializes_kebab_case() {
        let json = serde_json::to_string(&TransitionType::DipBlack).unwrap();
        assert_eq!(json, "\"dip-black\"");
    }

    #[test]
    fn transition_type_deserializes_all_variants() {
        let cases = [
            ("\"cut\"", TransitionType::Cut),
            ("\"fade\"", TransitionType::Fade),
            ("\"crossfade\"", TransitionType::Crossfade),
            ("\"dip-black\"", TransitionType::DipBlack),
            ("\"dip-white\"", TransitionType::DipWhite),
            ("\"slide\"", TransitionType::Slide),
        ];
        for (json, expected) in cases {
            let parsed: TransitionType = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn transition_spec_default_is_cut_zero() {
        let spec = TransitionSpec::default();
        assert_eq!(spec.kind, TransitionType::Cut);
        assert_eq!(spec.duration_ms, 0);
    }
}
```

- [ ] **Step 2: Create `videoeditor/src-tauri/src/model/mod.rs`.**

```rust
pub mod transition;
```

- [ ] **Step 3: Wire the module into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs` to add `pub mod model;` above the existing `pub fn run()`:

```rust
pub mod model;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test --lib model::transition
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/model/mod.rs videoeditor/src-tauri/src/model/transition.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): add transition data model"
```

---

### Task 4: Define the timeline clip types

**Files:**
- Create: `videoeditor/src-tauri/src/model/timeline.rs`

- [ ] **Step 1: Write the failing tests.**

Create `videoeditor/src-tauri/src/model/timeline.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::transition::TransitionSpec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Uuid,
    pub media_id: Uuid,
    pub source_in_ms: u64,
    pub source_out_ms: u64,
    pub timeline_start_ms: u64,
    pub volume: f32,
    pub muted: bool,
    pub transition_in: TransitionSpec,
    pub transition_out: TransitionSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: Uuid,
    pub media_id: Uuid,
    pub source_in_ms: u64,
    pub source_out_ms: u64,
    pub timeline_start_ms: u64,
    pub volume: f32,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAnchor {
    Tl, Tc, Tr,
    Ml, Mc, Mr,
    Bl, Bc, Br,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub size_px: u32,
    pub color: String,            // "#rrggbb" or "#rrggbbaa"
    pub weight: u16,              // 100..900
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_opacity: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextPosition {
    pub x_pct: f32,
    pub y_pct: f32,
    pub anchor: TextAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextKind {
    Title,
    Caption,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextClip {
    pub id: Uuid,
    pub text: String,
    pub timeline_start_ms: u64,
    pub duration_ms: u64,
    pub style: TextStyle,
    pub position: TextPosition,
    pub kind: TextKind,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub duration_ms: u64,
    pub video_track: Vec<VideoClip>,
    pub audio_track: Vec<AudioClip>,
    pub text_track: Vec<TextClip>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::transition::{TransitionSpec, TransitionType};

    #[test]
    fn empty_timeline_serializes_with_empty_arrays() {
        let timeline = Timeline::default();
        let json = serde_json::to_value(&timeline).unwrap();
        assert_eq!(json["duration_ms"], 0);
        assert_eq!(json["video_track"], serde_json::json!([]));
        assert_eq!(json["audio_track"], serde_json::json!([]));
        assert_eq!(json["text_track"], serde_json::json!([]));
    }

    #[test]
    fn video_clip_round_trip() {
        let clip = VideoClip {
            id: Uuid::nil(),
            media_id: Uuid::nil(),
            source_in_ms: 100,
            source_out_ms: 5000,
            timeline_start_ms: 0,
            volume: 0.8,
            muted: false,
            transition_in: TransitionSpec::default(),
            transition_out: TransitionSpec { kind: TransitionType::Fade, duration_ms: 500 },
        };
        let json = serde_json::to_string(&clip).unwrap();
        let parsed: VideoClip = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, clip);
    }

    #[test]
    fn text_anchor_serializes_lowercase() {
        let json = serde_json::to_string(&TextAnchor::Bc).unwrap();
        assert_eq!(json, "\"bc\"");
    }

    #[test]
    fn text_clip_round_trip_no_bg() {
        let clip = TextClip {
            id: Uuid::nil(),
            text: "Hello".into(),
            timeline_start_ms: 1000,
            duration_ms: 3000,
            style: TextStyle {
                font_family: "Inter".into(),
                size_px: 48,
                color: "#ffffff".into(),
                weight: 700,
                bg_color: None,
                bg_opacity: None,
            },
            position: TextPosition { x_pct: 50.0, y_pct: 50.0, anchor: TextAnchor::Mc },
            kind: TextKind::Title,
        };
        let json = serde_json::to_string(&clip).unwrap();
        assert!(!json.contains("bg_color"), "bg_color should be omitted when None");
        let parsed: TextClip = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, clip);
    }
}
```

- [ ] **Step 2: Wire the module.**

Edit `videoeditor/src-tauri/src/model/mod.rs`:

```rust
pub mod timeline;
pub mod transition;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test --lib model::timeline
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/model/mod.rs videoeditor/src-tauri/src/model/timeline.rs
git commit -m "feat(videoeditor): add timeline and clip types"
```

---

### Task 5: Define the Project + MediaItem types

**Files:**
- Create: `videoeditor/src-tauri/src/model/project.rs`

- [ ] **Step 1: Write the failing tests.**

Create `videoeditor/src-tauri/src/model/project.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::timeline::Timeline;

pub const PROJECT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OutputSettings {
    pub resolution: Resolution,
    pub framerate: f32,
    pub audio_sample_rate: u32,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution { width: 1920, height: 1080 },
            framerate: 30.0,
            audio_sample_rate: 48_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Probe {
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub video_codec: String,
    pub audio_codec: Option<String>,
    pub has_audio: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyStatus {
    Pending,
    Generating,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Uuid,
    pub source_path: String,
    pub proxy_path: Option<String>,
    pub proxy_status: ProxyStatus,
    pub probe: Option<Probe>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub output_settings: OutputSettings,
    pub media_pool: Vec<MediaItem>,
    pub timeline: Timeline,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            version: PROJECT_VERSION.into(),
            name,
            created_at: now,
            modified_at: now,
            output_settings: OutputSettings::default(),
            media_pool: Vec::new(),
            timeline: Timeline::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_has_version_1_and_empty_collections() {
        let p = Project::new("My Edit".into());
        assert_eq!(p.version, "1");
        assert_eq!(p.name, "My Edit");
        assert!(p.media_pool.is_empty());
        assert_eq!(p.timeline.duration_ms, 0);
        assert!(p.timeline.video_track.is_empty());
        assert_eq!(p.created_at, p.modified_at);
    }

    #[test]
    fn output_settings_default_is_1080p_30fps_48k() {
        let s = OutputSettings::default();
        assert_eq!(s.resolution.width, 1920);
        assert_eq!(s.resolution.height, 1080);
        assert!((s.framerate - 30.0).abs() < f32::EPSILON);
        assert_eq!(s.audio_sample_rate, 48_000);
    }

    #[test]
    fn proxy_status_serializes_lowercase() {
        let json = serde_json::to_string(&ProxyStatus::Generating).unwrap();
        assert_eq!(json, "\"generating\"");
    }

    #[test]
    fn project_serde_round_trip() {
        let p = Project::new("Test".into());
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }
}
```

- [ ] **Step 2: Wire the module.**

Edit `videoeditor/src-tauri/src/model/mod.rs`:

```rust
pub mod project;
pub mod timeline;
pub mod transition;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test --lib model::project
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/model/mod.rs videoeditor/src-tauri/src/model/project.rs
git commit -m "feat(videoeditor): add Project and MediaItem types"
```

---

### Task 6: AppError type

**Files:**
- Create: `videoeditor/src-tauri/src/error.rs`

- [ ] **Step 1: Write the failing test.**

Create `videoeditor/src-tauri/src/error.rs`:

```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("project file not found: {0}")]
    ProjectNotFound(PathBuf),

    #[error("project file is corrupt: {message}")]
    ProjectCorrupt { message: String },

    #[error("unsupported project version: {found} (supported: {supported})")]
    UnsupportedVersion { found: String, supported: String },

    #[error("invalid path: {0}")]
    InvalidPath(String),
}

pub type AppResult<T> = Result<T, AppError>;

// Tauri command results need to serialize errors to strings for the frontend.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_corrupt_error_displays_message() {
        let e = AppError::ProjectCorrupt { message: "expected }".into() };
        assert_eq!(e.to_string(), "project file is corrupt: expected }");
    }

    #[test]
    fn unsupported_version_error_displays_both_versions() {
        let e = AppError::UnsupportedVersion {
            found: "2".into(),
            supported: "1".into(),
        };
        assert_eq!(e.to_string(), "unsupported project version: 2 (supported: 1)");
    }

    #[test]
    fn app_error_serializes_to_string_message() {
        let e = AppError::InvalidPath("not absolute".into());
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, "\"invalid path: not absolute\"");
    }
}
```

- [ ] **Step 2: Wire it into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`:

```rust
pub mod error;
pub mod model;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test --lib error
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/error.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): add AppError type"
```

---

### Task 7: XDG paths helper

**Files:**
- Create: `videoeditor/src-tauri/src/paths.rs`

- [ ] **Step 1: Write the failing test.**

Create `videoeditor/src-tauri/src/paths.rs`:

```rust
use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub const APP_DIR_NAME: &str = "videoeditor";

pub fn config_dir() -> AppResult<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| AppError::InvalidPath("config dir unavailable".into()))?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn cache_dir() -> AppResult<PathBuf> {
    let base = dirs::cache_dir()
        .ok_or_else(|| AppError::InvalidPath("cache dir unavailable".into()))?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn ensure_dir(path: &PathBuf) -> AppResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn recent_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("recent.json"))
}

pub fn proxies_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("proxies"))
}

pub fn thumbnails_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("thumbnails"))
}

pub fn waveforms_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("waveforms"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_app_name() {
        let p = config_dir().unwrap();
        assert!(p.ends_with(APP_DIR_NAME), "expected dir ending with {APP_DIR_NAME}, got {p:?}");
    }

    #[test]
    fn cache_subdirs_are_under_app_cache() {
        let cache = cache_dir().unwrap();
        assert!(proxies_dir().unwrap().starts_with(&cache));
        assert!(thumbnails_dir().unwrap().starts_with(&cache));
        assert!(waveforms_dir().unwrap().starts_with(&cache));
    }

    #[test]
    fn recent_file_path_is_under_config_dir() {
        let recent = recent_file_path().unwrap();
        let config = config_dir().unwrap();
        assert!(recent.starts_with(&config));
        assert_eq!(recent.file_name().unwrap(), "recent.json");
    }

    #[test]
    fn ensure_dir_creates_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("a/b/c");
        assert!(!target.exists());
        ensure_dir(&target).unwrap();
        assert!(target.exists());
    }
}
```

- [ ] **Step 2: Wire it into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`:

```rust
pub mod error;
pub mod model;
pub mod paths;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test --lib paths
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/paths.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): add XDG paths helper"
```

---

### Task 8: Project file IO (load + save)

**Files:**
- Create: `videoeditor/src-tauri/src/project_io.rs`
- Create: `videoeditor/src-tauri/tests/project_roundtrip.rs`
- Create: `videoeditor/src-tauri/tests/fixtures/projects/empty_v1.vproj`
- Create: `videoeditor/src-tauri/tests/fixtures/projects/corrupt.vproj`

- [ ] **Step 1: Write the unit tests for `project_io.rs`.**

Create `videoeditor/src-tauri/src/project_io.rs`:

```rust
use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::model::project::{Project, PROJECT_VERSION};

pub fn save_project(project: &Project, path: &Path) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(project)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_project(path: &Path) -> AppResult<Project> {
    if !path.exists() {
        return Err(AppError::ProjectNotFound(path.to_path_buf()));
    }
    let bytes = std::fs::read(path)?;
    let project: Project = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::ProjectCorrupt { message: e.to_string() }
    })?;
    if project.version != PROJECT_VERSION {
        return Err(AppError::UnsupportedVersion {
            found: project.version,
            supported: PROJECT_VERSION.to_string(),
        });
    }
    Ok(project)
}

pub fn autosave_path_for(project_path: &Path) -> std::path::PathBuf {
    let mut s = project_path.as_os_str().to_owned();
    s.push(".autosave");
    s.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::project::Project;

    #[test]
    fn save_then_load_preserves_project() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("p.vproj");
        let original = Project::new("Round Trip".into());
        save_project(&original, &path).unwrap();
        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded, original);
    }

    #[test]
    fn save_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nested/deep/p.vproj");
        let p = Project::new("X".into());
        save_project(&p, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn load_missing_returns_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("missing.vproj");
        let err = load_project(&path).unwrap_err();
        assert!(matches!(err, AppError::ProjectNotFound(_)));
    }

    #[test]
    fn load_corrupt_returns_corrupt() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.vproj");
        std::fs::write(&path, "{ not valid json").unwrap();
        let err = load_project(&path).unwrap_err();
        assert!(matches!(err, AppError::ProjectCorrupt { .. }));
    }

    #[test]
    fn load_unsupported_version_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("v2.vproj");
        let mut json = serde_json::to_value(Project::new("V2".into())).unwrap();
        json["version"] = serde_json::Value::String("2".into());
        std::fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();
        let err = load_project(&path).unwrap_err();
        assert!(matches!(err, AppError::UnsupportedVersion { .. }));
    }

    #[test]
    fn autosave_path_appends_dot_autosave() {
        let p = std::path::PathBuf::from("/tmp/foo.vproj");
        let auto = autosave_path_for(&p);
        assert_eq!(auto, std::path::PathBuf::from("/tmp/foo.vproj.autosave"));
    }
}
```

- [ ] **Step 2: Wire `project_io` into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`:

```rust
pub mod error;
pub mod model;
pub mod paths;
pub mod project_io;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Create the corrupt fixture (used by integration test in next step).**

```bash
mkdir -p videoeditor/src-tauri/tests/fixtures/projects
```

Write `videoeditor/src-tauri/tests/fixtures/projects/corrupt.vproj`:

```
{ this is not json
```

- [ ] **Step 4: Generate the empty v1 fixture programmatically (write tooling first, then commit the fixture).**

Run from `videoeditor/src-tauri`:

```bash
cargo run --example gen_empty_fixture 2>/dev/null || true
```

That example doesn't exist; instead, create the fixture directly. Write `videoeditor/src-tauri/tests/fixtures/projects/empty_v1.vproj`:

```json
{
  "version": "1",
  "name": "Empty",
  "created_at": "2026-05-22T00:00:00Z",
  "modified_at": "2026-05-22T00:00:00Z",
  "output_settings": {
    "resolution": { "width": 1920, "height": 1080 },
    "framerate": 30.0,
    "audio_sample_rate": 48000
  },
  "media_pool": [],
  "timeline": {
    "duration_ms": 0,
    "video_track": [],
    "audio_track": [],
    "text_track": []
  }
}
```

- [ ] **Step 5: Write the integration test using the fixtures.**

Create `videoeditor/src-tauri/tests/project_roundtrip.rs`:

```rust
use std::path::PathBuf;

use videoeditor_lib::project_io::{load_project, save_project};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/projects")
        .join(name)
}

#[test]
fn loads_empty_v1_fixture() {
    let p = load_project(&fixture("empty_v1.vproj")).unwrap();
    assert_eq!(p.version, "1");
    assert_eq!(p.name, "Empty");
    assert!(p.media_pool.is_empty());
    assert!(p.timeline.video_track.is_empty());
}

#[test]
fn corrupt_fixture_is_rejected() {
    let err = load_project(&fixture("corrupt.vproj")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("corrupt"), "expected corrupt error, got: {msg}");
}

#[test]
fn save_then_load_round_trip_with_real_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("rt.vproj");
    let original = load_project(&fixture("empty_v1.vproj")).unwrap();
    save_project(&original, &path).unwrap();
    let loaded = load_project(&path).unwrap();
    assert_eq!(loaded, original);
}
```

- [ ] **Step 6: Run all tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test
```

Expected: all tests pass (unit tests in `project_io` + 3 integration tests).

- [ ] **Step 7: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/project_io.rs videoeditor/src-tauri/src/lib.rs videoeditor/src-tauri/tests/project_roundtrip.rs videoeditor/src-tauri/tests/fixtures/projects/empty_v1.vproj videoeditor/src-tauri/tests/fixtures/projects/corrupt.vproj
git commit -m "feat(videoeditor): project save/load with version check"
```

---

### Task 9: Recent projects registry

**Files:**
- Create: `videoeditor/src-tauri/src/recent.rs`
- Create: `videoeditor/src-tauri/tests/recent_registry.rs`

- [ ] **Step 1: Write the implementation with unit tests.**

Create `videoeditor/src-tauri/src/recent.rs`:

```rust
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

pub const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecentRegistry {
    pub items: Vec<RecentProject>,
}

impl RecentRegistry {
    pub fn touch(&mut self, path: &Path, name: &str) {
        let path_str = path.to_string_lossy().to_string();
        self.items.retain(|r| r.path != path_str);
        self.items.insert(0, RecentProject {
            path: path_str,
            name: name.to_string(),
            last_opened: Utc::now(),
        });
        self.items.truncate(MAX_RECENT);
    }

    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        self.items.retain(|r| r.path != path_str);
    }

    pub fn load(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = std::fs::read(path)?;
        let registry: Self = serde_json::from_slice(&bytes).unwrap_or_default();
        Ok(registry)
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn prune_missing(&mut self) {
        self.items.retain(|r| PathBuf::from(&r.path).exists());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_inserts_at_front_and_dedupes() {
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.touch(Path::new("/b/p.vproj"), "B");
        r.touch(Path::new("/a/p.vproj"), "A again");
        assert_eq!(r.items.len(), 2);
        assert_eq!(r.items[0].path, "/a/p.vproj");
        assert_eq!(r.items[0].name, "A again");
        assert_eq!(r.items[1].path, "/b/p.vproj");
    }

    #[test]
    fn touch_caps_at_max_recent() {
        let mut r = RecentRegistry::default();
        for i in 0..(MAX_RECENT + 5) {
            r.touch(&PathBuf::from(format!("/p{i}.vproj")), &format!("P{i}"));
        }
        assert_eq!(r.items.len(), MAX_RECENT);
        // most recent inserted is at front
        assert_eq!(r.items[0].path, format!("/p{}.vproj", MAX_RECENT + 4));
    }

    #[test]
    fn remove_drops_entry() {
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.touch(Path::new("/b/p.vproj"), "B");
        r.remove(Path::new("/a/p.vproj"));
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].path, "/b/p.vproj");
    }

    #[test]
    fn load_returns_empty_when_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nope.json");
        let r = RecentRegistry::load(&path).unwrap();
        assert!(r.items.is_empty());
    }

    #[test]
    fn save_then_load_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("recent.json");
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.save(&path).unwrap();
        let loaded = RecentRegistry::load(&path).unwrap();
        assert_eq!(loaded, r);
    }

    #[test]
    fn load_corrupt_file_yields_empty_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("recent.json");
        std::fs::write(&path, "garbage").unwrap();
        let r = RecentRegistry::load(&path).unwrap();
        assert!(r.items.is_empty());
    }

    #[test]
    fn prune_missing_drops_paths_that_dont_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let exists = tmp.path().join("real.vproj");
        std::fs::write(&exists, "{}").unwrap();
        let mut r = RecentRegistry::default();
        r.touch(&exists, "Real");
        r.touch(Path::new("/does/not/exist.vproj"), "Ghost");
        r.prune_missing();
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].path, exists.to_string_lossy());
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`:

```rust
pub mod error;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Write the integration test.**

Create `videoeditor/src-tauri/tests/recent_registry.rs`:

```rust
use std::path::Path;

use videoeditor_lib::recent::RecentRegistry;

#[test]
fn end_to_end_touch_save_load() {
    let tmp = tempfile::tempdir().unwrap();
    let registry_path = tmp.path().join("recent.json");

    let mut r = RecentRegistry::load(&registry_path).unwrap();
    assert!(r.items.is_empty());

    r.touch(Path::new("/projects/one.vproj"), "One");
    r.touch(Path::new("/projects/two.vproj"), "Two");
    r.save(&registry_path).unwrap();

    let reloaded = RecentRegistry::load(&registry_path).unwrap();
    assert_eq!(reloaded.items.len(), 2);
    assert_eq!(reloaded.items[0].path, "/projects/two.vproj");
    assert_eq!(reloaded.items[1].path, "/projects/one.vproj");
}
```

- [ ] **Step 4: Run tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test
```

Expected: all tests pass (7 unit + 1 integration for recent, plus prior).

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/recent.rs videoeditor/src-tauri/src/lib.rs videoeditor/src-tauri/tests/recent_registry.rs
git commit -m "feat(videoeditor): recent projects registry"
```

---

### Task 10: Tauri commands for project lifecycle

**Files:**
- Create: `videoeditor/src-tauri/src/commands.rs`
- Modify: `videoeditor/src-tauri/src/lib.rs`

- [ ] **Step 1: Write `commands.rs`.**

Create `videoeditor/src-tauri/src/commands.rs`:

```rust
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::model::project::Project;
use crate::paths::{ensure_dir, recent_file_path};
use crate::project_io::{load_project, save_project};
use crate::recent::{RecentProject, RecentRegistry};

#[tauri::command]
pub fn new_project(name: String) -> AppResult<Project> {
    Ok(Project::new(name))
}

#[tauri::command]
pub fn open_project(path: String) -> AppResult<Project> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_absolute() {
        return Err(AppError::InvalidPath(format!("not absolute: {path}")));
    }
    let project = load_project(&path_buf)?;

    let registry_path = recent_file_path()?;
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.touch(&path_buf, &project.name);
    registry.save(&registry_path)?;

    Ok(project)
}

#[tauri::command]
pub fn save_project_cmd(project: Project, path: String) -> AppResult<()> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_absolute() {
        return Err(AppError::InvalidPath(format!("not absolute: {path}")));
    }
    save_project(&project, &path_buf)?;

    let registry_path = recent_file_path()?;
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.touch(&path_buf, &project.name);
    registry.save(&registry_path)?;

    Ok(())
}

#[tauri::command]
pub fn get_recent_projects() -> AppResult<Vec<RecentProject>> {
    let registry_path = recent_file_path()?;
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.prune_missing();
    Ok(registry.items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_command_returns_named_project() {
        let p = new_project("Hello".into()).unwrap();
        assert_eq!(p.name, "Hello");
        assert_eq!(p.version, "1");
    }

    #[test]
    fn open_project_rejects_relative_path() {
        let err = open_project("relative/path.vproj".into()).unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }

    #[test]
    fn save_project_rejects_relative_path() {
        let p = Project::new("X".into());
        let err = save_project_cmd(p, "relative/path.vproj".into()).unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }
}
```

- [ ] **Step 2: Register commands in `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests + verify build.**

Run:
```bash
cd videoeditor/src-tauri && cargo test && cargo build
```

Expected: tests pass, build succeeds.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/commands.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): tauri commands for project lifecycle"
```

---

### Task 11: TypeScript type mirrors

**Files:**
- Create: `videoeditor/src/lib/types.ts`

- [ ] **Step 1: Write the types.**

Create `videoeditor/src/lib/types.ts`:

```ts
export type TransitionType =
  | 'cut' | 'fade' | 'crossfade' | 'dip-black' | 'dip-white' | 'slide';

export interface TransitionSpec {
  type: TransitionType;
  duration_ms: number;
}

export interface Resolution {
  width: number;
  height: number;
}

export interface OutputSettings {
  resolution: Resolution;
  framerate: number;
  audio_sample_rate: number;
}

export interface Probe {
  duration_ms: number;
  width: number;
  height: number;
  fps: number;
  video_codec: string;
  audio_codec: string | null;
  has_audio: boolean;
}

export type ProxyStatus = 'pending' | 'generating' | 'ready' | 'failed';

export interface MediaItem {
  id: string;
  source_path: string;
  proxy_path: string | null;
  proxy_status: ProxyStatus;
  probe: Probe | null;
}

export interface VideoClip {
  id: string;
  media_id: string;
  source_in_ms: number;
  source_out_ms: number;
  timeline_start_ms: number;
  volume: number;
  muted: boolean;
  transition_in: TransitionSpec;
  transition_out: TransitionSpec;
}

export interface AudioClip {
  id: string;
  media_id: string;
  source_in_ms: number;
  source_out_ms: number;
  timeline_start_ms: number;
  volume: number;
  fade_in_ms: number;
  fade_out_ms: number;
}

export type TextAnchor =
  | 'tl' | 'tc' | 'tr' | 'ml' | 'mc' | 'mr' | 'bl' | 'bc' | 'br';

export interface TextStyle {
  font_family: string;
  size_px: number;
  color: string;
  weight: number;
  bg_color?: string;
  bg_opacity?: number;
}

export interface TextPosition {
  x_pct: number;
  y_pct: number;
  anchor: TextAnchor;
}

export type TextKind = 'title' | 'caption';

export interface TextClip {
  id: string;
  text: string;
  timeline_start_ms: number;
  duration_ms: number;
  style: TextStyle;
  position: TextPosition;
  kind: TextKind;
}

export interface Timeline {
  duration_ms: number;
  video_track: VideoClip[];
  audio_track: AudioClip[];
  text_track: TextClip[];
}

export interface Project {
  version: string;
  name: string;
  created_at: string;
  modified_at: string;
  output_settings: OutputSettings;
  media_pool: MediaItem[];
  timeline: Timeline;
}

export interface RecentProject {
  path: string;
  name: string;
  last_opened: string;
}
```

- [ ] **Step 2: Verify TS compiles.**

Run:
```bash
cd videoeditor && npx svelte-check
```

Expected: 0 errors.

- [ ] **Step 3: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/types.ts
git commit -m "feat(videoeditor): typescript type mirrors"
```

---

### Task 12: IPC wrapper

**Files:**
- Create: `videoeditor/src/lib/ipc.ts`
- Create: `videoeditor/tests/frontend/ipc.test.ts`

- [ ] **Step 1: Write the failing test.**

Create `videoeditor/tests/frontend/ipc.test.ts`:

```ts
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
```

- [ ] **Step 2: Write `videoeditor/src/lib/ipc.ts`.**

```ts
import { invoke } from '@tauri-apps/api/core';
import type { Project, RecentProject } from './types';

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
};
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor && npm test -- ipc
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/ipc.ts videoeditor/tests/frontend/ipc.test.ts
git commit -m "feat(videoeditor): typed ipc wrapper"
```

---

### Task 13: Project store

**Files:**
- Create: `videoeditor/src/lib/stores/projectStore.ts`
- Create: `videoeditor/tests/frontend/projectStore.test.ts`

- [ ] **Step 1: Write the failing test.**

Create `videoeditor/tests/frontend/projectStore.test.ts`:

```ts
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
    mockInvoke.mockResolvedValueOnce(sampleProject); // newProject
    await projectActions.newProject('S');
    projectActions.markDirty();
    expect(get(projectStore).dirty).toBe(true);

    mockInvoke.mockResolvedValueOnce(null); // save
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
```

- [ ] **Step 2: Write `videoeditor/src/lib/stores/projectStore.ts`.**

```ts
import { writable, get } from 'svelte/store';
import { ipc } from '$lib/ipc';
import type { Project } from '$lib/types';

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
};
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd videoeditor && npm test -- projectStore
```

Expected: 6 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/stores/projectStore.ts videoeditor/tests/frontend/projectStore.test.ts
git commit -m "feat(videoeditor): project store"
```

---

### Task 14: Pane layout (empty placeholder components)

**Files:**
- Create: `videoeditor/src/lib/components/MediaPool.svelte`
- Create: `videoeditor/src/lib/components/PreviewPlayer.svelte`
- Create: `videoeditor/src/lib/components/Inspector.svelte`
- Create: `videoeditor/src/lib/components/Timeline.svelte`
- Create: `videoeditor/src/lib/components/PaneLayout.svelte`

- [ ] **Step 1: Write the placeholder components.**

Each placeholder follows the same shape. Write them all (no shared abstraction yet — YAGNI):

`videoeditor/src/lib/components/MediaPool.svelte`:
```svelte
<aside class="pane media-pool">
  <h2>Media</h2>
  <p class="placeholder">No media imported yet.</p>
</aside>

<style>
  .pane { padding: 0.5rem 0.75rem; overflow: auto; }
  .pane h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.7; margin: 0 0 0.5rem; }
  .placeholder { font-size: 0.875rem; opacity: 0.6; }
</style>
```

`videoeditor/src/lib/components/PreviewPlayer.svelte`:
```svelte
<section class="pane preview">
  <div class="canvas">
    <p class="placeholder">No preview.</p>
  </div>
</section>

<style>
  .pane { display: flex; flex-direction: column; }
  .canvas { flex: 1; display: grid; place-items: center; background: #0a0a0a; }
  .placeholder { opacity: 0.5; }
</style>
```

`videoeditor/src/lib/components/Inspector.svelte`:
```svelte
<section class="pane inspector">
  <h2>Inspector</h2>
  <p class="placeholder">Nothing selected.</p>
</section>

<style>
  .pane { padding: 0.5rem 0.75rem; overflow: auto; }
  .pane h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.7; margin: 0 0 0.5rem; }
  .placeholder { font-size: 0.875rem; opacity: 0.6; }
</style>
```

`videoeditor/src/lib/components/Timeline.svelte`:
```svelte
<section class="pane timeline">
  <p class="placeholder">Timeline</p>
</section>

<style>
  .pane { padding: 0.5rem 0.75rem; background: #111; }
  .placeholder { opacity: 0.5; }
</style>
```

`videoeditor/src/lib/components/PaneLayout.svelte`:
```svelte
<script lang="ts">
  import MediaPool from './MediaPool.svelte';
  import PreviewPlayer from './PreviewPlayer.svelte';
  import Inspector from './Inspector.svelte';
  import Timeline from './Timeline.svelte';
</script>

<div class="layout">
  <div class="top">
    <MediaPool />
    <PreviewPlayer />
  </div>
  <Inspector />
  <Timeline />
</div>

<style>
  .layout {
    display: grid;
    grid-template-rows: 1fr auto auto;
    height: 100%;
    min-height: 0;
  }
  .top {
    display: grid;
    grid-template-columns: 280px 1fr;
    min-height: 0;
    border-bottom: 1px solid #2a2a2a;
  }
  .top > :global(.pane:first-child) {
    border-right: 1px solid #2a2a2a;
  }
  :global(.layout > .pane) {
    border-bottom: 1px solid #2a2a2a;
  }
</style>
```

- [ ] **Step 2: Wire `PaneLayout` into `App.svelte` (replacing the placeholder content).**

Edit `videoeditor/src/App.svelte`:

```svelte
<script lang="ts">
  import PaneLayout from '$lib/components/PaneLayout.svelte';
</script>

<div class="app">
  <PaneLayout />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-rows: 1fr;
  }
</style>
```

- [ ] **Step 3: Verify TS compiles.**

Run:
```bash
cd videoeditor && npx svelte-check
```

Expected: 0 errors.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/components/MediaPool.svelte videoeditor/src/lib/components/PreviewPlayer.svelte videoeditor/src/lib/components/Inspector.svelte videoeditor/src/lib/components/Timeline.svelte videoeditor/src/lib/components/PaneLayout.svelte videoeditor/src/App.svelte
git commit -m "feat(videoeditor): pane layout with placeholder components"
```

---

### Task 15: Menu bar with project actions

**Files:**
- Create: `videoeditor/src/lib/components/MenuBar.svelte`
- Create: `videoeditor/src/lib/dialogs/NewProjectDialog.svelte`
- Create: `videoeditor/src/lib/dialogs/RecentProjectsDialog.svelte`
- Modify: `videoeditor/src/App.svelte`

- [ ] **Step 1: Write `NewProjectDialog.svelte`.**

`videoeditor/src/lib/dialogs/NewProjectDialog.svelte`:

```svelte
<script lang="ts">
  interface Props {
    open: boolean;
    onCreate: (name: string) => void;
    onCancel: () => void;
  }
  let { open, onCreate, onCancel }: Props = $props();
  let name = $state('Untitled');

  function submit(e: Event) {
    e.preventDefault();
    if (name.trim().length === 0) return;
    onCreate(name.trim());
  }
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={onCancel}>
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="new-title" onclick={(e) => e.stopPropagation()}>
      <h2 id="new-title">New project</h2>
      <form onsubmit={submit}>
        <label>
          <span>Name</span>
          <input bind:value={name} autofocus />
        </label>
        <div class="actions">
          <button type="button" onclick={onCancel}>Cancel</button>
          <button type="submit">Create</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: grid; place-items: center; }
  .modal { background: #1f1f1f; padding: 1.25rem; border-radius: 8px; min-width: 360px; }
  h2 { margin: 0 0 0.75rem; font-size: 1rem; }
  label { display: grid; gap: 0.25rem; margin-bottom: 1rem; }
  input { padding: 0.5rem; background: #2a2a2a; border: 1px solid #444; color: inherit; border-radius: 4px; }
  .actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
  button { padding: 0.4rem 0.9rem; border-radius: 4px; border: 1px solid #555; background: #2a2a2a; color: inherit; cursor: pointer; }
  button[type="submit"] { background: #2563eb; border-color: #2563eb; }
</style>
```

- [ ] **Step 2: Write `RecentProjectsDialog.svelte`.**

`videoeditor/src/lib/dialogs/RecentProjectsDialog.svelte`:

```svelte
<script lang="ts">
  import type { RecentProject } from '$lib/types';

  interface Props {
    open: boolean;
    items: RecentProject[];
    onPick: (path: string) => void;
    onCancel: () => void;
  }
  let { open, items, onPick, onCancel }: Props = $props();
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={onCancel}>
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="recent-title" onclick={(e) => e.stopPropagation()}>
      <h2 id="recent-title">Recent projects</h2>
      {#if items.length === 0}
        <p class="empty">No recent projects.</p>
      {:else}
        <ul>
          {#each items as item}
            <li>
              <button type="button" onclick={() => onPick(item.path)}>
                <span class="name">{item.name}</span>
                <span class="path">{item.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="actions">
        <button type="button" onclick={onCancel}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: grid; place-items: center; }
  .modal { background: #1f1f1f; padding: 1.25rem; border-radius: 8px; min-width: 480px; max-width: 720px; }
  h2 { margin: 0 0 0.75rem; font-size: 1rem; }
  .empty { opacity: 0.6; }
  ul { list-style: none; padding: 0; margin: 0 0 1rem; max-height: 50vh; overflow: auto; }
  li button {
    display: grid;
    grid-template-columns: 1fr;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 0;
    color: inherit;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    cursor: pointer;
  }
  li button:hover { background: #2a2a2a; }
  .name { font-weight: 600; }
  .path { font-size: 0.8rem; opacity: 0.6; }
  .actions { display: flex; justify-content: flex-end; }
  .actions button { padding: 0.4rem 0.9rem; border-radius: 4px; border: 1px solid #555; background: #2a2a2a; color: inherit; cursor: pointer; }
</style>
```

- [ ] **Step 3: Write `MenuBar.svelte`.**

`videoeditor/src/lib/components/MenuBar.svelte`:

```svelte
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
```

- [ ] **Step 4: Wire MenuBar into `App.svelte`.**

Edit `videoeditor/src/App.svelte`:

```svelte
<script lang="ts">
  import MenuBar from '$lib/components/MenuBar.svelte';
  import PaneLayout from '$lib/components/PaneLayout.svelte';
</script>

<div class="app">
  <MenuBar />
  <PaneLayout />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-rows: auto 1fr;
  }
</style>
```

- [ ] **Step 5: Verify TS compiles.**

Run:
```bash
cd videoeditor && npx svelte-check
```

Expected: 0 errors.

- [ ] **Step 6: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/components/MenuBar.svelte videoeditor/src/lib/dialogs/NewProjectDialog.svelte videoeditor/src/lib/dialogs/RecentProjectsDialog.svelte videoeditor/src/App.svelte
git commit -m "feat(videoeditor): menu bar with project actions"
```

---

### Task 16: Manual end-to-end smoke test

**Files:** none — verification only.

- [ ] **Step 1: Run the dev server.**

Run:
```bash
cd videoeditor && npm run tauri dev
```

Expected: an app window opens.

- [ ] **Step 2: Smoke test the project lifecycle.**

In the running app, manually verify:

1. Click **File → New**. The new-project dialog opens. Type a name, click **Create**. The status area shows the project name with no dirty indicator.
2. Click **File → Save As…**. A native file dialog opens. Choose a path inside `~/Documents/` (or any writable dir) named `test.vproj`. Click Save. The status area shows the project name.
3. Verify the file exists:
   ```bash
   cat ~/Documents/test.vproj | python3 -m json.tool
   ```
   Expected: well-formed JSON with `version: "1"`, your project name, empty `media_pool`, empty timeline tracks.

4. Click **File → New** again to create a different project. The status updates.
5. Click **File → Recent…**. The dialog lists `test.vproj`. Click it. The status updates back to the loaded project's name.
6. Manually corrupt the file:
   ```bash
   echo "garbage" > ~/Documents/test.vproj
   ```
   In the app, **File → Open…**, pick that file. Expected: an error surfaces (currently as a console error in the webview — wired-up UI error display lands in a future plan; for now confirm via DevTools console with Ctrl+Shift+I).

- [ ] **Step 3: Close the app.**

Ctrl+C in the terminal to stop the dev server.

- [ ] **Step 4: No commit needed (verification step). If you discovered any bugs, file them and address before continuing to subsequent plans.**

---

### Task 17: Final verification + plan close-out

**Files:** none — verification only.

- [ ] **Step 1: Run all Rust tests.**

Run:
```bash
cd videoeditor/src-tauri && cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run all frontend tests.**

Run:
```bash
cd videoeditor && npm test
```

Expected: all tests pass.

- [ ] **Step 3: Run TypeScript check.**

Run:
```bash
cd videoeditor && npx svelte-check
```

Expected: 0 errors.

- [ ] **Step 4: Run a release build.**

Run:
```bash
cd videoeditor && npm run build
cd src-tauri && cargo build --release
```

Expected: both succeed. Release binary at `videoeditor/src-tauri/target/release/videoeditor`.

- [ ] **Step 5: Update plan status.**

Edit this plan file's first lines if you want — append to the goal: `(complete YYYY-MM-DD)`.

- [ ] **Step 6: Commit any final cleanup.**

```bash
cd /home/mthomas/data/git/claude-apps
git status
# If anything is untracked or modified, address it. Otherwise nothing to commit.
```

---

## What ships at the end of this plan

- A launchable Tauri+Svelte desktop app titled **VideoEditor**
- Three-pane layout (placeholder content) + working menu bar
- Working **New / Open / Save / Save As / Recent…** for `.vproj` files
- Project file format defined and round-trip-tested
- Recent-projects registry persisted at `$XDG_CONFIG_HOME/videoeditor/recent.json`
- Rust core: 25+ unit tests + 4 integration tests, exercising data model, IO, and recent-projects logic
- Frontend logic: 10 Vitest tests covering IPC and the project store
- A release build that produces a single binary

## What's NOT in this plan (intentionally — covered by later plans)

- Media import / probing
- Proxy generation
- Timeline editing UI and reducer logic
- Preview playback
- Transitions, text overlays
- Export pipeline
- Autosave (deferred — first surfaces in plan 7 / Polish)
- Error UX modals (errors currently bubble to console)
- Keyboard shortcuts (other than browser defaults)
- Packaging (AppImage, .desktop, file association)

## Next plan

**Plan 2 — Media + proxy pipeline.** Builds on this foundation by adding `import_media`, `ffprobe`-based metadata, the proxy worker, and a real MediaPool component.
