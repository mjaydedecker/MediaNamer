use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tmdb_read_access_token: String,
    pub templates: Templates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Templates {
    pub movie: String,
    pub tv: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tmdb_read_access_token: String::new(),
            templates: Templates {
                movie: "{title} ({year}) ({resolution}) ({codec})".to_string(),
                tv: "{series} - S{season:02}E{episode:02} - {title} ({codec})".to_string(),
            },
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("medianamer")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_templates_are_populated() {
        let c = Config::default();
        assert!(c.templates.movie.contains("{title}"));
        assert!(c.templates.tv.contains("{series}"));
    }

    #[test]
    fn round_trip_toml() {
        let c = Config {
            tmdb_read_access_token: "testkey".to_string(),
            templates: Templates {
                movie: "{title} ({year})".to_string(),
                tv: "{series} S{season:02}E{episode:02}".to_string(),
            },
        };
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.tmdb_read_access_token, "testkey");
        assert_eq!(back.templates.movie, "{title} ({year})");
    }
}
