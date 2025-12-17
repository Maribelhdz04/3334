mod analyzers;
mod fs_walk;
mod processor;
mod thread_pool;
mod types;

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

use parallel_file_processor::analyzers::default_analyzers;
use parallel_file_processor::processor::{process_files_parallel, ProcessorConfig};
use parallel_file_processor::types::{FileAnalysis, ProcessingError};

fn print_usage() {
    eprintln!(
        "Usage:\n  parallel_file_processor --dir <path> [--dir <path2> ...] [--threads N] [--out report.txt] [--max-bytes BYTES] [--progress-file progress.tsv] [--resume]\n\n\
         While running: type 'c' then Enter to cancel.\n"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut threads: Option<usize> = None;
    let mut out: Option<PathBuf> = None;
    let mut max_bytes: Option<u64> = None;
    let mut progress_file: Option<PathBuf> = None;
    let mut resume = false;

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--dir" => {
                let p = it.next().ok_or("--dir needs a path")?;
                dirs.push(PathBuf::from(p));
            }
            "--threads" => {
                let n = it.next().ok_or("--threads needs a number")?;
                threads = Some(n.parse::<usize>().map_err(|_| "invalid --threads value")?);
            }
            "--out" => {
                let p = it.next().ok_or("--out needs a path")?;
                out = Some(PathBuf::from(p));
            }
            "--max-bytes" => {
                let n = it.next().ok_or("--max-bytes needs a number")?;
                max_bytes = Some(n.parse::<u64>().map_err(|_| "invalid --max-bytes value")?);
            }
            "--progress-file" => {
                let p = it.next().ok_or("--progress-file needs a path")?;
                progress_file = Some(PathBuf::from(p));
            }
            "--resume" => {
                resume = true;
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    if dirs.is_empty() {
        dirs.push(PathBuf::from("books"));
    }

    let threads = threads.unwrap_or_else(default_thread_count);

    Ok(Args {
        dirs,
        threads,
        out,
        max_bytes,
        progress_file,
        resume,
    })
}

fn default_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

struct Args {
    dirs: Vec<PathBuf>,
    threads: usize,
    out: Option<PathBuf>,
    max_bytes: Option<u64>,
    progress_file: Option<PathBuf>,
    resume: bool,
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Argument error: {e}\n");
            print_usage();
            std::process::exit(2);
        }
    };

    let cancel_flag = Arc::new(AtomicBool::new(false));
    spawn_cancel_listener(Arc::clone(&cancel_flag));

    let (files, walk_errors) = parallel_file_processor::fs_walk::collect_files(&args.dirs);
    if files.is_empty() {
        eprintln!("No files found. Make sure your --dir points to a folder with .txt files.");
    }

    let mut cfg = ProcessorConfig::default();
    cfg.max_bytes = args.max_bytes;
    cfg.progress_file = args.progress_file.clone();
    cfg.resume = args.resume;

    let analyzers = Arc::new(default_analyzers());
    let progress_state = Arc::new(Mutex::new(parallel_file_processor::processor::ProgressState::default()));

    eprintln!(
        "Found {} files in {:?}. Starting {} worker threads...",
        files.len(),
        args.dirs,
        args.threads
    );

    let overall_start = Instant::now();
    let pool = parallel_file_processor::thread_pool::ThreadPool::new(args.threads);

    let mut all_results = process_files_parallel(
        files,
        analyzers,
        Arc::clone(&cancel_flag),
        cfg,
        &pool,
        Arc::clone(&progress_state),
    );

    // Sort results for stable output
    all_results.sort_by(|a, b| a.filename.cmp(&b.filename));

    let total_time = overall_start.elapsed();
    let (done, err, skipped, cancelled) = {
        let st = progress_state.lock().unwrap();
        (st.done, st.errors, st.skipped, st.cancelled)
    };

    let header = format!(
        "\n=== SUMMARY ===\nDone: {done} | Errors: {err} | Skipped: {skipped} | Total time: {:?}{}\n",
        total_time,
        if cancelled { " (CANCELLED)" } else { "" }
    );

    print!("{header}");
    print_report(&all_results);

    if let Some(out_path) = args.out {
        if let Err(e) = write_report(&out_path, &header, &all_results, &walk_errors) {
            eprintln!("Failed to write report to {}: {e}", out_path.display());
        } else {
            eprintln!("Wrote report to {}", out_path.display());
        }
    }

    // Also surface any directory-walk errors
    if !walk_errors.is_empty() {
        eprintln!("\nDirectory-walk errors:");
        for e in walk_errors {
            eprintln!("- {}", format_error(&e));
        }
    }
}

fn spawn_cancel_listener(cancel_flag: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            let res = stdin.read_line(&mut line);
            if res.is_err() {
                // stdin closed
                break;
            }
            let s = line.trim().to_lowercase();
            if s == "c" || s == "cancel" || s == "q" || s == "quit" {
                cancel_flag.store(true, Ordering::SeqCst);
                break;
            }
        }
    });
}

fn print_report(results: &[FileAnalysis]) {
    for r in results {
        println!(
            "\nFILE: {}\n  size_bytes: {}\n  lines: {}\n  words: {}\n  processing_time: {:?}\n  errors: {}",
            r.filename,
            r.stats.size_bytes,
            r.stats.line_count,
            r.stats.word_count,
            r.processing_time,
            r.errors.len()
        );

        if !r.errors.is_empty() {
            for e in &r.errors {
                println!("    - {}", format_error(e));
            }
        }

        // show top 10 chars by frequency 
        let mut freqs: Vec<(char, usize)> = r.stats.char_frequencies.iter().map(|(c, n)| (*c, *n)).collect();
        freqs.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<String> = freqs.into_iter().take(10).map(|(c, n)| format!("{:?}:{n}", c)).collect();
        if !top.is_empty() {
            println!("  top_chars: {}", top.join(", "));
        }
    }
}

fn write_report(
    out_path: &PathBuf,
    header: &str,
    results: &[FileAnalysis],
    walk_errors: &[ProcessingError],
) -> std::io::Result<()> {
    use std::io::Write;

    let mut f = std::fs::File::create(out_path)?;
    writeln!(f, "{header}")?;

    if !walk_errors.is_empty() {
        writeln!(f, "=== DIRECTORY WALK ERRORS ===")?;
        for e in walk_errors {
            writeln!(f, "- {}", format_error(e))?;
        }
        writeln!(f)?;
    }

    writeln!(f, "=== FILE RESULTS ===")?;
    for r in results {
        writeln!(f, "FILE\t{}\tSIZE\t{}\tLINES\t{}\tWORDS\t{}\tTIME_MS\t{}\tERRORS\t{}",
            r.filename,
            r.stats.size_bytes,
            r.stats.line_count,
            r.stats.word_count,
            r.processing_time.as_millis(),
            r.errors.len()
        )?;
    }
    Ok(())
}

fn format_error(e: &ProcessingError) -> String {
    match &e.path {
        Some(p) => format!("{:?} at {}: {}", e.kind, p.display(), e.message),
        None => format!("{:?}: {}", e.kind, e.message),
    }
}
