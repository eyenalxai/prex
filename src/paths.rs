use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use std::path::PathBuf;

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "prex", "prex")
        .ok_or_else(|| anyhow!("Unable to resolve data directory"))
}

pub fn data_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().to_path_buf())
}

pub fn logs_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().join("logs"))
}
