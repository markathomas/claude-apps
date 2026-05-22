use std::path::PathBuf;

use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProxyJob {
    pub media_id: Uuid,
    pub source_path: PathBuf,
    pub proxy_path: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub waveform_path: PathBuf,
    pub has_audio: bool,
    pub duration_ms: u64,
}

#[derive(Clone)]
pub struct ProxyWorkerHandle {
    sender: mpsc::UnboundedSender<ProxyJob>,
}

impl ProxyWorkerHandle {
    pub fn new(sender: mpsc::UnboundedSender<ProxyJob>) -> Self {
        Self { sender }
    }

    pub fn enqueue(&self, job: ProxyJob) -> Result<(), String> {
        self.sender.send(job).map_err(|e| e.to_string())
    }
}

pub fn channel() -> (ProxyWorkerHandle, mpsc::UnboundedReceiver<ProxyJob>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ProxyWorkerHandle::new(tx), rx)
}

use std::sync::Arc;

use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::ffmpeg::progress::ProgressParser;
use crate::ffmpeg::proxy::build_proxy_args;
use crate::ffmpeg::thumbnails::build_thumbnails_args;
use crate::ffmpeg::waveform::{build_waveform_args, compute_peaks, Waveform, WAVEFORM_BUCKET_MS};
use crate::media_repo::MediaRepo;
use crate::model::project::ProxyStatus;

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, name: &str, payload: serde_json::Value);
}

#[derive(Serialize)]
struct ProxyProgressPayload {
    media_id: String,
    percent: f32,
}

#[derive(Serialize)]
struct ProxyReadyPayload {
    media_id: String,
    proxy_path: String,
    thumbnails_dir: String,
    waveform_path: String,
}

#[derive(Serialize)]
struct ProxyFailedPayload {
    media_id: String,
    reason: String,
}

pub async fn run_worker_loop(
    mut rx: mpsc::UnboundedReceiver<ProxyJob>,
    repo: Arc<MediaRepo>,
    emitter: Arc<dyn EventEmitter>,
) {
    while let Some(job) = rx.recv().await {
        let _ = repo.set_proxy_status(job.media_id, ProxyStatus::Generating, None);

        if let Err(e) = process_job(&job, &repo, emitter.as_ref()).await {
            let _ = repo.set_proxy_status(job.media_id, ProxyStatus::Failed, None);
            emitter.emit(
                "proxy_failed",
                serde_json::to_value(ProxyFailedPayload {
                    media_id: job.media_id.to_string(),
                    reason: e,
                })
                .unwrap(),
            );
        }
    }
}

async fn process_job(
    job: &ProxyJob,
    repo: &MediaRepo,
    emitter: &dyn EventEmitter,
) -> Result<(), String> {
    if let Some(parent) = job.proxy_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&job.thumbnails_dir).map_err(|e| e.to_string())?;
    if let Some(parent) = job.waveform_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    run_proxy(job, emitter).await?;
    run_thumbnails(job).await?;
    if job.has_audio {
        run_waveform(job).await?;
    } else {
        std::fs::write(
            &job.waveform_path,
            serde_json::to_string(&Waveform {
                bucket_ms: WAVEFORM_BUCKET_MS,
                peaks: vec![],
            })
            .unwrap(),
        )
        .map_err(|e| e.to_string())?;
    }

    repo.set_proxy_status(
        job.media_id,
        ProxyStatus::Ready,
        Some(job.proxy_path.to_string_lossy().to_string()),
    )
    .map_err(|e| e.to_string())?;

    emitter.emit(
        "proxy_ready",
        serde_json::to_value(ProxyReadyPayload {
            media_id: job.media_id.to_string(),
            proxy_path: job.proxy_path.to_string_lossy().to_string(),
            thumbnails_dir: job.thumbnails_dir.to_string_lossy().to_string(),
            waveform_path: job.waveform_path.to_string_lossy().to_string(),
        })
        .unwrap(),
    );
    Ok(())
}

async fn run_proxy(job: &ProxyJob, emitter: &dyn EventEmitter) -> Result<(), String> {
    let args = build_proxy_args(&job.source_path, &job.proxy_path);
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args);
    cmd.stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "no stderr".to_string())?;
    let mut reader = BufReader::new(stderr).lines();
    let mut parser = ProgressParser::default();

    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(progress) = parser.feed(&line) {
            let percent = if job.duration_ms == 0 {
                0.0
            } else {
                (progress.out_time_ms as f32 / job.duration_ms as f32 * 100.0).clamp(0.0, 100.0)
            };
            emitter.emit(
                "proxy_progress",
                serde_json::to_value(ProxyProgressPayload {
                    media_id: job.media_id.to_string(),
                    percent,
                })
                .unwrap(),
            );
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("proxy ffmpeg exited with {status}"));
    }
    Ok(())
}

async fn run_thumbnails(job: &ProxyJob) -> Result<(), String> {
    let args = build_thumbnails_args(&job.source_path, &job.thumbnails_dir);
    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .await
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("thumbnails ffmpeg exited with {status}"));
    }
    Ok(())
}

async fn run_waveform(job: &ProxyJob) -> Result<(), String> {
    let tmp = job.waveform_path.with_extension("pcm");
    let args = build_waveform_args(&job.source_path, &tmp);
    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .await
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("waveform ffmpeg exited with {status}"));
    }
    let pcm = std::fs::read(&tmp).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp);
    let peaks = compute_peaks(&pcm, 8000, WAVEFORM_BUCKET_MS);
    let waveform = Waveform {
        bucket_ms: WAVEFORM_BUCKET_MS,
        peaks,
    };
    std::fs::write(
        &job.waveform_path,
        serde_json::to_string(&waveform).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job() -> ProxyJob {
        ProxyJob {
            media_id: Uuid::new_v4(),
            source_path: PathBuf::from("/in.mp4"),
            proxy_path: PathBuf::from("/cache/p.mp4"),
            thumbnails_dir: PathBuf::from("/cache/thumbs/id"),
            waveform_path: PathBuf::from("/cache/wf/id.json"),
            has_audio: true,
            duration_ms: 5000,
        }
    }

    #[tokio::test]
    async fn enqueue_delivers_job_to_receiver() {
        let (handle, mut rx) = channel();
        let job = sample_job();
        handle.enqueue(job.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.media_id, job.media_id);
        assert_eq!(received.source_path, job.source_path);
    }

    #[tokio::test]
    async fn enqueue_fails_after_receiver_dropped() {
        let (handle, rx) = channel();
        drop(rx);
        let result = handle.enqueue(sample_job());
        assert!(result.is_err());
    }
}
