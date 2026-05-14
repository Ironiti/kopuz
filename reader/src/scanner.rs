use super::converter::convert_to_mp3;
use super::metadata::read;
use super::models::Library;
use async_recursion::async_recursion;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

pub async fn scan_directory(
    dir: PathBuf,
    cover_cache: PathBuf,
    library: &mut Library,
    on_progress: Arc<dyn Fn(String) + Send + Sync>,
) -> std::io::Result<()> {
    let existing_paths: HashSet<PathBuf> = library.tracks.iter().map(|t| t.path.clone()).collect();
    scan_directory_internal(dir, cover_cache, library, &existing_paths, on_progress).await
}

#[async_recursion]
async fn scan_directory_internal(
    dir: PathBuf,
    cover_cache: PathBuf,
    library: &mut Library,
    existing_paths: &HashSet<PathBuf>,
    on_progress: Arc<dyn Fn(String) + Send + Sync>,
) -> std::io::Result<()> {
    let mut entries = match fs::read_dir(&dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut audio_files = Vec::new();
    let mut sub_dirs = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            sub_dirs.push(path);
        } else if is_audio_file(&path) {
            if !existing_paths.contains(&path) {
                audio_files.push(path);
            }
        }
    }

    if !audio_files.is_empty() {
        let mut lib = std::mem::take(library);
        let cover_cache_clone = cover_cache.clone();
        let progress = on_progress.clone();

        lib = tokio::task::spawn_blocking(move || {
            // Create a temporary directory for converted files
            let temp_dir = std::env::temp_dir().join("kopuz_converted");
            let _ = std::fs::create_dir_all(&temp_dir);

            for path in audio_files {
                let name = path.file_name();
                if let Some(name) = name {
                    progress(name.to_string_lossy().into_owned());
                }
                
                // Check if file needs conversion (not MP3)
                let file_to_read = if path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase())
                    .as_deref() != Some("mp3")
                {
                    // Try to convert to MP3
                    match convert_to_mp3(&path, &temp_dir) {
                        Ok(converted_path) => {
                            if let Some(name) = name {
                                progress(format!("Converted: {}", name.to_string_lossy()));
                            }
                            converted_path
                        }
                        Err(e) => {
                            eprintln!("Failed to convert {}: {}", path.display(), e);
                            // If conversion fails, try to read the original file
                            path
                        }
                    }
                } else {
                    // Already MP3, use as is
                    path
                };
                
                read(&file_to_read, &cover_cache_clone, &mut lib);
            }
            lib
        })
        .await
        .unwrap();

        *library = lib;
    }

    for sub_dir in sub_dirs {
        let _ = scan_directory_internal(
            sub_dir,
            cover_cache.clone(),
            library,
            existing_paths,
            on_progress.clone(),
        )
        .await;
    }

    Ok(())
}

pub fn is_audio_file(path: &Path) -> bool {
    use super::converter::is_convertible_format;
    
    // Accept all convertible formats (audio and video)
    is_convertible_format(path)
}
