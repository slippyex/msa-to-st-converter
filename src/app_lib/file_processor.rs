use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use rayon::prelude::*;

use crate::app_lib::msa_decoder::decode_msa_to_st;

pub fn process_file(src_path: &Path,
                    dest_dir: PathBuf,
                    relative_path: PathBuf,
                    log: &Arc<RwLock<Vec<String>>>,
                    count: &Arc<RwLock<u32>>,
                    batch_size: usize,
                    in_ui_mode: bool) -> io::Result<()> {
    let ext = src_path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
    if ext == "msa" {
        let st_path = dest_dir.join(relative_path).with_extension("st");
        if let Some(parent) = st_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut msa_data = Vec::new();
        File::open(&src_path)?.read_to_end(&mut msa_data)?;
        if let Some(st_data) = decode_msa_to_st(&msa_data) {
            File::create(&st_path)?.write_all(&st_data)?;
            let msg = format!("Converted {} successfully.", src_path.display());
            if !in_ui_mode {
                println!("{}", msg);
            }
            {
                let mut log = log.write().unwrap();
                log.push(msg);
                if log.len() > batch_size {
                    log.remove(0);
                }
            }
            {
                let mut count = count.write().unwrap();
                *count += 1;
            }
        } else {
            let msg = format!("Could not convert {}. Skipping this file.", src_path.display());
            if !in_ui_mode {
                println!("{}", msg);
            }
            {
                let mut log = log.write().unwrap();
                log.push(msg);
                if log.len() > batch_size {
                    log.remove(0);
                }
            }
        }
    }
    Ok(())
}

pub fn traverse_directory(src_dir: &Path, dest_dir: &Path,
                          relative_path: PathBuf,
                          log: Arc<RwLock<Vec<String>>>,
                          count: Arc<RwLock<u32>>,
                          batch_size: usize,
                          in_ui_mode: bool) -> io::Result<()> {
    fs::create_dir_all(dest_dir)?;

    let entries: Vec<_> = match fs::read_dir(src_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", src_dir.display(), e);
            return Ok(());
        }
    };

    entries.into_par_iter().for_each(|entry| {
        let src_path = entry.path();
        let new_relative_path = relative_path.join(entry.file_name());
        let dest_dir = dest_dir.to_path_buf();
        let log = log.clone();
        let count = count.clone();

        if let Err(e) = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            traverse_directory(&src_path, &dest_dir, new_relative_path, log, count, batch_size, in_ui_mode)
        } else {
            process_file(&src_path, dest_dir, new_relative_path, &log, &count, batch_size, in_ui_mode)
        } {
            if let Some(os_err) = e.raw_os_error() {
                if os_err == 3 || os_err == 2 {
                    // Ignore and swallow OS Error 3 and OS Error 2
                } else {
                    eprintln!("Error processing {}: {}", src_path.display(), e);
                }
            } else {
                eprintln!("Error processing {}: {}", src_path.display(), e);
            }
        }
    });

    // Folders cleanup. If after traversing files, the directory is empty, delete it.
    if fs::read_dir(dest_dir.join(&relative_path)).map_or(true, |mut dir| dir.next().is_none()) {
        if let Err(e) = fs::remove_dir(dest_dir.join(&relative_path)) {
            if let Some(os_err) = e.raw_os_error() {
                if os_err != 3 && os_err != 2 {
                    eprintln!("Failed to remove directory {}: {}", dest_dir.join(&relative_path).display(), e);
                }
            } else {
                eprintln!("Failed to remove directory {}: {}", dest_dir.join(&relative_path).display(), e);
            }
        } else {
            let msg = format!("Directory {} is empty and was removed.", dest_dir.join(&relative_path).display());
            if !in_ui_mode {
                println!("{}", msg);
            }
        }
    }
    Ok(())
}
