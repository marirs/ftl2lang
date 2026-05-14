//! Filesystem helpers shared across modules.

use crate::error::AppError;
use std::path::Path;

/// Write `contents` to `path` atomically: write to a sibling temp file in the
/// same directory, then `rename` it over the target.
///
/// `rename` is atomic on POSIX and on Windows for same-volume moves, so a
/// reader either sees the complete old file or the complete new one — never
/// a half-written file. The temp file MUST live in the same directory as the
/// target; a cross-filesystem rename is not atomic and would fail with
/// `EXDEV`.
///
/// The temp file name includes the process id so two concurrent `ftl2lang`
/// runs writing to the same target don't clobber each other's temp file. On
/// any error the temp file is best-effort removed so we don't leave litter.
pub fn atomic_write(path: &Path, contents: &str) -> Result<(), AppError> {
    // Resolve the parent directory; create it if missing. An empty parent
    // (bare filename) means "current directory" — don't try to create "".
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(dir) = parent {
        std::fs::create_dir_all(dir)?;
    }

    // Temp path: <target>.<pid>.tmp, sitting next to the target so the
    // rename stays within one filesystem.
    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(format!(".{}.tmp", std::process::id()));
    let tmp_path = std::path::PathBuf::from(tmp_name);

    if let Err(e) = std::fs::write(&tmp_path, contents) {
        // Nothing to clean up — the write itself failed.
        return Err(AppError::Io(e));
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        // Rename failed; remove the temp file so we don't leave litter.
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::Io(e));
    }

    Ok(())
}
