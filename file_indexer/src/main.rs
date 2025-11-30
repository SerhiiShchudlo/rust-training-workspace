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

    let max_threads_count = thread::available_parallelism()?.get();
    let (tx_paths, rx_paths) = mpsc::channel::<PathBuf>();
    let (tx_results, rx_results) = mpsc::channel::<(String, HashMap<String, Vec<usize>>)>();
    let rx_paths = Arc::new(Mutex::new(rx_paths));

    for path in files {
        tx_paths.send(path).unwrap();
    }
    drop(tx_paths);

    let threads: Vec<_> = (0..max_threads_count)
        .map(|_| {
            let rx = Arc::clone(&rx_paths);
            let tx = tx_results.clone();
            thread::spawn(move || loop {
                let next_path = {
                    let locked = rx.lock().unwrap();
                    locked.recv()
                };

                match next_path {
                    Ok(path) => {
                        if let Ok(map) = index_file(&path) {
                            let _ = tx.send((path.display().to_string(), map));
                        }
                    }
                    Err(_) => break,
                }
            })
        })
        .collect();
    drop(tx_results);

    let mut result_map: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();
    for (file, file_index) in rx_results {
        for (word, positions) in file_index {
            result_map
                .entry(word)
                .or_default()
                .insert(file.clone(), positions);
        }
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => println!("Worker thread error"),
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
