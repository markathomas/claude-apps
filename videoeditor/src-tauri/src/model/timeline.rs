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
    Tl,
    Tc,
    Tr,
    Ml,
    Mc,
    Mr,
    Bl,
    Bc,
    Br,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub size_px: u32,
    pub color: String,
    pub weight: u16,
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
            transition_out: TransitionSpec {
                kind: TransitionType::Fade,
                duration_ms: 500,
            },
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
            position: TextPosition {
                x_pct: 50.0,
                y_pct: 50.0,
                anchor: TextAnchor::Mc,
            },
            kind: TextKind::Title,
        };
        let json = serde_json::to_string(&clip).unwrap();
        assert!(
            !json.contains("bg_color"),
            "bg_color should be omitted when None"
        );
        let parsed: TextClip = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, clip);
    }
}
