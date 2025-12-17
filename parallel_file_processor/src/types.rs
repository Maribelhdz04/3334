use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub filename: String,
    pub stats: FileStats,
    pub errors: Vec<ProcessingError>,
    pub processing_time: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct FileStats {
    pub word_count: usize,
    pub line_count: usize,
    pub char_frequencies: HashMap<char, usize>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub kind: ProcessingErrorKind,
    pub path: Option<PathBuf>,
    pub message: String,
    pub io_kind: Option<std::io::ErrorKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingErrorKind {
    Io,
    Decode,
    Cancelled,
    TooLarge,
    Other,
}

impl ProcessingError {
    pub fn io(path: PathBuf, err: &std::io::Error) -> Self {
        Self {
            kind: ProcessingErrorKind::Io,
            path: Some(path),
            message: err.to_string(),
            io_kind: Some(err.kind()),
        }
    }

    pub fn decode(path: PathBuf, msg: impl Into<String>) -> Self {
        Self {
            kind: ProcessingErrorKind::Decode,
            path: Some(path),
            message: msg.into(),
            io_kind: None,
        }
    }

    pub fn cancelled(path: Option<PathBuf>) -> Self {
        Self {
            kind: ProcessingErrorKind::Cancelled,
            path,
            message: "cancelled".to_string(),
            io_kind: None,
        }
    }

    pub fn too_large(path: PathBuf, size: u64, max: u64) -> Self {
        Self {
            kind: ProcessingErrorKind::TooLarge,
            path: Some(path),
            message: format!("file too large: {size} bytes (max {max})"),
            io_kind: None,
        }
    }

    pub fn other(path: Option<PathBuf>, msg: impl Into<String>) -> Self {
        Self {
            kind: ProcessingErrorKind::Other,
            path,
            message: msg.into(),
            io_kind: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Pending,
    InProgress,
    Done,
    Error,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub total: usize,
    pub done: usize,
    pub errors: usize,
    pub skipped: usize,
    pub in_progress: usize,
    pub cancelled: bool,
}
