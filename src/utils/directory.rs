use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

pub struct DirectorySearch {}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub ctime: Option<DateTime<Utc>>,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

impl Default for FileEntry {
    fn default() -> Self {
        FileEntry {
            name: String::new(),
            ctime: None,
            path: PathBuf::new(),
            size: 0,
            is_dir: false,
        }
    }
}

impl DirectorySearch {
    pub fn new() -> Self {
        DirectorySearch {}
    }

    pub fn open_directory(
        path: &PathBuf,
        hidden: bool,
        restrict: Option<&Vec<String>>,
    ) -> Vec<FileEntry> {
        log::info!("Opening directory: {}", path.display());
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                // Skip if list is empty
                // if let Some(restrict) = restrict {
                //     log::debug!("Restricting file types: {:?}", restrict);
                //     if let Some(ext) = entry.path().extension() {
                //         log::debug!("Checking file extension: {:?}", ext);
                //         if !restrict.contains(&ext.to_string_lossy().to_string()) {
                //             continue; // Skip files not in the restricted list
                //         }
                //     }
                // }
                let file_path = entry.path();
                let file_size = entry.metadata().map_or(0, |m| m.len());
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());
                if !hidden && file_name.starts_with('.') {
                    continue; // Skip hidden files if not allowed
                }
                let ctime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.created().ok())
                    .map(DateTime::<Utc>::from);
                files.push(FileEntry {
                    name: file_name,
                    path: file_path,
                    ctime,
                    size: file_size,
                    is_dir,
                });
            }
            files.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
        } else {
            log::error!("Failed to read directory: {}", path.display());
        }
        files.insert(
            0,
            FileEntry {
                name: String::from(".."),
                path: path.parent().unwrap_or(&path.clone()).to_path_buf(),
                ctime: None,
                size: files.iter().map(|f| f.size).sum::<u64>(),
                is_dir: true,
            },
        );
        files
    }
}
