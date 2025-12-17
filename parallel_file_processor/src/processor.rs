use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use crate::analyzers::Analyzer;
use crate::types::{FileAnalysis, FileStats, FileStatus, ProcessingError};

#[derive(Clone)]
pub struct ProcessorConfig {
    /// If set, skip files larger than this (bytes).
    pub max_bytes: Option<u64>,
    /// If true, try a naive ISO-8859-1 decode when UTF-8 fails.
    pub allow_latin1_fallback: bool,
    /// If set, append completed file paths to this progress file.
    pub progress_file: Option<PathBuf>,
    /// If true, skip any file already listed as DONE in progress_file.
    pub resume: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_bytes: None,
            allow_latin1_fallback: true,
            progress_file: None,
            resume: false,
        }
    }
}

#[derive(Debug)]
pub enum ProgressEvent {
    Started { path: String },
    Finished { path: String, status: FileStatus, processing_time: Duration, error_count: usize },
    Skipped { path: String, reason: String },
}

#[derive(Default)]
pub struct ProgressState {
    pub total: usize,
    pub done: usize,
    pub errors: usize,
    pub skipped: usize,
    pub in_progress: usize,
    pub statuses: std::collections::HashMap<String, FileStatus>,
    pub cancelled: bool,
}

pub fn load_resume_set(progress_file: &Path) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    if let Ok(content) = fs::read_to_string(progress_file) {
        for line in content.lines() {
            // format: "<STATUS>\t<path>"
            let mut parts = line.splitn(2, '\t');
            let status = parts.next().unwrap_or("").trim();
            let path = parts.next().unwrap_or("").trim();
            if status == "DONE" && !path.is_empty() {
                set.insert(path.to_string());
            }
        }
    }
    set
}

pub fn process_files_parallel(
    files: Vec<PathBuf>,
    analyzers: Arc<Vec<Box<dyn Analyzer>>>,
    cancel_flag: Arc<AtomicBool>,
    cfg: ProcessorConfig,
    pool: &crate::thread_pool::ThreadPool,
    progress_state: Arc<Mutex<ProgressState>>,
) -> Vec<FileAnalysis> {
    let (tx, rx) = mpsc::channel::<ProgressEvent>();
    let results: Arc<Mutex<Vec<FileAnalysis>>> = Arc::new(Mutex::new(Vec::new()));

    let resume_set = if cfg.resume {
        cfg.progress_file
            .as_deref()
            .map(load_resume_set)
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    {
        let mut st = progress_state.lock().unwrap();
        st.total = files.len();
        for p in &files {
            st.statuses.insert(p.display().to_string(), FileStatus::Pending);
        }
    }

    // Progress printer thread
    let printer_cancel = Arc::clone(&cancel_flag);
    let printer_state = Arc::clone(&progress_state);
    let printer = std::thread::spawn(move || {
        let mut last_print = Instant::now();
        loop {
            if last_print.elapsed() >= Duration::from_millis(250) {
                let snap = {
                    let st = printer_state.lock().unwrap();
                    (st.total, st.done, st.errors, st.skipped, st.in_progress, st.cancelled)
                };

                eprint!(
                    "\rTotal: {:>5} | Done: {:>5} | Err: {:>5} | Skip: {:>5} | InProg: {:>5}{}",
                    snap.0, snap.1, snap.2, snap.3, snap.4,
                    if snap.5 { " | CANCELLED" } else { "" }
                );
                let _ = std::io::Write::flush(&mut std::io::stderr());
                last_print = Instant::now();
            }

            if printer_cancel.load(Ordering::SeqCst) {
                {
                    let mut st = printer_state.lock().unwrap();
                    st.cancelled = true;
                }
                break;
            }

            // stop when everything done
            {
                let st = printer_state.lock().unwrap();
                if st.done + st.errors + st.skipped >= st.total && st.total > 0 {
                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(40));
        }
        eprintln!(); // newline
    });

    // Event consumer thread (updates state + persistence)
    let consumer_state = Arc::clone(&progress_state);
    let consumer_cfg = cfg.clone();
    let consumer = std::thread::spawn(move || {
        let mut progress_writer = consumer_cfg.progress_file.as_ref().and_then(|p| {
            fs::OpenOptions::new().create(true).append(true).open(p).ok()
        });

        while let Ok(ev) = rx.recv() {
            match ev {
                ProgressEvent::Started { path } => {
                    let mut st = consumer_state.lock().unwrap();
                    st.in_progress += 1;
                    st.statuses.insert(path, FileStatus::InProgress);
                }
                ProgressEvent::Finished { path, status, processing_time: _, error_count } => {
                    let mut st = consumer_state.lock().unwrap();
                    if st.in_progress > 0 { st.in_progress -= 1; }
                    match status {
                        FileStatus::Done => {
                            st.done += 1;
                            if let Some(w) = progress_writer.as_mut() {
                                let _ = writeln!(w, "DONE\t{}", path);
                            }
                        }
                        FileStatus::Error => {
                            st.errors += 1;
                            if let Some(w) = progress_writer.as_mut() {
                                let _ = writeln!(w, "ERROR({})\t{}", error_count, path);
                            }
                        }
                        _ => {}
                    }
                    st.statuses.insert(path, status);
                }
                ProgressEvent::Skipped { path, reason: _ } => {
                    let mut st = consumer_state.lock().unwrap();
                    st.skipped += 1;
                    st.statuses.insert(path, FileStatus::Skipped);
                }
            }
        }
    });

    // Submit tasks
    let mut iter = files.into_iter();
    while let Some(path) = iter.next() {
        if cancel_flag.load(Ordering::SeqCst) {
            {
                let mut st = progress_state.lock().unwrap();
                st.cancelled = true;
            }
            // Any files that didn't even enqueue should be marked skipped so progress can finish.
            for rem in iter {
                let _ = tx.send(ProgressEvent::Skipped {
                    path: rem.display().to_string(),
                    reason: "cancelled (not scheduled)".to_string(),
                });
            }
            break;
        }

        let path_str = path.display().to_string();

        if cfg.resume && resume_set.contains(&path_str) {
            let _ = tx.send(ProgressEvent::Skipped {
                path: path_str,
                reason: "resume: already DONE".to_string(),
            });
            continue;
        }

        let txc = tx.clone();
        let analyzers = Arc::clone(&analyzers);
        let cancel = Arc::clone(&cancel_flag);
        let results = Arc::clone(&results);
        let cfgc = cfg.clone();

        let _ = pool.execute(move || {
            if cancel.load(Ordering::SeqCst) {
                let _ = txc.send(ProgressEvent::Skipped {
                    path: path.display().to_string(),
                    reason: "cancelled before start".to_string(),
                });
                return;
            }

            let _ = txc.send(ProgressEvent::Started { path: path.display().to_string() });

            let analysis = process_one_file(&path, &*analyzers, &cancel, &cfgc);

            let status = if analysis.errors.is_empty() { FileStatus::Done } else { FileStatus::Error };
            let _ = txc.send(ProgressEvent::Finished {
                path: analysis.filename.clone(),
                status,
                processing_time: analysis.processing_time,
                error_count: analysis.errors.len(),
            });

            results.lock().unwrap().push(analysis);
        });
    }

    // Drop sender so consumer can finish
    drop(tx);

    // Wait for consumer/printer threads
    let _ = consumer.join();
    let _ = printer.join();

   let out = results.lock().unwrap().clone();
   out 
}

fn process_one_file(
    path: &Path,
    analyzers: &[Box<dyn Analyzer>],
    cancel: &AtomicBool,
    cfg: &ProcessorConfig,
) -> FileAnalysis {
    let start = Instant::now();

    let mut errors = Vec::new();
    let mut stats = FileStats::default();
    let filename = path.display().to_string();

    if cancel.load(Ordering::SeqCst) {
        errors.push(ProcessingError::cancelled(Some(path.to_path_buf())));
        return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
    }

    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            errors.push(ProcessingError::io(path.to_path_buf(), &e));
            return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
        }
    };
    let size_bytes = meta.len();

    if let Some(max) = cfg.max_bytes {
        if size_bytes > max {
            errors.push(ProcessingError::too_large(path.to_path_buf(), size_bytes, max));
            return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
        }
    }

    // Read bytes
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            errors.push(ProcessingError::io(path.to_path_buf(), &e));
            return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
        }
    };

    let mut bytes = Vec::new();
    if let Err(e) = file.read_to_end(&mut bytes) {
        errors.push(ProcessingError::io(path.to_path_buf(), &e));
        return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
    }

    if cancel.load(Ordering::SeqCst) {
        errors.push(ProcessingError::cancelled(Some(path.to_path_buf())));
        return FileAnalysis { filename, stats, errors, processing_time: start.elapsed() };
    }

    // Decode (bonus: latin1 fallback)
    let content = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            if cfg.allow_latin1_fallback {
                let bytes = e.into_bytes();
                // ISO-8859-1: byte -> same codepoint
                let s: String = bytes.into_iter().map(|b| char::from(b)).collect();
                errors.push(ProcessingError::decode(path.to_path_buf(), "UTF-8 decode failed; used ISO-8859-1 fallback"));
                s
            } else {
                let bytes = e.into_bytes();
                let s = String::from_utf8_lossy(&bytes).to_string();
                errors.push(ProcessingError::decode(path.to_path_buf(), "UTF-8 decode failed; used lossy UTF-8"));
                s
            }
        }
    };

    for analyzer in analyzers {
        if cancel.load(Ordering::SeqCst) {
            errors.push(ProcessingError::cancelled(Some(path.to_path_buf())));
            break;
        }
        analyzer.analyze(path, &content, size_bytes, &mut stats, &mut errors);
    }

    FileAnalysis {
        filename,
        stats,
        errors,
        processing_time: start.elapsed(),
    }
}
