use uuid::Uuid;

use super::timeline::Timeline;
use crate::error::AppResult;

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

pub fn insert_clip(
    _timeline: &Timeline,
    _track: Track,
    _media_id: Uuid,
    _timeline_start_ms: u64,
    _source_in_ms: u64,
    _source_out_ms: u64,
) -> AppResult<Timeline> {
    unimplemented!()
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
