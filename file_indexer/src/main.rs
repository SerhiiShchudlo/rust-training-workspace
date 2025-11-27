use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let root = Path::new("/home/serhii/texts");

    let mut files = Vec::new();
    collect_files(root, &mut files)?;

    let mut result_map: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();

    for path in files {
        match index_file(&path) {
            Ok(file_index) => {
                let file_key = path.display().to_string();
                for (word, positions) in file_index {
                    result_map
                        .entry(word)
                        .or_default()
                        .insert(file_key.clone(), positions);
                }
            }
            Err(_) => println!("Failed to index {}", path.display()),
        }
    }

    println!("{result_map:#?}");

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
