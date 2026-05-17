use crate::error::AppError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Walk `root` and return every `.ftl` file found beneath it.
///
/// Symlinks are deliberately NOT followed: a symlink inside the source tree
/// pointing outside it would otherwise let the walker pull in arbitrary
/// files, and `target_path_for` would then write them into the target tree
/// at attacker-influenced relative paths. Traversal errors (permission
/// denied, broken symlink, etc.) are reported to stderr and skipped rather
/// than silently dropped, so the user can tell when something was unreadable.
pub fn collect_ftl_files(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        match entry {
            Ok(e) => {
                if e.file_type().is_file()
                    && e.path().extension().and_then(|x| x.to_str()) == Some("ftl")
                {
                    out.push(e.path().to_path_buf());
                }
            }
            Err(err) => {
                // Don't fail the whole run for one unreadable directory;
                // warn and continue. The path may be absent from err if
                // walkdir couldn't even get that far.
                let where_at = err
                    .path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "<unknown>".into());
                eprintln!("warning: skipping {}: {}", where_at, err);
            }
        }
    }
    Ok(out)
}

/// Map a source file to its target location, preserving the relative layout
/// under `source_root`. Returns an error if `file` is not under
/// `source_root` — a defensive check that prevents an unexpected absolute
/// path from being joined onto `target_root` (which would write outside
/// the intended target tree).
pub fn target_path_for(
    file: &Path,
    source_root: &Path,
    target_root: &Path,
) -> Result<PathBuf, AppError> {
    let rel = file.strip_prefix(source_root).map_err(|_| {
        AppError::Other(format!(
            "{} is not under source root {}",
            file.display(),
            source_root.display()
        ))
    })?;
    Ok(target_root.join(rel))
}
