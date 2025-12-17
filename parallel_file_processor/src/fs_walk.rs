use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{ProcessingError, ProcessingErrorKind};

/// Recursively collect file paths from multiple directories.
/// Errors are returned as ProcessingError entries; the walk continues.
pub fn collect_files(dirs: &[PathBuf]) -> (Vec<PathBuf>, Vec<ProcessingError>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();

    for dir in dirs {
        walk_dir(dir, &mut files, &mut errors);
    }

    (files, errors)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>, errors: &mut Vec<ProcessingError>) {
    let read = fs::read_dir(dir);
    let entries = match read {
        Ok(e) => e,
        Err(err) => {
            errors.push(ProcessingError::io(dir.to_path_buf(), &err));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                errors.push(ProcessingError {
                    kind: ProcessingErrorKind::Io,
                    path: Some(dir.to_path_buf()),
                    message: err.to_string(),
                    io_kind: Some(err.kind()),
                });
                continue;
            }
        };

        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(err) => {
                errors.push(ProcessingError::io(path.clone(), &err));
                continue;
            }
        };

        if ft.is_dir() {
            walk_dir(&path, files, errors);
        } else if ft.is_file() {
            files.push(path);
        }
    }
}
