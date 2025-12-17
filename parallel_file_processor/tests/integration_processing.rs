use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};

use parallel_file_processor::analyzers::default_analyzers;
use parallel_file_processor::processor::{process_files_parallel, ProcessorConfig, ProgressState};
use parallel_file_processor::thread_pool::ThreadPool;

fn make_temp_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "pfp_test_{}_{}",
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
fn processes_files_and_counts_words_lines() {
    let dir = make_temp_dir();
    let f1 = dir.join("a.txt");
    let f2 = dir.join("b.txt");
    fs::write(&f1, "hello world\nsecond line\n").unwrap();
    fs::write(&f2, "one two three\n").unwrap();

    let files = vec![f1.clone(), f2.clone()];

    let analyzers = Arc::new(default_analyzers());
    let cancel = Arc::new(AtomicBool::new(false));
    let cfg = ProcessorConfig::default();
    let pool = ThreadPool::new(2);
    let progress = Arc::new(Mutex::new(ProgressState::default()));

    let mut results = process_files_parallel(files, analyzers, cancel, cfg, &pool, progress);

    results.sort_by(|a, b| a.filename.cmp(&b.filename));

    assert_eq!(results.len(), 2);

    let a = &results[0];
    assert!(a.filename.ends_with("a.txt"));
    assert_eq!(a.stats.line_count, 2);
    assert_eq!(a.stats.word_count, 4);

    let b = &results[1];
    assert!(b.filename.ends_with("b.txt"));
    assert_eq!(b.stats.line_count, 1);
    assert_eq!(b.stats.word_count, 3);

    // cleanup
    let _ = fs::remove_dir_all(&dir);
}
