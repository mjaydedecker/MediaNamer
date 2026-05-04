use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("mediainfo not found on PATH: {0}. Install it with your package manager.")]
    MediaInfoNotFound(String),

    #[error("mediainfo error: {0}")]
    MediaInfo(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("TMDB API error: {0}")]
    Tmdb(String),

    #[error("Naming error: {0}")]
    Naming(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_display() {
        let e = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file missing"));
        assert!(e.to_string().contains("file missing"));
    }

    #[test]
    fn mediainfo_not_found_display() {
        let e = Error::MediaInfoNotFound("not in PATH".to_string());
        assert!(e.to_string().contains("mediainfo"));
    }
}
