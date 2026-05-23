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

fn check_no_overlap_video(
    track: &[VideoClip],
    start: u64,
    end: u64,
    ignore: Option<Uuid>,
) -> AppResult<()> {
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

fn check_no_overlap_audio(
    track: &[AudioClip],
    start: u64,
    end: u64,
    ignore: Option<Uuid>,
) -> AppResult<()> {
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
    timeline: &Timeline,
    track: Track,
    clip_id: Uuid,
    new_start_ms: u64,
    snap: SnapConfig,
) -> AppResult<Timeline> {
    let mut new_timeline = timeline.clone();

    match track {
        Track::Video => {
            let idx = new_timeline
                .video_track
                .iter()
                .position(|c| c.id == clip_id)
                .ok_or_else(|| invalid("clip not found on video track"))?;
            let length = {
                let c = &new_timeline.video_track[idx];
                c.source_out_ms - c.source_in_ms
            };
            let candidates = video_snap_candidates(&new_timeline.video_track, clip_id, length);
            let snapped = apply_snap(new_start_ms, &candidates, snap);
            let end = snapped + length;
            check_no_overlap_video(&new_timeline.video_track, snapped, end, Some(clip_id))?;
            new_timeline.video_track[idx].timeline_start_ms = snapped;
            new_timeline
                .video_track
                .sort_by_key(|c| c.timeline_start_ms);
        }
        Track::Audio => {
            let idx = new_timeline
                .audio_track
                .iter()
                .position(|c| c.id == clip_id)
                .ok_or_else(|| invalid("clip not found on audio track"))?;
            let length = {
                let c = &new_timeline.audio_track[idx];
                c.source_out_ms - c.source_in_ms
            };
            let candidates = audio_snap_candidates(&new_timeline.audio_track, clip_id, length);
            let snapped = apply_snap(new_start_ms, &candidates, snap);
            let end = snapped + length;
            check_no_overlap_audio(&new_timeline.audio_track, snapped, end, Some(clip_id))?;
            new_timeline.audio_track[idx].timeline_start_ms = snapped;
            new_timeline
                .audio_track
                .sort_by_key(|c| c.timeline_start_ms);
        }
    }

    new_timeline.duration_ms = span(&new_timeline);
    Ok(new_timeline)
}

fn video_snap_candidates(track: &[VideoClip], ignore: Uuid, length: u64) -> Vec<u64> {
    let mut candidates = vec![0u64];
    for clip in track {
        if clip.id == ignore {
            continue;
        }
        let end = clip.timeline_start_ms + (clip.source_out_ms - clip.source_in_ms);
        candidates.push(end);
        candidates.push(clip.timeline_start_ms.saturating_sub(length));
    }
    candidates
}

fn audio_snap_candidates(track: &[AudioClip], ignore: Uuid, length: u64) -> Vec<u64> {
    let mut candidates = vec![0u64];
    for clip in track {
        if clip.id == ignore {
            continue;
        }
        let end = clip.timeline_start_ms + (clip.source_out_ms - clip.source_in_ms);
        candidates.push(end);
        candidates.push(clip.timeline_start_ms.saturating_sub(length));
    }
    candidates
}

fn apply_snap(target: u64, candidates: &[u64], snap: SnapConfig) -> u64 {
    if !snap.enabled {
        return target;
    }
    let mut best = target;
    let mut best_dist = snap.threshold_ms + 1;
    for &candidate in candidates {
        let dist = candidate.abs_diff(target);
        if dist <= snap.threshold_ms && dist < best_dist {
            best = candidate;
            best_dist = dist;
        }
    }
    best
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

pub fn delete_clip(_timeline: &Timeline, _track: Track, _clip_id: Uuid) -> AppResult<Timeline> {
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

#[cfg(test)]
mod move_tests {
    use super::*;

    fn timeline_with_clip_at(start: u64, in_ms: u64, out_ms: u64) -> (Timeline, Uuid) {
        let mut t = Timeline::default();
        let id = Uuid::new_v4();
        t.video_track.push(VideoClip {
            id,
            media_id: Uuid::new_v4(),
            source_in_ms: in_ms,
            source_out_ms: out_ms,
            timeline_start_ms: start,
            volume: 1.0,
            muted: false,
            transition_in: TransitionSpec::default(),
            transition_out: TransitionSpec::default(),
        });
        t.duration_ms = span(&t);
        (t, id)
    }

    fn add_video_clip(t: &mut Timeline, start: u64, in_ms: u64, out_ms: u64) -> Uuid {
        let id = Uuid::new_v4();
        t.video_track.push(VideoClip {
            id,
            media_id: Uuid::new_v4(),
            source_in_ms: in_ms,
            source_out_ms: out_ms,
            timeline_start_ms: start,
            volume: 1.0,
            muted: false,
            transition_in: TransitionSpec::default(),
            transition_out: TransitionSpec::default(),
        });
        t.duration_ms = span(t);
        id
    }

    #[test]
    fn moves_clip_earlier() {
        let (t, id) = timeline_with_clip_at(2000, 0, 1000);
        let snap = SnapConfig {
            enabled: false,
            threshold_ms: 0,
        };
        let r = move_clip(&t, Track::Video, id, 500, snap).unwrap();
        assert_eq!(r.video_track[0].timeline_start_ms, 500);
    }

    #[test]
    fn moves_clip_later() {
        let (t, id) = timeline_with_clip_at(500, 0, 1000);
        let snap = SnapConfig {
            enabled: false,
            threshold_ms: 0,
        };
        let r = move_clip(&t, Track::Video, id, 3000, snap).unwrap();
        assert_eq!(r.video_track[0].timeline_start_ms, 3000);
        assert_eq!(r.duration_ms, 4000);
    }

    #[test]
    fn snaps_to_neighbor_right_edge() {
        let mut t = Timeline::default();
        let _neighbor = add_video_clip(&mut t, 0, 0, 1000);
        let moving = add_video_clip(&mut t, 5000, 0, 500);
        let snap = SnapConfig {
            enabled: true,
            threshold_ms: 200,
        };
        let r = move_clip(&t, Track::Video, moving, 1100, snap).unwrap();
        let moved = r.video_track.iter().find(|c| c.id == moving).unwrap();
        assert_eq!(moved.timeline_start_ms, 1000);
    }

    #[test]
    fn snaps_to_neighbor_left_edge() {
        let mut t = Timeline::default();
        let _neighbor = add_video_clip(&mut t, 5000, 0, 1000);
        let moving = add_video_clip(&mut t, 100, 0, 500);
        let snap = SnapConfig {
            enabled: true,
            threshold_ms: 200,
        };
        let r = move_clip(&t, Track::Video, moving, 4600, snap).unwrap();
        let moved = r.video_track.iter().find(|c| c.id == moving).unwrap();
        assert_eq!(moved.timeline_start_ms, 4500);
    }

    #[test]
    fn snaps_to_timeline_zero() {
        let (t, id) = timeline_with_clip_at(2000, 0, 1000);
        let snap = SnapConfig {
            enabled: true,
            threshold_ms: 200,
        };
        let r = move_clip(&t, Track::Video, id, 100, snap).unwrap();
        assert_eq!(r.video_track[0].timeline_start_ms, 0);
    }

    #[test]
    fn no_snap_when_threshold_exceeded() {
        let (t, id) = timeline_with_clip_at(2000, 0, 1000);
        let snap = SnapConfig {
            enabled: true,
            threshold_ms: 100,
        };
        let r = move_clip(&t, Track::Video, id, 500, snap).unwrap();
        assert_eq!(r.video_track[0].timeline_start_ms, 500);
    }

    #[test]
    fn rejects_overlap_with_other_clip() {
        let mut t = Timeline::default();
        let _other = add_video_clip(&mut t, 0, 0, 1000);
        let moving = add_video_clip(&mut t, 2000, 0, 500);
        let snap = SnapConfig {
            enabled: false,
            threshold_ms: 0,
        };
        let r = move_clip(&t, Track::Video, moving, 500, snap);
        assert!(r.is_err());
    }

    #[test]
    fn snap_disabled_is_honored() {
        let mut t = Timeline::default();
        let _neighbor = add_video_clip(&mut t, 0, 0, 1000);
        let moving = add_video_clip(&mut t, 5000, 0, 500);
        let snap = SnapConfig {
            enabled: false,
            threshold_ms: 1000,
        };
        let r = move_clip(&t, Track::Video, moving, 1050, snap).unwrap();
        let moved = r.video_track.iter().find(|c| c.id == moving).unwrap();
        assert_eq!(moved.timeline_start_ms, 1050);
    }

    #[test]
    fn returns_error_when_clip_not_found() {
        let t = Timeline::default();
        let snap = SnapConfig::default();
        let r = move_clip(&t, Track::Video, Uuid::new_v4(), 0, snap);
        assert!(r.is_err());
    }
}
