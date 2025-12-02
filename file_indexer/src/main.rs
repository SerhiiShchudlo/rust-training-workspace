use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use serde_json;
use anyhow::Result;

fn main() -> Result<()> {
    let root = Path::new("/home/serhii/texts");

    let mut files = Vec::new();
    collect_files(root, &mut files)?;

    let max_threads_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(3);
    let (tx_results, rx_results) = mpsc::channel::<(String, HashMap<String, Vec<usize>>)>();
    let rx = Arc::new(Mutex::new(files));

    thread::scope(|scope| {
        let threads: Vec<_> = (0..max_threads_count)
            .map(|_| {
                let rx = Arc::clone(&rx);
                let tx = tx_results.clone();
                scope.spawn(move || loop {
                    let next_path = {
                        rx.lock().unwrap().pop()
                    };

                    match next_path {
                        Some(path) => {
                            if let Ok(map) = index_file(&path) {
                                let _ = tx.send((path.display().to_string(), map));
                            }
                        }
                        None => break,
                    }
                })
            })
            .collect();

        drop(tx_results);

        for thread in threads {
            if let Err(e) = thread.join() {
                println!("Worker thread error: {e:?}");
            }
        }
    });

    let mut result_map: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();
    for (file, file_index) in rx_results {
        for (word, positions) in file_index {
            result_map
                .entry(word)
                .or_default()
                .insert(file.clone(), positions);
        }
    }

    let result_map_json = serde_json::to_string_pretty(&result_map)?;
    println!("{result_map_json}");

    Ok(())
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }

    Ok(())
}

fn index_file(path: &Path) -> io::Result<HashMap<String, Vec<usize>>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut index = 0;

    for line in reader.lines() {
        let line = line?;
        for word in line.split_whitespace() {
            map.entry(word.to_lowercase()).or_default().push(index);
            index += 1;
        }
    }
    
    Ok(map)
}
