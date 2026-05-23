use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::ffmpeg::probe::probe_file;
use crate::ffmpeg::proxy::proxy_path_for;
use crate::ffmpeg::thumbnails::thumbnails_dir_for;
use crate::ffmpeg::waveform::waveform_path_for;
use crate::media_repo::MediaRepo;
use crate::model::project::{MediaItem, Project};
use crate::model::timeline::Timeline;
use crate::model::timeline_ops::{
    delete_clip, insert_clip, move_clip, split_clip, trim_clip, SnapConfig, Track,
};
use crate::paths::{ensure_dir, proxies_dir, recent_file_path, thumbnails_dir, waveforms_dir};
use crate::project_io::{load_project, save_project};
use crate::proxy_worker::{ProxyJob, ProxyWorkerHandle};
use crate::recent::{RecentProject, RecentRegistry};

fn parse_track(s: &str) -> AppResult<Track> {
    match s {
        "video" => Ok(Track::Video),
        "audio" => Ok(Track::Audio),
        other => Err(AppError::Validation {
            message: format!("invalid track: {other}"),
        }),
    }
}

fn snap_config(enabled: bool, threshold_ms: Option<u64>) -> SnapConfig {
    let default = SnapConfig::default();
    SnapConfig {
        enabled,
        threshold_ms: threshold_ms.unwrap_or(default.threshold_ms),
    }
}

#[tauri::command]
pub fn new_project(name: String) -> AppResult<Project> {
    Ok(Project::new(name))
}

#[tauri::command]
pub fn open_project(path: String, repo: State<'_, Arc<MediaRepo>>) -> AppResult<Project> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_absolute() {
        return Err(AppError::InvalidPath(format!("not absolute: {path}")));
    }
    let project = load_project(&path_buf)?;

    repo.reconcile_from_project(&project)?;

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

#[tauri::command]
pub async fn import_media(
    paths: Vec<String>,
    repo: State<'_, Arc<MediaRepo>>,
    worker: State<'_, ProxyWorkerHandle>,
) -> AppResult<Vec<MediaItem>> {
    let proxies_root = proxies_dir()?;
    let thumbs_root = thumbnails_dir()?;
    let waves_root = waveforms_dir()?;
    ensure_dir(&proxies_root)?;
    ensure_dir(&thumbs_root)?;
    ensure_dir(&waves_root)?;

    let mut imported = Vec::with_capacity(paths.len());
    for path_str in paths {
        let path_buf = PathBuf::from(&path_str);
        if !path_buf.is_absolute() {
            return Err(AppError::InvalidPath(format!("not absolute: {path_str}")));
        }

        let probe = probe_file(&path_buf).await?;
        let mut item = repo.add_pending(path_str.clone())?;
        repo.set_probe(item.id, probe.clone())?;
        item.probe = Some(probe.clone());

        let job = ProxyJob {
            media_id: item.id,
            source_path: path_buf,
            proxy_path: proxy_path_for(&proxies_root, &item.id.to_string()),
            thumbnails_dir: thumbnails_dir_for(&thumbs_root, &item.id.to_string()),
            waveform_path: waveform_path_for(&waves_root, &item.id.to_string()),
            has_audio: probe.has_audio,
            duration_ms: probe.duration_ms,
        };
        worker.enqueue(job).map_err(AppError::InvalidPath)?;

        imported.push(item);
    }
    Ok(imported)
}

#[tauri::command]
pub fn delete_media(id: String, repo: State<'_, Arc<MediaRepo>>) -> AppResult<()> {
    let uuid =
        Uuid::parse_str(&id).map_err(|e| AppError::InvalidPath(format!("invalid uuid: {e}")))?;
    repo.remove(uuid)?;
    Ok(())
}

#[tauri::command]
pub fn list_media(repo: State<'_, Arc<MediaRepo>>) -> AppResult<Vec<MediaItem>> {
    repo.list()
}

#[tauri::command]
pub fn timeline_insert_clip(
    timeline: Timeline,
    track: String,
    media_id: Uuid,
    timeline_start_ms: u64,
    source_in_ms: u64,
    source_out_ms: u64,
) -> AppResult<Timeline> {
    let track = parse_track(&track)?;
    insert_clip(
        &timeline,
        track,
        media_id,
        timeline_start_ms,
        source_in_ms,
        source_out_ms,
    )
}

#[tauri::command]
pub fn timeline_move_clip(
    timeline: Timeline,
    track: String,
    clip_id: Uuid,
    new_start_ms: u64,
    snap_enabled: bool,
    snap_threshold_ms: Option<u64>,
) -> AppResult<Timeline> {
    let track = parse_track(&track)?;
    move_clip(
        &timeline,
        track,
        clip_id,
        new_start_ms,
        snap_config(snap_enabled, snap_threshold_ms),
    )
}

#[tauri::command]
pub fn timeline_trim_clip(
    timeline: Timeline,
    track: String,
    clip_id: Uuid,
    new_source_in_ms: u64,
    new_source_out_ms: u64,
    snap_enabled: bool,
    snap_threshold_ms: Option<u64>,
) -> AppResult<Timeline> {
    let track = parse_track(&track)?;
    trim_clip(
        &timeline,
        track,
        clip_id,
        new_source_in_ms,
        new_source_out_ms,
        snap_config(snap_enabled, snap_threshold_ms),
    )
}

#[tauri::command]
pub fn timeline_split_clip(
    timeline: Timeline,
    track: String,
    clip_id: Uuid,
    at_timeline_ms: u64,
) -> AppResult<Timeline> {
    let track = parse_track(&track)?;
    split_clip(&timeline, track, clip_id, at_timeline_ms)
}

#[tauri::command]
pub fn timeline_delete_clip(
    timeline: Timeline,
    track: String,
    clip_id: Uuid,
) -> AppResult<Timeline> {
    let track = parse_track(&track)?;
    delete_clip(&timeline, track, clip_id)
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
    fn save_project_rejects_relative_path() {
        let p = Project::new("X".into());
        let err = save_project_cmd(p, "relative/path.vproj".into()).unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }
}
