use std::path::{Path, PathBuf};
use crate::{
    mediainfo::MediaInfo,
    naming::{format_name, TokenValues},
    sources::MediaMatch,
    Result,
};

pub struct RenameJob {
    pub source: PathBuf,
    pub media_match: MediaMatch,
    pub media_info: MediaInfo,
    pub template: String,
}

pub fn build_new_path(job: &RenameJob) -> Result<PathBuf> {
    let values = TokenValues::from_match_and_info(&job.media_match, &job.media_info);
    let new_name = format_name(&job.template, &values)?;
    let dir = job.source.parent().unwrap_or_else(|| Path::new(""));
    Ok(dir.join(new_name))
}

pub fn execute_rename(job: &RenameJob) -> Result<PathBuf> {
    let new_path = build_new_path(job)?;
    std::fs::rename(&job.source, &new_path)?;
    Ok(new_path)
}
