use std::fs;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::time::Instant;

use parallel_file_processor::analyzers::default_analyzers;
use parallel_file_processor::processor::{process_files_parallel, ProcessorConfig, ProgressState};
use parallel_file_processor::thread_pool::ThreadPool;

fn make_temp_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "pfp_bench_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[ignore]
fn bench_small_dataset() {
    let dir = make_temp_dir();

    // Create a bunch of small files
    for i in 0..200 {
        let path = dir.join(format!("file_{i}.txt"));
        fs::write(&path, "the quick brown fox jumps over the lazy dog\n".repeat(200)).unwrap();
    }

    let files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();

    let analyzers = Arc::new(default_analyzers());
    let cancel = Arc::new(AtomicBool::new(false));
    let cfg = ProcessorConfig::default();
    let pool = ThreadPool::new(8);
    let progress = Arc::new(Mutex::new(ProgressState::default()));

    let start = Instant::now();
    let results = process_files_parallel(files, analyzers, cancel, cfg, &pool, progress);
    let elapsed = start.elapsed();

    println!("Processed {} files in {:?}", results.len(), elapsed);

    let _ = fs::remove_dir_all(&dir);
}
