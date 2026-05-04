use serde::Deserialize;
use std::path::Path;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub codec: String,
    pub resolution: String,
    pub extension: String,
}

#[derive(Deserialize)]
struct MiOutput {
    media: MiMedia,
}

#[derive(Deserialize)]
struct MiMedia {
    track: Vec<MiTrack>,
}

#[derive(Deserialize)]
struct MiTrack {
    #[serde(rename = "@type")]
    track_type: String,
    #[serde(rename = "Format")]
    format: Option<String>,
    #[serde(rename = "Height")]
    height: Option<String>,
    #[serde(rename = "FileExtension")]
    file_extension: Option<String>,
}

impl MediaInfo {
    pub fn from_file(path: &Path) -> Result<Self> {
        let output = std::process::Command::new("mediainfo")
            .arg("--Output=JSON")
            .arg(path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::MediaInfoNotFound(e.to_string())
                } else {
                    Error::Io(e)
                }
            })?;

        if !output.status.success() {
            return Err(Error::MediaInfo(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Self::from_json(&output.stdout)
    }

    pub fn from_json(json: &[u8]) -> Result<Self> {
        let parsed: MiOutput = serde_json::from_slice(json)?;

        let mut codec = "Unknown".to_string();
        let mut height: Option<u32> = None;
        let mut extension = String::new();

        for track in &parsed.media.track {
            match track.track_type.as_str() {
                "General" => {
                    if let Some(ext) = &track.file_extension {
                        extension = ext.clone();
                    }
                }
                "Video" => {
                    if let Some(fmt) = &track.format {
                        codec = normalize_codec(fmt);
                    }
                    if let Some(h) = &track.height {
                        height = h.split_whitespace().next()
                            .and_then(|s| s.parse().ok());
                    }
                }
                _ => {}
            }
        }

        Ok(MediaInfo {
            codec,
            resolution: bucket_resolution(height),
            extension,
        })
    }
}

fn normalize_codec(format: &str) -> String {
    match format {
        "AV1"  => "AV1".to_string(),
        "HEVC" => "H.265".to_string(),
        "AVC"  => "H.264".to_string(),
        other  => other.to_string(),
    }
}

fn bucket_resolution(height: Option<u32>) -> String {
    match height {
        Some(h) if h >= 2160 => "4K".to_string(),
        Some(h) if h >= 1080 => "1080p".to_string(),
        Some(h) if h >= 720  => "720p".to_string(),
        Some(h)              => format!("{}p", h),
        None                 => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(format: &str, height: &str, ext: &str) -> String {
        format!(r#"{{
          "media": {{
            "track": [
              {{"@type": "General", "FileExtension": "{ext}"}},
              {{"@type": "Video", "Format": "{format}", "Height": "{height}"}}
            ]
          }}
        }}"#)
    }

    #[test]
    fn parses_av1_1080p() {
        let info = MediaInfo::from_json(fixture("AV1", "1080", "mkv").as_bytes()).unwrap();
        assert_eq!(info.codec, "AV1");
        assert_eq!(info.resolution, "1080p");
        assert_eq!(info.extension, "mkv");
    }

    #[test]
    fn parses_hevc_4k() {
        let info = MediaInfo::from_json(fixture("HEVC", "2160", "mp4").as_bytes()).unwrap();
        assert_eq!(info.codec, "H.265");
        assert_eq!(info.resolution, "4K");
    }

    #[test]
    fn parses_avc_720p() {
        let info = MediaInfo::from_json(fixture("AVC", "720", "avi").as_bytes()).unwrap();
        assert_eq!(info.codec, "H.264");
        assert_eq!(info.resolution, "720p");
    }

    #[test]
    fn buckets_480p() {
        let info = MediaInfo::from_json(fixture("AV1", "480", "mkv").as_bytes()).unwrap();
        assert_eq!(info.resolution, "480p");
    }
}
