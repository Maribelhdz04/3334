use std::path::Path;

use crate::types::{FileStats, ProcessingError};

pub trait Analyzer: Send + Sync {
    fn name(&self) -> &'static str;

    /// Update stats/errors for this file.
    ///
    /// `content` is decoded text (may be lossy); `size_bytes` is the on-disk size.
    fn analyze(
        &self,
        path: &Path,
        content: &str,
        size_bytes: u64,
        stats: &mut FileStats,
        errors: &mut Vec<ProcessingError>,
    );
}

pub struct WordCount;
impl Analyzer for WordCount {
    fn name(&self) -> &'static str { "word_count" }
    fn analyze(&self, _path: &Path, content: &str, _size: u64, stats: &mut FileStats, _errors: &mut Vec<ProcessingError>) {
        stats.word_count = content.split_whitespace().count();
    }
}

pub struct LineCount;
impl Analyzer for LineCount {
    fn name(&self) -> &'static str { "line_count" }
    fn analyze(&self, _path: &Path, content: &str, _size: u64, stats: &mut FileStats, _errors: &mut Vec<ProcessingError>) {
        stats.line_count = content.lines().count();
    }
}

pub struct CharFrequency;
impl Analyzer for CharFrequency {
    fn name(&self) -> &'static str { "char_frequencies" }
    fn analyze(&self, _path: &Path, content: &str, _size: u64, stats: &mut FileStats, _errors: &mut Vec<ProcessingError>) {
        for ch in content.chars() {
            *stats.char_frequencies.entry(ch).or_insert(0) += 1;
        }
    }
}

pub struct FileSize;
impl Analyzer for FileSize {
    fn name(&self) -> &'static str { "size_bytes" }
    fn analyze(&self, _path: &Path, _content: &str, size: u64, stats: &mut FileStats, _errors: &mut Vec<ProcessingError>) {
        stats.size_bytes = size;
    }
}

/// Default analyzers required by the assignment.
pub fn default_analyzers() -> Vec<Box<dyn Analyzer>> {
    vec![
        Box::new(FileSize),
        Box::new(LineCount),
        Box::new(WordCount),
        Box::new(CharFrequency),
    ]
}
