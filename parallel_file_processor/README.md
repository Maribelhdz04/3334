# Parallel File Processor (Thread Pool + Multi-Dir File Analysis)

Processes many text files concurrently using a **custom thread pool** (no async, no rayon/tokio, no third-party crates).

## Features
- Dynamic worker thread count (`--threads N`)
- Processes files from **multiple directories** (`--dir path` can repeat)
- Analyzers:
  - word count
  - line count
  - character frequency
  - file size stats
- Real-time progress (per-file status + totals)
- Cancellation (type `c` + Enter at runtime)
- Graceful error handling (filesystem / decoding issues)
- Progress persistence + resume (`--progress-file progress.tsv` + `--resume`)

## Data Requirement (100+ Project Gutenberg books)
Project Gutenberg provides "Top 100" lists and plain-text downloads. You can use the script below to download at least 100 plain-text books.

### Quick download script (uses `curl`)
```bash
bash scripts/download_gutenberg_top100.sh books
```

This script scrapes book IDs from Project Gutenberg’s “Top 100” page and downloads each as `ebooks/<id>.txt.utf-8`.
Sources: https://www.gutenberg.org/browse/scores/top and example plain-text: https://www.gutenberg.org/ebooks/1342.txt.utf-8

> Note: Project Gutenberg’s lists change over time, but this approach consistently gets 100+ text files quickly.

## Run
```bash
cargo run --release -- --dir books --threads 8 --out report.txt
```

Multiple directories:
```bash
cargo run --release -- --dir books --dir more_books --threads 6
```

Resume:
```bash
cargo run --release -- --dir books --resume --progress-file progress.tsv
```

Cancel:
- While running, type `c` then press Enter.

## Tests
```bash
cargo test
```

Performance-ish smoke bench (ignored by default):
```bash
cargo test --release -- --ignored --nocapture
```

## Output
Results are printed and optionally written to a report file. The core output structs match the assignment:

```rust
struct FileAnalysis {
    filename: String,
    stats: FileStats,
    errors: Vec<ProcessingError>,
    processing_time: Duration,
}

struct FileStats {
    word_count: usize,
    line_count: usize,
    char_frequencies: HashMap<char, usize>,
    size_bytes: u64,
}
```
