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
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
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
