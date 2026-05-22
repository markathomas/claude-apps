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

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_corrupt_error_displays_message() {
        let e = AppError::ProjectCorrupt {
            message: "expected }".into(),
        };
        assert_eq!(e.to_string(), "project file is corrupt: expected }");
    }

    #[test]
    fn unsupported_version_error_displays_both_versions() {
        let e = AppError::UnsupportedVersion {
            found: "2".into(),
            supported: "1".into(),
        };
        assert_eq!(
            e.to_string(),
            "unsupported project version: 2 (supported: 1)"
        );
    }

    #[test]
    fn app_error_serializes_to_string_message() {
        let e = AppError::InvalidPath("not absolute".into());
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, "\"invalid path: not absolute\"");
    }
}
