# MediaNamer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build MediaNamer, a Linux GUI application that renames movie and TV episode files using TMDB metadata and MediaInfo technical properties.

**Architecture:** Cargo workspace with two crates — `medianamer-core` (pure business logic, no UI) and `medianamer-app` (iced GUI). All TMDB calls are async via Tokio; MediaInfo runs as a blocking subprocess. State mutations flow through a single `Message` enum in the iced update loop.

**Tech Stack:** Rust 2021, iced 0.13 (GUI + tokio runtime), reqwest 0.12 (HTTP), strsim 0.11 (Jaro-Winkler), serde/toml 0.8 (config), thiserror 2, dirs 5, wiremock 0.6 (test fixtures)

---

## File Map

```
Cargo.toml                                    workspace root
.gitignore

medianamer-core/
  Cargo.toml
  src/
    lib.rs                                    public re-exports
    error.rs                                  MediaNamerError enum
    config.rs                                 Config struct, TOML read/write
    mediainfo/
      mod.rs                                  run mediainfo CLI, parse JSON
    sources/
      mod.rs                                  MediaMatch, MediaType, MediaSource trait
      tmdb.rs                                 TmdbSource implementation
    matcher/
      mod.rs                                  ParsedFilename, parse_filename(), score()
    naming/
      mod.rs                                  TokenValues, format_name(), sanitize()
    renamer/
      mod.rs                                  RenameJob, execute_rename()
  tests/
    tmdb_integration.rs                       wiremock-based TMDB tests
    pipeline.rs                               full pipeline integration test

medianamer-app/
  Cargo.toml
  src/
    main.rs                                   iced entry point, subscription wiring
    state.rs                                  AppState, MatchState, View, Message
    ui/
      mod.rs                                  root view() dispatcher
      toolbar.rs                              toolbar widget
      format_bar.rs                           format bar + help toggle
      file_list.rs                            file table rows
      match_picker.rs                         ambiguous-match dialog
      settings.rs                             settings panel
      help_panel.rs                           token reference panel
  assets/
    icon.png                                  512×512 app icon (placeholder PNG)
    medianamer.desktop                        .desktop launcher entry

packaging/
  debian/                                     cargo-deb config
  arch/PKGBUILD                               AUR package build script
  rpm/                                        cargo-generate-rpm config
```

---

## Task 1: Workspace Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `medianamer-core/Cargo.toml`
- Create: `medianamer-app/Cargo.toml`

- [ ] **Step 1: Initialise git**

```bash
cd /home/matt/MediaNamer
git init
```

- [ ] **Step 2: Write workspace Cargo.toml**

```toml
# Cargo.toml
[workspace]
members = ["medianamer-core", "medianamer-app"]
resolver = "2"
```

- [ ] **Step 3: Write medianamer-core/Cargo.toml**

```toml
[package]
name = "medianamer-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
strsim = "0.11"
async-trait = "0.1"
thiserror = "2"
dirs = "5"

[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 4: Write medianamer-app/Cargo.toml**

```toml
[package]
name = "medianamer-app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "medianamer"
path = "src/main.rs"

[dependencies]
medianamer-core = { path = "../medianamer-core" }
iced = { version = "0.13", features = ["tokio"] }
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 5: Create stub source files so the workspace compiles**

```bash
mkdir -p medianamer-core/src/{sources,mediainfo,matcher,naming,renamer}
mkdir -p medianamer-core/tests
mkdir -p medianamer-app/src/ui
mkdir -p medianamer-app/assets
touch medianamer-core/src/{lib,error,config}.rs
touch medianamer-core/src/{sources/mod,mediainfo/mod,matcher/mod,naming/mod,renamer/mod}.rs
touch medianamer-core/src/sources/tmdb.rs
touch medianamer-app/src/{main,state}.rs
touch medianamer-app/src/ui/{mod,toolbar,format_bar,file_list,match_picker,settings,help_panel}.rs
```

Add a minimal `fn main() {}` to `medianamer-app/src/main.rs`.

- [ ] **Step 6: Write .gitignore**

```
/target
.env
*.deb
*.rpm
```

- [ ] **Step 7: Verify workspace compiles**

```bash
cargo build
```

Expected: compiles with zero errors (warnings about empty files are fine).

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml .gitignore medianamer-core/Cargo.toml medianamer-app/Cargo.toml
git add medianamer-core/src medianamer-app/src medianamer-app/assets
git commit -m "chore: scaffold cargo workspace"
```

---

## Task 2: Error Types

**Files:**
- Write: `medianamer-core/src/error.rs`
- Write: `medianamer-core/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add to `medianamer-core/src/error.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core 2>&1 | head -20
```

Expected: compile error — `Error` not defined.

- [ ] **Step 3: Write error.rs**

```rust
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

    #[error("TOML serialise error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("TMDB API error: {0}")]
    Tmdb(String),

    #[error("Naming error: {0}")]
    Naming(String),
}
```

- [ ] **Step 4: Write lib.rs**

```rust
pub mod config;
pub mod error;
pub mod matcher;
pub mod mediainfo;
pub mod naming;
pub mod renamer;
pub mod sources;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p medianamer-core error
```

Expected: 2 tests pass.

- [ ] **Step 6: Commit**

```bash
git add medianamer-core/src/error.rs medianamer-core/src/lib.rs
git commit -m "feat(core): add error types"
```

---

## Task 3: Config Module

**Files:**
- Write: `medianamer-core/src/config.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-core/src/config.rs  (bottom of file)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_templates_are_populated() {
        let c = Config::default();
        assert!(c.templates.movie.contains("{title}"));
        assert!(c.templates.tv.contains("{series}"));
    }

    #[test]
    fn round_trip_toml() {
        let c = Config {
            tmdb_api_key: "testkey".to_string(),
            templates: Templates {
                movie: "{title} ({year})".to_string(),
                tv: "{series} S{season:02}E{episode:02}".to_string(),
            },
        };
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.tmdb_api_key, "testkey");
        assert_eq!(back.templates.movie, "{title} ({year})");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core config
```

Expected: compile error — `Config` not defined.

- [ ] **Step 3: Write config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tmdb_api_key: String,
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
            tmdb_api_key: String::new(),
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
    // ... (tests from Step 1 go here)
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core config
```

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/config.rs
git commit -m "feat(core): add config module"
```

---

## Task 4: MediaInfo Wrapper

**Files:**
- Write: `medianamer-core/src/mediainfo/mod.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// Inside medianamer-core/src/mediainfo/mod.rs
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
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core mediainfo
```

Expected: compile error.

- [ ] **Step 3: Write mediainfo/mod.rs**

```rust
use serde::Deserialize;
use std::path::Path;
use crate::{Error, Result};

#[derive(Debug, Clone)]
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
            .map_err(|e| Error::MediaInfoNotFound(e.to_string()))?;

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
        let mut extension = "mkv".to_string();

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
mod tests { /* paste tests from Step 1 */ }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core mediainfo
```

Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/mediainfo/mod.rs
git commit -m "feat(core): add mediainfo wrapper"
```

---

## Task 5: Data Types and MediaSource Trait

**Files:**
- Write: `medianamer-core/src/sources/mod.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-core/src/sources/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movie_display_title() {
        let m = MediaMatch {
            tmdb_id: 1,
            kind: MatchKind::Movie { title: "The Matrix".to_string(), year: 1999 },
        };
        assert_eq!(m.display_title(), "The Matrix");
    }

    #[test]
    fn tv_display_title() {
        let m = MediaMatch {
            tmdb_id: 2,
            kind: MatchKind::TvEpisode {
                series_title: "Breaking Bad".to_string(),
                season: 1,
                episode: 1,
                episode_title: "Pilot".to_string(),
            },
        };
        assert_eq!(m.display_title(), "Breaking Bad");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core sources
```

Expected: compile error.

- [ ] **Step 3: Write sources/mod.rs**

```rust
use async_trait::async_trait;
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Movie,
    Tv,
}

#[derive(Debug, Clone)]
pub enum MatchKind {
    Movie {
        title: String,
        year: u32,
    },
    TvEpisode {
        series_title: String,
        season: u32,
        episode: u32,
        episode_title: String,
    },
}

#[derive(Debug, Clone)]
pub struct MediaMatch {
    pub tmdb_id: u64,
    pub kind: MatchKind,
}

impl MediaMatch {
    pub fn display_title(&self) -> &str {
        match &self.kind {
            MatchKind::Movie { title, .. } => title,
            MatchKind::TvEpisode { series_title, .. } => series_title,
        }
    }

    pub fn year(&self) -> Option<u32> {
        match &self.kind {
            MatchKind::Movie { year, .. } => Some(*year),
            MatchKind::TvEpisode { .. } => None,
        }
    }
}

#[async_trait]
pub trait MediaSource: Send + Sync {
    fn name(&self) -> &str;
    async fn search_movie(&self, query: &str) -> Result<Vec<MediaMatch>>;
    async fn search_tv(
        &self,
        query: &str,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> Result<Vec<MediaMatch>>;
}

pub mod tmdb;

#[cfg(test)]
mod tests { /* paste tests from Step 1 */ }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core sources
```

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/sources/mod.rs
git commit -m "feat(core): add MediaMatch types and MediaSource trait"
```

---

## Task 6: TMDB Source

**Files:**
- Write: `medianamer-core/src/sources/tmdb.rs`
- Write: `medianamer-core/tests/tmdb_integration.rs`

- [ ] **Step 1: Write the failing integration tests**

```rust
// medianamer-core/tests/tmdb_integration.rs
use medianamer_core::sources::{tmdb::TmdbSource, MatchKind, MediaSource};
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn search_movie_returns_match() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/movie"))
        .and(query_param("query", "Fire on the Amazon"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"id": 18898, "title": "Fire on the Amazon", "release_date": "1993-03-14"}
            ]
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.search_movie("Fire on the Amazon").await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_title(), "Fire on the Amazon");
    assert!(matches!(&results[0].kind, MatchKind::Movie { year: 1993, .. }));
}

#[tokio::test]
async fn search_tv_fetches_episode_detail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/tv"))
        .and(query_param("query", "BBC Television Shakespeare"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": 5555, "name": "BBC Television Shakespeare"}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/3/tv/5555/season/4/episode/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "A Midsummer Night's Dream",
            "season_number": 4,
            "episode_number": 3
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source
        .search_tv("BBC Television Shakespeare", Some(4), Some(3))
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    if let MatchKind::TvEpisode { episode_title, season, episode, .. } = &results[0].kind {
        assert_eq!(episode_title, "A Midsummer Night's Dream");
        assert_eq!(*season, 4);
        assert_eq!(*episode, 3);
    } else {
        panic!("expected TvEpisode");
    }
}

#[tokio::test]
async fn search_movie_empty_results() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/movie"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": []
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.search_movie("xyznotarealfilm").await.unwrap();
    assert!(results.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p medianamer-core --test tmdb_integration 2>&1 | head -20
```

Expected: compile error — `TmdbSource` not defined.

- [ ] **Step 3: Write sources/tmdb.rs**

```rust
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::{Result, Error};
use super::{MatchKind, MediaMatch, MediaSource};

pub struct TmdbSource {
    api_key: String,
    base_url: String,
    client: Client,
}

impl TmdbSource {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.themoviedb.org".to_string(),
            client: Client::new(),
        }
    }

    pub fn new_with_base_url(api_key: impl Into<String>, base_url: &str) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }
}

// --- TMDB response shapes ---

#[derive(Deserialize)]
struct MovieSearchResponse {
    results: Vec<MovieResult>,
}

#[derive(Deserialize)]
struct MovieResult {
    id: u64,
    title: String,
    release_date: Option<String>,
}

#[derive(Deserialize)]
struct TvSearchResponse {
    results: Vec<TvResult>,
}

#[derive(Deserialize)]
struct TvResult {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct EpisodeDetail {
    name: String,
    season_number: u32,
    episode_number: u32,
}

// --- helpers ---

fn year_from_date(date: &Option<String>) -> u32 {
    date.as_deref()
        .and_then(|d| d.split('-').next())
        .and_then(|y| y.parse().ok())
        .unwrap_or(0)
}

#[async_trait]
impl MediaSource for TmdbSource {
    fn name(&self) -> &str { "TMDB" }

    async fn search_movie(&self, query: &str) -> Result<Vec<MediaMatch>> {
        let url = format!("{}/3/search/movie", self.base_url);
        let resp: MovieSearchResponse = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .query(&[("query", query)])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| Error::Tmdb(e.to_string()))?
            .json()
            .await?;

        Ok(resp.results.into_iter().map(|r| MediaMatch {
            tmdb_id: r.id,
            kind: MatchKind::Movie {
                title: r.title,
                year: year_from_date(&r.release_date),
            },
        }).collect())
    }

    async fn search_tv(
        &self,
        query: &str,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> Result<Vec<MediaMatch>> {
        let url = format!("{}/3/search/tv", self.base_url);
        let resp: TvSearchResponse = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .query(&[("query", query)])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| Error::Tmdb(e.to_string()))?
            .json()
            .await?;

        if resp.results.is_empty() {
            return Ok(vec![]);
        }

        let series = &resp.results[0];

        // If season + episode provided, fetch episode detail for the top series result
        if let (Some(s), Some(e)) = (season, episode) {
            let ep_url = format!("{}/3/tv/{}/season/{}/episode/{}", self.base_url, series.id, s, e);
            let ep: EpisodeDetail = self
                .client
                .get(&ep_url)
                .bearer_auth(&self.api_key)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| Error::Tmdb(e.to_string()))?
                .json()
                .await?;

            return Ok(vec![MediaMatch {
                tmdb_id: series.id,
                kind: MatchKind::TvEpisode {
                    series_title: series.name.clone(),
                    season: ep.season_number,
                    episode: ep.episode_number,
                    episode_title: ep.name,
                },
            }]);
        }

        // No episode info — return series-level matches
        Ok(resp.results.into_iter().map(|r| MediaMatch {
            tmdb_id: r.id,
            kind: MatchKind::TvEpisode {
                series_title: r.name,
                season: 0,
                episode: 0,
                episode_title: String::new(),
            },
        }).collect())
    }
}
```

- [ ] **Step 4: Run integration tests**

```bash
cargo test -p medianamer-core --test tmdb_integration
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/sources/tmdb.rs medianamer-core/tests/tmdb_integration.rs
git commit -m "feat(core): add TmdbSource with wiremock integration tests"
```

---

## Task 7: Filename Matcher

**Files:**
- Write: `medianamer-core/src/matcher/mod.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-core/src/matcher/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_movie_filename() {
        let p = parse_filename("Fire.on.the.Amazon.1993.1080p.AV1.mkv");
        assert_eq!(p.title_query, "fire on the amazon");
        assert_eq!(p.season, None);
        assert_eq!(p.episode, None);
    }

    #[test]
    fn parses_tv_filename_sxxexx() {
        let p = parse_filename("BBC.Shakespeare.S04E03.mkv");
        assert_eq!(p.title_query, "bbc shakespeare");
        assert_eq!(p.season, Some(4));
        assert_eq!(p.episode, Some(3));
    }

    #[test]
    fn parses_underscored_filename() {
        let p = parse_filename("the_matrix_1999_bluray.mkv");
        assert_eq!(p.title_query, "the matrix");
        assert_eq!(p.season, None);
    }

    #[test]
    fn high_confidence_above_threshold() {
        assert!(score("fire on the amazon", "Fire on the Amazon") >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn low_confidence_below_threshold() {
        assert!(score("bbc shakespeare", "Shakespeare Uncovered") < CONFIDENCE_THRESHOLD);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core matcher
```

Expected: compile error.

- [ ] **Step 3: Write matcher/mod.rs**

```rust
use strsim::jaro_winkler;

pub const CONFIDENCE_THRESHOLD: f64 = 0.85;

pub struct ParsedFilename {
    pub title_query: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
}

pub fn parse_filename(filename: &str) -> ParsedFilename {
    // Strip extension
    let stem = filename.rsplit_once('.').map(|(s, _)| s).unwrap_or(filename);

    // Extract SxxExx if present
    let (season, episode, se_end) = extract_season_episode(stem);

    // Take everything before the SxxExx marker (or full stem)
    let title_part = &stem[..se_end];

    // Replace separators, lowercase
    let spaced = title_part.replace(['.', '_', '-'], " ").to_lowercase();

    // Remove noise tokens (resolution, codec, source tags)
    const NOISE: &[&str] = &[
        "1080p", "720p", "480p", "4k", "2160p",
        "bluray", "bdrip", "dvdrip", "webrip", "hdtv", "web",
        "x264", "x265", "h264", "h265", "hevc", "avc", "av1",
        "aac", "ac3", "dts", "flac",
    ];
    let tokens: Vec<&str> = spaced
        .split_whitespace()
        .filter(|t| !NOISE.contains(t))
        .filter(|t| {
            // Remove bare 4-digit years
            !(t.len() == 4
                && t.chars().all(|c| c.is_ascii_digit())
                && (t.starts_with("19") || t.starts_with("20")))
        })
        .collect();

    ParsedFilename {
        title_query: tokens.join(" "),
        season,
        episode,
    }
}

pub fn score(query: &str, candidate: &str) -> f64 {
    jaro_winkler(&query.to_lowercase(), &candidate.to_lowercase())
}

fn extract_season_episode(s: &str) -> (Option<u32>, Option<u32>, usize) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] == b'S' || bytes[i] == b's') && i + 2 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'E' || bytes[j] == b'e') {
                let season_str = &s[i + 1..j];
                let mut k = j + 1;
                while k < bytes.len() && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                let episode_str = &s[j + 1..k];
                if let (Ok(season), Ok(episode)) =
                    (season_str.parse::<u32>(), episode_str.parse::<u32>())
                {
                    return (Some(season), Some(episode), i);
                }
            }
        }
        i += 1;
    }
    (None, None, s.len())
}

#[cfg(test)]
mod tests { /* paste tests from Step 1 */ }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core matcher
```

Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/matcher/mod.rs
git commit -m "feat(core): add filename matcher and confidence scorer"
```

---

## Task 8: Naming Engine

**Files:**
- Write: `medianamer-core/src/naming/mod.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-core/src/naming/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    fn movie_values() -> TokenValues {
        TokenValues {
            title: Some("Fire on the Amazon".to_string()),
            series: None,
            year: Some(1993),
            season: None,
            episode: None,
            resolution: "1080p".to_string(),
            codec: "AV1".to_string(),
            ext: "mkv".to_string(),
        }
    }

    fn tv_values() -> TokenValues {
        TokenValues {
            title: Some("A Midsummer Night's Dream".to_string()),
            series: Some("BBC Television Shakespeare".to_string()),
            year: None,
            season: Some(4),
            episode: Some(3),
            resolution: "1080p".to_string(),
            codec: "AV1".to_string(),
            ext: "mkv".to_string(),
        }
    }

    #[test]
    fn formats_movie() {
        let result = format_name(
            "{title} ({year}) ({resolution}) ({codec}).{ext}",
            &movie_values(),
        ).unwrap();
        assert_eq!(result, "Fire on the Amazon (1993) (1080p) (AV1).mkv");
    }

    #[test]
    fn formats_tv_with_padding() {
        let result = format_name(
            "{series} - S{season:02}E{episode:02} - {title} ({codec}).{ext}",
            &tv_values(),
        ).unwrap();
        assert_eq!(
            result,
            "BBC Television Shakespeare - S04E03 - A Midsummer Night's Dream (AV1).mkv"
        );
    }

    #[test]
    fn unknown_token_is_error() {
        let err = format_name("{bogus}", &movie_values()).unwrap_err();
        assert!(err.to_string().contains("bogus"));
    }

    #[test]
    fn sanitizes_slash() {
        let mut v = movie_values();
        v.title = Some("AC/DC Live".to_string());
        let result = format_name("{title}.{ext}", &v).unwrap();
        assert!(!result.contains('/'));
    }

    #[test]
    fn strips_leading_dot() {
        let result = format_name(".{ext}", &movie_values()).unwrap();
        assert!(!result.starts_with('.'));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core naming
```

Expected: compile error.

- [ ] **Step 3: Write naming/mod.rs**

```rust
use crate::sources::{MatchKind, MediaMatch};
use crate::mediainfo::MediaInfo;

#[derive(Debug, Clone)]
pub struct TokenValues {
    pub title: Option<String>,
    pub series: Option<String>,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub resolution: String,
    pub codec: String,
    pub ext: String,
}

impl TokenValues {
    pub fn from_match_and_info(media_match: &MediaMatch, media_info: &MediaInfo) -> Self {
        match &media_match.kind {
            MatchKind::Movie { title, year } => TokenValues {
                title: Some(title.clone()),
                series: None,
                year: Some(*year),
                season: None,
                episode: None,
                resolution: media_info.resolution.clone(),
                codec: media_info.codec.clone(),
                ext: media_info.extension.clone(),
            },
            MatchKind::TvEpisode { series_title, season, episode, episode_title } => TokenValues {
                title: Some(episode_title.clone()),
                series: Some(series_title.clone()),
                year: None,
                season: Some(*season),
                episode: Some(*episode),
                resolution: media_info.resolution.clone(),
                codec: media_info.codec.clone(),
                ext: media_info.extension.clone(),
            },
        }
    }
}

pub fn format_name(template: &str, values: &TokenValues) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut token = String::new();
            for c in chars.by_ref() {
                if c == '}' { break; }
                token.push(c);
            }
            match substitute(&token, values) {
                Ok(val) => result.push_str(&val),
                Err(e) => return Err(e),
            }
        } else {
            result.push(ch);
        }
    }

    Ok(sanitize(&result))
}

fn substitute(token: &str, v: &TokenValues) -> Result<String, String> {
    match token {
        "title"      => v.title.clone().ok_or_else(|| format!("{{title}} not available for this media type")),
        "series"     => v.series.clone().ok_or_else(|| format!("{{series}} not available for this media type")),
        "year"       => v.year.map(|y| y.to_string()).ok_or_else(|| format!("{{year}} not available for this media type")),
        "season"     => v.season.map(|s| s.to_string()).ok_or_else(|| format!("{{season}} not available for this media type")),
        "season:02"  => v.season.map(|s| format!("{:02}", s)).ok_or_else(|| format!("{{season:02}} not available")),
        "episode"    => v.episode.map(|e| e.to_string()).ok_or_else(|| format!("{{episode}} not available")),
        "episode:02" => v.episode.map(|e| format!("{:02}", e)).ok_or_else(|| format!("{{episode:02}} not available")),
        "resolution" => Ok(v.resolution.clone()),
        "codec"      => Ok(v.codec.clone()),
        "ext"        => Ok(v.ext.clone()),
        other        => Err(format!("unknown token: {{{}}}", other)),
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .filter(|&c| c != '/' && c != '\0')
        .collect::<String>()
        .trim_start_matches('.')
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests { /* paste tests from Step 1 */ }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core naming
```

Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/naming/mod.rs
git commit -m "feat(core): add token naming engine"
```

---

## Task 9: Renamer Pipeline

**Files:**
- Write: `medianamer-core/src/renamer/mod.rs`
- Write: `medianamer-core/tests/pipeline.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-core/tests/pipeline.rs
use medianamer_core::{
    mediainfo::MediaInfo,
    naming::TokenValues,
    renamer::{build_new_path, RenameJob},
    sources::{MatchKind, MediaMatch},
};
use std::path::PathBuf;

fn movie_match() -> MediaMatch {
    MediaMatch {
        tmdb_id: 18898,
        kind: MatchKind::Movie {
            title: "Fire on the Amazon".to_string(),
            year: 1993,
        },
    }
}

fn fake_info() -> MediaInfo {
    MediaInfo {
        codec: "AV1".to_string(),
        resolution: "1080p".to_string(),
        extension: "mkv".to_string(),
    }
}

#[test]
fn builds_correct_new_path() {
    let job = RenameJob {
        source: PathBuf::from("/media/Fire.on.the.Amazon.1993.mkv"),
        media_match: movie_match(),
        media_info: fake_info(),
        template: "{title} ({year}) ({resolution}) ({codec}).{ext}".to_string(),
    };
    let new_path = build_new_path(&job).unwrap();
    assert_eq!(
        new_path,
        PathBuf::from("/media/Fire on the Amazon (1993) (1080p) (AV1).mkv")
    );
}

#[test]
fn new_path_preserves_directory() {
    let job = RenameJob {
        source: PathBuf::from("/home/user/movies/something.mkv"),
        media_match: movie_match(),
        media_info: fake_info(),
        template: "{title}.{ext}".to_string(),
    };
    let new_path = build_new_path(&job).unwrap();
    assert_eq!(new_path.parent().unwrap(), PathBuf::from("/home/user/movies").as_path());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-core --test pipeline 2>&1 | head -20
```

Expected: compile error.

- [ ] **Step 3: Write renamer/mod.rs**

```rust
use std::path::{Path, PathBuf};
use crate::{
    mediainfo::MediaInfo,
    naming::{format_name, TokenValues},
    sources::MediaMatch,
    Result, Error,
};

pub struct RenameJob {
    pub source: PathBuf,
    pub media_match: MediaMatch,
    pub media_info: MediaInfo,
    pub template: String,
}

pub fn build_new_path(job: &RenameJob) -> Result<PathBuf> {
    let values = TokenValues::from_match_and_info(&job.media_match, &job.media_info);
    let new_name = format_name(&job.template, &values)
        .map_err(|e| Error::Naming(e))?;
    let dir = job.source.parent().unwrap_or_else(|| Path::new(""));
    Ok(dir.join(new_name))
}

pub fn execute_rename(job: &RenameJob) -> Result<PathBuf> {
    let new_path = build_new_path(job)?;
    std::fs::rename(&job.source, &new_path)?;
    Ok(new_path)
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-core --test pipeline
```

Expected: 2 tests pass.

- [ ] **Step 5: Run full core test suite to check nothing broke**

```bash
cargo test -p medianamer-core
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add medianamer-core/src/renamer/mod.rs medianamer-core/tests/pipeline.rs
git commit -m "feat(core): add renamer pipeline"
```

---

## Task 10: App State and Messages

**Files:**
- Write: `medianamer-app/src/state.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// medianamer-app/src/state.rs
#[cfg(test)]
mod tests {
    use super::*;
    use medianamer_core::sources::{MatchKind, MediaMatch};
    use std::path::PathBuf;

    fn dummy_match() -> MediaMatch {
        MediaMatch {
            tmdb_id: 1,
            kind: MatchKind::Movie {
                title: "Test Movie".to_string(),
                year: 2020,
            },
        }
    }

    #[test]
    fn rename_button_disabled_with_no_matched_files() {
        let state = AppState::default();
        assert!(!state.any_matched());
    }

    #[test]
    fn rename_button_enabled_after_match() {
        let mut state = AppState::default();
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/test.mkv"),
            media_info: None,
            match_state: MatchState::Matched(dummy_match()),
        });
        assert!(state.any_matched());
    }

    #[test]
    fn matched_files_removed_after_rename_complete() {
        let mut state = AppState::default();
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/a.mkv"),
            media_info: None,
            match_state: MatchState::Matched(dummy_match()),
        });
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/b.mkv"),
            media_info: None,
            match_state: MatchState::Unmatched,
        });
        state.remove_renamed(&[0]);
        assert_eq!(state.files.len(), 1);
        assert_eq!(state.files[0].path, PathBuf::from("/tmp/b.mkv"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p medianamer-app state 2>&1 | head -20
```

Expected: compile error.

- [ ] **Step 3: Write state.rs**

```rust
use std::path::PathBuf;
use medianamer_core::{
    config::Config,
    mediainfo::MediaInfo,
    sources::{MediaMatch, MediaType},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub media_type: MediaType,
    pub files: Vec<MediaFile>,
    pub config: Config,
    pub view: View,
    pub api_key_draft: String,
    pub movie_template_draft: String,
    pub tv_template_draft: String,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            media_type: MediaType::Movie,
            files: vec![],
            api_key_draft: config.tmdb_api_key.clone(),
            movie_template_draft: config.templates.movie.clone(),
            tv_template_draft: config.templates.tv.clone(),
            config,
            view: View::Main,
        }
    }
}

impl AppState {
    pub fn any_matched(&self) -> bool {
        self.files.iter().any(|f| matches!(f.match_state, MatchState::Matched(_)))
    }

    pub fn current_template(&self) -> &str {
        match self.media_type {
            MediaType::Movie => &self.config.templates.movie,
            MediaType::Tv => &self.config.templates.tv,
        }
    }

    /// Remove files at the given indices (must be sorted descending or use retain).
    pub fn remove_renamed(&mut self, indices: &[usize]) {
        let mut to_remove: std::collections::HashSet<usize> = indices.iter().copied().collect();
        let mut i = 0;
        self.files.retain(|_| {
            let keep = !to_remove.contains(&i);
            i += 1;
            keep
        });
    }
}

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub path: PathBuf,
    pub media_info: Option<MediaInfo>,
    pub match_state: MatchState,
}

#[derive(Debug, Clone)]
pub enum MatchState {
    Pending,
    Loading,
    Matched(MediaMatch),
    Ambiguous(Vec<MediaMatch>),
    Unmatched,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Main,
    Settings,
    Help,
    MatchPicker(usize),
}

#[derive(Debug, Clone)]
pub enum Message {
    FilesDropped(Vec<PathBuf>),
    MediaInfoLoaded(usize, Result<MediaInfo, String>),
    MediaTypeChanged(MediaType),
    TemplateChanged(String),
    MatchAll,
    FileMatched(usize, Result<Vec<medianamer_core::sources::MediaMatch>, String>),
    ResolveAmbiguous(usize),
    MatchSelected(usize, MediaMatch),
    Rename,
    RenameComplete(Vec<usize>),
    OpenSettings,
    CloseSettings,
    ApiKeyChanged(String),
    MovieTemplateChanged(String),
    TvTemplateChanged(String),
    SaveSettings,
    OpenHelp,
    CloseHelp,
}

#[cfg(test)]
mod tests { /* paste tests from Step 1 */ }
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p medianamer-app state
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-app/src/state.rs
git commit -m "feat(app): add AppState, MatchState, Message types"
```

---

## Task 11: Main App Wiring

**Files:**
- Write: `medianamer-app/src/main.rs`
- Write: `medianamer-app/src/ui/mod.rs`

This task wires up the iced `Application`, implements the full `update()` function, connects drag-and-drop via subscription, and dispatches async tasks for MediaInfo and TMDB.

- [ ] **Step 1: Write main.rs**

```rust
use iced::{application, event, window, Event, Subscription, Task};
use state::{AppState, Message, MatchState, View};
use medianamer_core::{
    config::Config,
    matcher::{parse_filename, score, CONFIDENCE_THRESHOLD},
    mediainfo::MediaInfo,
    renamer::execute_rename,
    renamer::RenameJob,
    sources::{tmdb::TmdbSource, MediaSource, MediaType},
};
use std::path::PathBuf;

mod state;
mod ui;

fn main() -> iced::Result {
    application("MediaNamer", update, ui::view)
        .subscription(subscription)
        .run_with(|| (AppState::default(), Task::none()))
}

fn subscription(_state: &AppState) -> Subscription<Message> {
    event::listen_with(|event, _status, _id| match event {
        Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::FilesDropped(vec![path]))
        }
        _ => None,
    })
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::FilesDropped(paths) => {
            let start_index = state.files.len();
            for (i, path) in paths.into_iter().enumerate() {
                state.files.push(state::MediaFile {
                    path: path.clone(),
                    media_info: None,
                    match_state: MatchState::Loading,
                });
                let idx = start_index + i;
                return Task::perform(
                    async move {
                        MediaInfo::from_file(&path).map_err(|e| e.to_string())
                    },
                    move |result| Message::MediaInfoLoaded(idx, result),
                );
            }
            Task::none()
        }

        Message::MediaInfoLoaded(idx, result) => {
            if let Some(file) = state.files.get_mut(idx) {
                match result {
                    Ok(info) => {
                        file.media_info = Some(info);
                        file.match_state = MatchState::Pending;
                    }
                    Err(e) => file.match_state = MatchState::Error(e),
                }
            }
            Task::none()
        }

        Message::MediaTypeChanged(mt) => {
            state.media_type = mt;
            Task::none()
        }

        Message::TemplateChanged(t) => {
            match state.media_type {
                MediaType::Movie => state.config.templates.movie = t,
                MediaType::Tv    => state.config.templates.tv = t,
            }
            Task::none()
        }

        Message::MatchAll => {
            let api_key = state.config.tmdb_api_key.clone();
            let media_type = state.media_type.clone();
            let mut tasks = vec![];

            for (idx, file) in state.files.iter_mut().enumerate() {
                if !matches!(file.match_state, MatchState::Pending) { continue; }
                file.match_state = MatchState::Loading;

                let parsed = parse_filename(
                    file.path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(""),
                );
                let query = parsed.title_query.clone();
                let season = parsed.season;
                let episode = parsed.episode;
                let key = api_key.clone();
                let mt = media_type.clone();

                tasks.push(Task::perform(
                    async move {
                        let source = TmdbSource::new(key);
                        match mt {
                            MediaType::Movie => source.search_movie(&query).await,
                            MediaType::Tv    => source.search_tv(&query, season, episode).await,
                        }
                        .map_err(|e| e.to_string())
                    },
                    move |result| Message::FileMatched(idx, result),
                ));
            }
            Task::batch(tasks)
        }

        Message::FileMatched(idx, result) => {
            if let Some(file) = state.files.get_mut(idx) {
                file.match_state = match result {
                    Err(e) => MatchState::Error(e),
                    Ok(matches) if matches.is_empty() => MatchState::Unmatched,
                    Ok(mut matches) => {
                        let filename = file.path.file_stem()
                            .and_then(|s| s.to_str()).unwrap_or("");
                        let query = parse_filename(filename).title_query;
                        let top = matches.remove(0);
                        let confidence = score(&query, top.display_title());
                        if confidence >= CONFIDENCE_THRESHOLD {
                            MatchState::Matched(top)
                        } else {
                            let mut all = vec![top];
                            all.extend(matches);
                            MatchState::Ambiguous(all)
                        }
                    }
                };
            }
            Task::none()
        }

        Message::ResolveAmbiguous(idx) => {
            state.view = View::MatchPicker(idx);
            Task::none()
        }

        Message::MatchSelected(idx, media_match) => {
            if let Some(file) = state.files.get_mut(idx) {
                file.match_state = MatchState::Matched(media_match);
            }
            state.view = View::Main;
            Task::none()
        }

        Message::Rename => {
            let template = state.current_template().to_string();
            let mut jobs: Vec<(usize, RenameJob)> = vec![];

            for (idx, file) in state.files.iter().enumerate() {
                if let (MatchState::Matched(m), Some(info)) = (&file.match_state, &file.media_info) {
                    jobs.push((idx, RenameJob {
                        source: file.path.clone(),
                        media_match: m.clone(),
                        media_info: info.clone(),
                        template: template.clone(),
                    }));
                }
            }

            let renamed_indices: Vec<usize> = jobs.iter().map(|(i, _)| *i).collect();
            for (_, job) in &jobs {
                let _ = execute_rename(job); // errors silently ignored in MVP
            }

            Task::perform(
                async move { renamed_indices },
                Message::RenameComplete,
            )
        }

        Message::RenameComplete(indices) => {
            state.remove_renamed(&indices);
            Task::none()
        }

        Message::OpenSettings => { state.view = View::Settings; Task::none() }
        Message::CloseSettings => { state.view = View::Main; Task::none() }
        Message::ApiKeyChanged(k) => { state.api_key_draft = k; Task::none() }
        Message::MovieTemplateChanged(t) => { state.movie_template_draft = t; Task::none() }
        Message::TvTemplateChanged(t) => { state.tv_template_draft = t; Task::none() }
        Message::SaveSettings => {
            state.config.tmdb_api_key = state.api_key_draft.clone();
            state.config.templates.movie = state.movie_template_draft.clone();
            state.config.templates.tv = state.tv_template_draft.clone();
            let _ = state.config.save();
            state.view = View::Main;
            Task::none()
        }
        Message::OpenHelp => { state.view = View::Help; Task::none() }
        Message::CloseHelp => { state.view = View::Main; Task::none() }
    }
}
```

- [ ] **Step 2: Write ui/mod.rs**

```rust
use iced::Element;
use crate::state::{AppState, Message, View};

mod file_list;
mod format_bar;
mod help_panel;
mod match_picker;
mod settings;
mod toolbar;

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.view {
        View::Settings      => settings::view(state),
        View::Help          => help_panel::view(state),
        View::MatchPicker(idx) => match_picker::view(state, *idx),
        View::Main          => main_view(state),
    }
}

fn main_view(state: &AppState) -> Element<'_, Message> {
    use iced::widget::{column, text};
    column![
        toolbar::view(state),
        format_bar::view(state),
        file_list::view(state),
        text("Drop files here to add").size(12),
    ]
    .into()
}
```

- [ ] **Step 3: Verify it compiles (stubs for UI widgets are empty — fill in next tasks)**

```bash
cargo build -p medianamer-app 2>&1 | grep error
```

Expected: any remaining errors are only about unimplemented stub functions in ui/ files — fix by adding `todo!()` returns.

- [ ] **Step 4: Commit**

```bash
git add medianamer-app/src/main.rs medianamer-app/src/ui/mod.rs
git commit -m "feat(app): wire up iced Application, update loop, and subscription"
```

---

## Task 12: Toolbar and Format Bar

**Files:**
- Write: `medianamer-app/src/ui/toolbar.rs`
- Write: `medianamer-app/src/ui/format_bar.rs`

- [ ] **Step 1: Write toolbar.rs**

```rust
use iced::widget::{button, pick_list, row, Space};
use iced::{Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::MediaType;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let media_types = &[MediaType::Movie, MediaType::Tv];

    row![
        button("+ Add Files").on_press(Message::FilesDropped(vec![])), // opens native dialog — placeholder
        button("Match All").on_press(Message::MatchAll),
        button("Rename")
            .on_press_maybe(state.any_matched().then_some(Message::Rename)),
        Space::with_width(Length::Fill),
        pick_list(
            media_types.as_ref(),
            Some(&state.media_type),
            Message::MediaTypeChanged,
        ),
        button("⚙").on_press(Message::OpenSettings),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
```

Note: `MediaType` needs `Display` and `PartialEq + Clone` for `pick_list`. Add to `sources/mod.rs`:

```rust
impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "Movies"),
            MediaType::Tv    => write!(f, "TV Episodes"),
        }
    }
}
```

- [ ] **Step 2: Write format_bar.rs**

```rust
use iced::widget::{button, row, text, text_input};
use iced::{Element};
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let template = state.current_template().to_string();

    row![
        text("FORMAT").size(11),
        text_input("", &template)
            .on_input(Message::TemplateChanged)
            .padding(4),
        button("?").on_press(Message::OpenHelp),
    ]
    .spacing(8)
    .padding([4, 8])
    .into()
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build -p medianamer-app 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add medianamer-app/src/ui/toolbar.rs medianamer-app/src/ui/format_bar.rs
git add medianamer-core/src/sources/mod.rs
git commit -m "feat(app): add toolbar and format bar widgets"
```

---

## Task 13: File List

**Files:**
- Write: `medianamer-app/src/ui/file_list.rs`

- [ ] **Step 1: Write file_list.rs**

```rust
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};
use crate::state::{AppState, MatchState, Message};
use medianamer_core::naming::{format_name, TokenValues};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = row![
        text("Original Filename").width(Length::FillPortion(4)),
        text("New Filename").width(Length::FillPortion(4)),
        text("Status").width(Length::FillPortion(1)),
    ]
    .padding([4, 8])
    .spacing(8);

    let rows: Vec<Element<'_, Message>> = state
        .files
        .iter()
        .enumerate()
        .map(|(idx, file)| file_row(state, idx, file))
        .collect();

    scrollable(
        column![header]
            .extend(rows)
            .width(Length::Fill)
    )
    .height(Length::Fill)
    .into()
}

fn file_row<'a>(
    state: &'a AppState,
    idx: usize,
    file: &'a crate::state::MediaFile,
) -> Element<'a, Message> {
    let original = file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let (preview, status_text, status_color) = match &file.match_state {
        MatchState::Pending | MatchState::Loading => {
            ("…".to_string(), "Loading", Color::from_rgb8(180, 180, 180))
        }
        MatchState::Matched(m) => {
            let template = state.current_template();
            let preview = if let Some(info) = &file.media_info {
                let values = TokenValues::from_match_and_info(m, info);
                format_name(template, &values).unwrap_or_else(|e| e)
            } else {
                "?".to_string()
            };
            (preview, "✓", Color::from_rgb8(106, 191, 105))
        }
        MatchState::Ambiguous(_) => {
            ("(click to resolve)".to_string(), "?", Color::from_rgb8(255, 167, 38))
        }
        MatchState::Unmatched => {
            ("(not matched)".to_string(), "✗", Color::from_rgb8(239, 83, 80))
        }
        MatchState::Error(e) => {
            (format!("Error: {}", e), "⚠", Color::from_rgb8(239, 83, 80))
        }
    };

    let row_content = row![
        text(&original).width(Length::FillPortion(4)).size(12),
        text(&preview).width(Length::FillPortion(4)).size(12),
        text(status_text).color(status_color).width(Length::FillPortion(1)).size(12),
    ]
    .spacing(8)
    .padding([4, 8]);

    if matches!(file.match_state, MatchState::Ambiguous(_)) {
        button(row_content)
            .on_press(Message::ResolveAmbiguous(idx))
            .into()
    } else {
        container(row_content).into()
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build -p medianamer-app 2>&1 | grep "^error"
```

- [ ] **Step 3: Commit**

```bash
git add medianamer-app/src/ui/file_list.rs
git commit -m "feat(app): add file list widget with preview and status"
```

---

## Task 14: Match Picker, Settings, and Help Panel

**Files:**
- Write: `medianamer-app/src/ui/match_picker.rs`
- Write: `medianamer-app/src/ui/settings.rs`
- Write: `medianamer-app/src/ui/help_panel.rs`

- [ ] **Step 1: Write match_picker.rs**

```rust
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{AppState, MatchState, Message};
use medianamer_core::sources::MatchKind;

pub fn view(state: &AppState, file_idx: usize) -> Element<'_, Message> {
    let file = match state.files.get(file_idx) {
        Some(f) => f,
        None => return text("Error: file not found").into(),
    };

    let candidates = match &file.match_state {
        MatchState::Ambiguous(list) => list.clone(),
        _ => return text("No candidates").into(),
    };

    let filename = file.path.file_name()
        .and_then(|n| n.to_str()).unwrap_or("?");

    let candidate_rows: Vec<Element<'_, Message>> = candidates
        .into_iter()
        .map(|m| {
            let label = match &m.kind {
                MatchKind::Movie { title, year } => format!("{} ({})", title, year),
                MatchKind::TvEpisode { series_title, season, episode, episode_title } =>
                    format!("{} S{:02}E{:02} — {}", series_title, season, episode, episode_title),
            };
            let m_clone = m.clone();
            button(text(label).size(13))
                .on_press(Message::MatchSelected(file_idx, m_clone))
                .width(Length::Fill)
                .into()
        })
        .collect();

    container(
        column![
            text(format!("Select match for: {}", filename)).size(14),
            scrollable(column(candidate_rows).spacing(4)).height(Length::Fill),
            button("Cancel").on_press(Message::CloseSettings),
        ]
        .spacing(12)
        .padding(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
```

- [ ] **Step 2: Write settings.rs**

```rust
use iced::widget::{button, column, row, text, text_input};
use iced::Element;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    column![
        text("Settings").size(18),
        row![
            text("TMDB API Key").width(160),
            text_input("Paste your TMDB API key here", &state.api_key_draft)
                .on_input(Message::ApiKeyChanged)
                .password()
                .padding(4),
        ].spacing(8),
        row![
            text("Movie template").width(160),
            text_input("{title} ({year}) ({resolution}) ({codec})", &state.movie_template_draft)
                .on_input(Message::MovieTemplateChanged)
                .padding(4),
        ].spacing(8),
        row![
            text("TV template").width(160),
            text_input("{series} - S{season:02}E{episode:02} - {title} ({codec})", &state.tv_template_draft)
                .on_input(Message::TvTemplateChanged)
                .padding(4),
        ].spacing(8),
        row![
            button("Save").on_press(Message::SaveSettings),
            button("Cancel").on_press(Message::CloseSettings),
        ].spacing(8),
    ]
    .spacing(16)
    .padding(24)
    .into()
}
```

- [ ] **Step 3: Write help_panel.rs**

```rust
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};
use crate::state::Message;

const TOKENS: &[(&str, &str, &str)] = &[
    ("{title}",      "TMDB",      "Fire on the Amazon  /  A Midsummer Night's Dream"),
    ("{series}",     "TMDB (TV)", "BBC Television Shakespeare"),
    ("{year}",       "TMDB",      "1993"),
    ("{season}",     "TMDB (TV)", "4"),
    ("{season:02}",  "TMDB (TV)", "04"),
    ("{episode}",    "TMDB (TV)", "3"),
    ("{episode:02}", "TMDB (TV)", "03"),
    ("{resolution}", "MediaInfo", "1080p  /  4K  /  720p"),
    ("{codec}",      "MediaInfo", "AV1  /  H.265  /  H.264"),
    ("{ext}",        "MediaInfo", "mkv  /  mp4"),
];

pub fn view(_state: &crate::state::AppState) -> Element<'_, Message> {
    let header = row![
        text("Token").width(160).size(12),
        text("Source").width(120).size(12),
        text("Example").size(12),
    ]
    .spacing(8);

    let rows: Vec<Element<'_, Message>> = TOKENS
        .iter()
        .map(|(token, source, example)| {
            row![
                text(*token).width(160).size(12),
                text(*source).width(120).size(12),
                text(*example).size(12),
            ]
            .spacing(8)
            .into()
        })
        .collect();

    container(
        column![
            text("Token Reference").size(18),
            header,
            column(rows).spacing(6),
            button("Close").on_press(Message::CloseHelp),
        ]
        .spacing(12)
        .padding(24),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
```

- [ ] **Step 4: Verify the full app compiles and runs**

```bash
cargo build -p medianamer-app 2>&1 | grep "^error"
cargo run -p medianamer-app
```

Expected: app window opens, toolbar and file list visible, drag-and-drop responds.

- [ ] **Step 5: Commit**

```bash
git add medianamer-app/src/ui/match_picker.rs
git add medianamer-app/src/ui/settings.rs
git add medianamer-app/src/ui/help_panel.rs
git commit -m "feat(app): add match picker, settings, and help panel"
```

---

## Task 15: Assets and .desktop File

**Files:**
- Create: `medianamer-app/assets/medianamer.desktop`
- Create: `medianamer-app/assets/icon.png` (placeholder)

- [ ] **Step 1: Write the .desktop file**

```ini
# medianamer-app/assets/medianamer.desktop
[Desktop Entry]
Name=MediaNamer
Comment=Rename movie and TV episode files using TMDB metadata
Exec=medianamer
Icon=medianamer
Terminal=false
Type=Application
Categories=AudioVideo;Utility;
Keywords=rename;media;movie;tv;
```

- [ ] **Step 2: Create placeholder icon**

```bash
# Create a minimal 512x512 PNG placeholder (requires imagemagick)
convert -size 512x512 xc:#1e88e5 \
  -fill white -font DejaVu-Sans-Bold -pointsize 120 \
  -gravity center -annotate 0 "MN" \
  medianamer-app/assets/icon.png
```

If ImageMagick is not available, copy any 512×512 PNG to that path.

- [ ] **Step 3: Commit**

```bash
git add medianamer-app/assets/
git commit -m "chore: add app icon and .desktop entry"
```

---

## Task 16: Packaging

**Files:**
- Create: `medianamer-app/Cargo.toml` — add `[package.metadata.deb]`
- Create: `packaging/arch/PKGBUILD`
- Create: `packaging/rpm/medianamer.toml`

- [ ] **Step 1: Add cargo-deb metadata to medianamer-app/Cargo.toml**

Append to `medianamer-app/Cargo.toml`:

```toml
[package.metadata.deb]
name = "medianamer"
maintainer = "MediaNamer Contributors"
copyright = "2026"
license-file = ["../LICENSE", "0"]
extended-description = "Rename movie and TV episode files using TMDB metadata and MediaInfo."
depends = "mediainfo, libssl3"
section = "utils"
priority = "optional"
assets = [
    ["target/release/medianamer", "usr/bin/", "755"],
    ["assets/medianamer.desktop", "usr/share/applications/", "644"],
    ["assets/icon.png", "usr/share/icons/hicolor/512x512/apps/medianamer.png", "644"],
]
```

- [ ] **Step 2: Write packaging/arch/PKGBUILD**

```bash
# packaging/arch/PKGBUILD
pkgname=medianamer
pkgver=0.1.0
pkgrel=1
pkgdesc="Rename movie and TV episode files using TMDB metadata"
arch=('x86_64')
url="https://github.com/youruser/medianamer"
license=('MIT')
depends=('mediainfo')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/youruser/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release -p medianamer-app
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/medianamer" "$pkgdir/usr/bin/medianamer"
    install -Dm644 "medianamer-app/assets/medianamer.desktop" \
        "$pkgdir/usr/share/applications/medianamer.desktop"
    install -Dm644 "medianamer-app/assets/icon.png" \
        "$pkgdir/usr/share/icons/hicolor/512x512/apps/medianamer.png"
}
```

- [ ] **Step 3: Write packaging/rpm/medianamer.toml** (cargo-generate-rpm config)

```toml
[package.metadata.generate-rpm]
name = "medianamer"
version = "0.1.0"
license = "MIT"
summary = "Rename movie and TV episode files using TMDB metadata"
requires = { mediainfo = "*", openssl = "*" }

[[package.metadata.generate-rpm.assets]]
source = "target/release/medianamer"
dest = "/usr/bin/medianamer"
mode = "0755"

[[package.metadata.generate-rpm.assets]]
source = "medianamer-app/assets/medianamer.desktop"
dest = "/usr/share/applications/medianamer.desktop"
mode = "0644"

[[package.metadata.generate-rpm.assets]]
source = "medianamer-app/assets/icon.png"
dest = "/usr/share/icons/hicolor/512x512/apps/medianamer.png"
mode = "0644"
```

Note: Add the `[package.metadata.generate-rpm]` block to `medianamer-app/Cargo.toml` or keep it in a separate file and pass `--config` to cargo-generate-rpm.

- [ ] **Step 4: Build and test the .deb (on Debian/Ubuntu)**

```bash
cargo install cargo-deb
cargo deb -p medianamer-app
ls target/debian/
```

Expected: `medianamer_0.1.0_amd64.deb` present.

- [ ] **Step 5: Commit**

```bash
git add packaging/ medianamer-app/Cargo.toml
git commit -m "chore: add packaging configs for Debian, Arch, and RPM"
```

---

## Self-Review Checklist

All spec sections covered:

| Spec section | Tasks |
|---|---|
| GUI — iced, single panel layout | 11, 12, 13, 14 |
| Drag-and-drop | 11 (subscription in main.rs) |
| TMDB integration | 6 |
| MediaInfo integration | 4 |
| Token naming engine | 8 |
| Confidence-based auto-matching | 7, 10 |
| Match picker dialog | 14 |
| Batch rename | 11 (update loop) |
| Config file | 3 |
| Packaging (Debian, Arch, RPM) | 16 |
| Unit tests | 2–10 |
| Integration tests (wiremock) | 6 |
