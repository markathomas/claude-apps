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
