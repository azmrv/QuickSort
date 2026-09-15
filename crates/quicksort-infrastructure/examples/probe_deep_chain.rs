//! One-off diagnostic: compare recursive vs iterative traversal on a deep tree.
//! Not part of the library — run with:
//! `cargo run -p quicksort-infrastructure --example probe_deep_chain -- <mode> <path> [stack_mb]`
//!   mode: recursive | iterative  (default: iterative)
//!   stack_mb: thread stack size in MiB (default: 1, matching a typical Windows thread)
//! Run the two modes in SEPARATE processes: a stack overflow aborts the process.

use std::env;
use std::path::{Path, PathBuf};

/// Recursive traversal — mirrors the current `walk_directory` shape
/// (Box::pin + .await at each level). Expected to blow the stack on deep chains.
fn probe_recursive<'a>(
    dir: &'a Path,
    depth: u32,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = u64> + Send + 'a>> {
    Box::pin(async move {
        if depth > 5000 {
            return 0;
        }
        let mut count = 1u64;
        let mut entries = match tokio::fs::read_dir(dir).await {
            Ok(e) => e,
            Err(_) => return count,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                count += probe_recursive(&entry.path(), depth + 1).await;
            }
        }
        count
    })
}

/// Iterative traversal — the proposed fix.
async fn probe_iterative(start: PathBuf) -> u64 {
    let mut stack = vec![start];
    let mut count = 0u64;
    while let Some(dir) = stack.pop() {
        count += 1;
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                stack.push(entry.path());
            }
        }
    }
    count
}

fn run(mode: String, path: PathBuf, stack_mb: usize) {
    let child = std::thread::Builder::new()
        .name(format!("probe-{mode}"))
        .stack_size(stack_mb * 1024 * 1024)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let start = std::time::Instant::now();
                match mode.as_str() {
                    "recursive" => {
                        println!("[recursive] running on thread (stack={stack_mb}MiB)...");
                        let n = probe_recursive(&path, 0).await;
                        println!("[recursive] ok: dirs={n} elapsed={:?}", start.elapsed());
                    }
                    _ => {
                        println!("[iterative] running on thread (stack={stack_mb}MiB)...");
                        let n = probe_iterative(path).await;
                        println!("[iterative] ok: dirs={n} elapsed={:?}", start.elapsed());
                    }
                }
            });
        })
        .expect("spawn probe thread");

    let _ = child.join();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "iterative".to_string());
    let path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let stack_mb: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

    println!(
        "probe mode={mode} target={} stack={stack_mb}MiB",
        path.display()
    );
    run(mode, path, stack_mb);
    println!("(thread joined; if no '[..] ok:' line printed above, the mode crashed the process)");
}
