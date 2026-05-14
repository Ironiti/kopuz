use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported input formats for conversion
pub fn is_convertible_format(path: &Path) -> bool {
    let audio_extensions = [
        // Audio formats
        "mp3", "flac", "m4a", "aac", "wav", "ogg", "opus", "wma", "aiff", "ape", "alac",
        "webm", "mka", "oga", "spx", "tta", "wv", "dts", "ac3", "amr",
        // Video formats (extract audio)
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg",
        "3gp", "ogv", "ts", "vob", "divx", "f4v", "asf", "rm", "rmvb",
    ];
    
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| audio_extensions.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Convert audio/video file to MP3 format using ffmpeg
/// Returns the path to the converted MP3 file
pub fn convert_to_mp3(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    // Check if ffmpeg is available
    if !is_ffmpeg_available() {
        return Err("ffmpeg is not installed or not found in PATH".to_string());
    }

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Generate output filename
    let input_filename = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid input filename")?;
    
    let output_path = output_dir.join(format!("{}.mp3", input_filename));

    // If output file already exists, generate unique name
    let output_path = if output_path.exists() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        output_dir.join(format!("{}_{}.mp3", input_filename, timestamp))
    } else {
        output_path
    };

    // Run ffmpeg conversion
    // -i: input file
    // -vn: no video (for video files, extract only audio)
    // -ar 44100: sample rate 44.1kHz
    // -ac 2: stereo audio
    // -b:a 320k: bitrate 320kbps (high quality)
    // -y: overwrite output file if exists
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vn")
        .arg("-ar")
        .arg("44100")
        .arg("-ac")
        .arg("2")
        .arg("-b:a")
        .arg("320k")
        .arg("-y")
        .arg(&output_path)
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg conversion failed: {}", error_msg));
    }

    Ok(output_path)
}

/// Check if ffmpeg is available in the system
pub fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get ffmpeg version string
pub fn get_ffmpeg_version() -> Option<String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .ok()?;

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        version_output.lines().next().map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_convertible_format() {
        assert!(is_convertible_format(Path::new("test.mp3")));
        assert!(is_convertible_format(Path::new("test.flac")));
        assert!(is_convertible_format(Path::new("test.mp4")));
        assert!(is_convertible_format(Path::new("test.mkv")));
        assert!(is_convertible_format(Path::new("test.wav")));
        assert!(!is_convertible_format(Path::new("test.txt")));
        assert!(!is_convertible_format(Path::new("test.jpg")));
    }

    #[test]
    fn test_ffmpeg_availability() {
        // This test will pass if ffmpeg is installed
        let available = is_ffmpeg_available();
        println!("ffmpeg available: {}", available);
        
        if available {
            let version = get_ffmpeg_version();
            println!("ffmpeg version: {:?}", version);
            assert!(version.is_some());
        }
    }
}
