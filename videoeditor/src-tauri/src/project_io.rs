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
