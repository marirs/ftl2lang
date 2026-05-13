use crate::error::AppError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn collect_ftl_files(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("ftl")
        {
            out.push(entry.path().to_path_buf());
        }
    }
    Ok(out)
}

pub fn target_path_for(file: &Path, source_root: &Path, target_root: &Path) -> PathBuf {
    let rel = file.strip_prefix(source_root).unwrap_or(file);
    target_root.join(rel)
}
