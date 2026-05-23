use uuid::Uuid;

use super::timeline::{AudioClip, Timeline, VideoClip};
use super::transition::TransitionSpec;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapConfig {
    pub enabled: bool,
    pub threshold_ms: u64,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_ms: 166,
        }
    }
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::Validation {
        message: message.into(),
    }
}

fn span(timeline: &Timeline) -> u64 {
    let v = timeline
        .video_track
        .iter()
        .map(|c| c.timeline_start_ms + (c.source_out_ms - c.source_in_ms))
        .max()
        .unwrap_or(0);
    let a = timeline
        .audio_track
        .iter()
        .map(|c| c.timeline_start_ms + (c.source_out_ms - c.source_in_ms))
        .max()
        .unwrap_or(0);
    let t = timeline
        .text_track
        .iter()
        .map(|c| c.timeline_start_ms + c.duration_ms)
        .max()
        .unwrap_or(0);
    v.max(a).max(t)
}

fn overlaps(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    a_start < b_end && b_start < a_end
}

fn check_no_overlap_video(track: &[VideoClip], start: u64, end: u64, ignore: Option<Uuid>) -> AppResult<()> {
    for clip in track {
        if Some(clip.id) == ignore {
            continue;
        }
        let clip_end = clip.timeline_start_ms + (clip.source_out_ms - clip.source_in_ms);
        if overlaps(start, end, clip.timeline_start_ms, clip_end) {
            return Err(invalid("clip overlaps an existing clip on the same track"));
        }
    }
    Ok(())
}

fn check_no_overlap_audio(track: &[AudioClip], start: u64, end: u64, ignore: Option<Uuid>) -> AppResult<()> {
    for clip in track {
        if Some(clip.id) == ignore {
            continue;
        }
        let clip_end = clip.timeline_start_ms + (clip.source_out_ms - clip.source_in_ms);
        if overlaps(start, end, clip.timeline_start_ms, clip_end) {
            return Err(invalid("clip overlaps an existing clip on the same track"));
        }
    }
    Ok(())
}

pub fn insert_clip(
    timeline: &Timeline,
    track: Track,
    media_id: Uuid,
    timeline_start_ms: u64,
    source_in_ms: u64,
    source_out_ms: u64,
) -> AppResult<Timeline> {
    if source_out_ms <= source_in_ms {
        return Err(invalid("source_out_ms must be greater than source_in_ms"));
    }
    let length = source_out_ms - source_in_ms;
    let end = timeline_start_ms + length;

    let mut new_timeline = timeline.clone();

    match track {
        Track::Video => {
            check_no_overlap_video(&new_timeline.video_track, timeline_start_ms, end, None)?;
            new_timeline.video_track.push(VideoClip {
                id: Uuid::new_v4(),
                media_id,
                source_in_ms,
                source_out_ms,
                timeline_start_ms,
                volume: 1.0,
                muted: false,
                transition_in: TransitionSpec::default(),
                transition_out: TransitionSpec::default(),
            });
            new_timeline
                .video_track
                .sort_by_key(|c| c.timeline_start_ms);
        }
        Track::Audio => {
            check_no_overlap_audio(&new_timeline.audio_track, timeline_start_ms, end, None)?;
            new_timeline.audio_track.push(AudioClip {
                id: Uuid::new_v4(),
                media_id,
                source_in_ms,
                source_out_ms,
                timeline_start_ms,
                volume: 1.0,
                fade_in_ms: 0,
                fade_out_ms: 0,
            });
            new_timeline
                .audio_track
                .sort_by_key(|c| c.timeline_start_ms);
        }
    }

    new_timeline.duration_ms = span(&new_timeline);
    Ok(new_timeline)
}

pub fn move_clip(
    _timeline: &Timeline,
    _track: Track,
    _clip_id: Uuid,
    _new_start_ms: u64,
    _snap: SnapConfig,
) -> AppResult<Timeline> {
    unimplemented!()
}

pub fn trim_clip(
    _timeline: &Timeline,
    _track: Track,
    _clip_id: Uuid,
    _new_source_in_ms: u64,
    _new_source_out_ms: u64,
    _snap: SnapConfig,
) -> AppResult<Timeline> {
    unimplemented!()
}

pub fn split_clip(
    _timeline: &Timeline,
    _track: Track,
    _clip_id: Uuid,
    _at_timeline_ms: u64,
) -> AppResult<Timeline> {
    unimplemented!()
}

pub fn delete_clip(
    _timeline: &Timeline,
    _track: Track,
    _clip_id: Uuid,
) -> AppResult<Timeline> {
    unimplemented!()
}

#[cfg(test)]
mod insert_tests {
    use super::*;

    fn empty() -> Timeline {
        Timeline::default()
    }

    #[test]
    fn inserts_into_empty_video_track() {
        let t = insert_clip(&empty(), Track::Video, Uuid::new_v4(), 0, 0, 1000).unwrap();
        assert_eq!(t.video_track.len(), 1);
        assert_eq!(t.video_track[0].timeline_start_ms, 0);
        assert_eq!(t.duration_ms, 1000);
    }

    #[test]
    fn rejects_zero_length_clip() {
        let r = insert_clip(&empty(), Track::Video, Uuid::new_v4(), 0, 500, 500);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_inverted_in_out() {
        let r = insert_clip(&empty(), Track::Video, Uuid::new_v4(), 0, 500, 100);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_overlap_on_same_track() {
        let t = insert_clip(&empty(), Track::Video, Uuid::new_v4(), 0, 0, 1000).unwrap();
        let r = insert_clip(&t, Track::Video, Uuid::new_v4(), 500, 0, 1000);
        assert!(r.is_err());
    }

    #[test]
    fn allows_back_to_back() {
        let t = insert_clip(&empty(), Track::Video, Uuid::new_v4(), 0, 0, 1000).unwrap();
        let t = insert_clip(&t, Track::Video, Uuid::new_v4(), 1000, 0, 500).unwrap();
        assert_eq!(t.video_track.len(), 2);
        assert_eq!(t.duration_ms, 1500);
    }

    #[test]
    fn input_timeline_is_not_mutated() {
        let original = empty();
        let _ = insert_clip(&original, Track::Video, Uuid::new_v4(), 0, 0, 1000).unwrap();
        assert_eq!(original.video_track.len(), 0);
    }
}
