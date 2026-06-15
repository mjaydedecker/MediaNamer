use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Movie,
    Tv,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "Movies"),
            MediaType::Tv    => write!(f, "TV Episodes"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum MovieSource { #[default] Tmdb, Omdb }

impl std::fmt::Display for MovieSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { MovieSource::Tmdb => write!(f, "TMDB"), MovieSource::Omdb => write!(f, "OMDB") }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TvSource { #[default] Tmdb, Omdb, Tvmaze, Tvdb }

impl std::fmt::Display for TvSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TvSource::Tmdb   => write!(f, "TMDB"),
            TvSource::Omdb   => write!(f, "OMDB"),
            TvSource::Tvmaze => write!(f, "TVmaze"),
            TvSource::Tvdb   => write!(f, "TheTVDB"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MatchKind {
    Movie {
        title: String,
        year: Option<u32>,
    },
    TvEpisode {
        series_title: String,
        season: Option<u32>,
        episode: Option<u32>,
        episode_title: Option<String>,
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
            MatchKind::Movie { year, .. } => *year,
            MatchKind::TvEpisode { .. } => None,
        }
    }
}

#[async_trait]
pub trait MediaSource: Send + Sync {
    fn name(&self) -> &str;
    async fn search_movie(&self, query: &str, year: Option<u32>) -> Result<Vec<MediaMatch>>;
    async fn search_tv(
        &self,
        query: &str,
        season: Option<u32>,
        episode: Option<u32>,
        year: Option<u32>,
    ) -> Result<Vec<MediaMatch>>;

    /// Exact movie lookup by an embedded provider id (IMDb or TMDB).
    /// Providers that support it return the single authoritative match;
    /// the default returns nothing so callers fall back to title search.
    async fn lookup_movie(
        &self,
        _imdb_id: Option<&str>,
        _tmdb_id: Option<u64>,
    ) -> Result<Vec<MediaMatch>> {
        Ok(vec![])
    }

    /// Exact TV lookup by an embedded provider id, resolving the given
    /// season/episode when supplied. Default returns nothing (title fallback).
    async fn lookup_tv(
        &self,
        _imdb_id: Option<&str>,
        _tmdb_id: Option<u64>,
        _season: Option<u32>,
        _episode: Option<u32>,
    ) -> Result<Vec<MediaMatch>> {
        Ok(vec![])
    }
}

pub mod tmdb;
pub mod tvmaze;
pub mod omdb;
pub mod tvdb;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movie_display_title() {
        let m = MediaMatch {
            tmdb_id: 1,
            kind: MatchKind::Movie { title: "The Matrix".to_string(), year: Some(1999) },
        };
        assert_eq!(m.display_title(), "The Matrix");
    }

    #[test]
    fn tv_display_title() {
        let m = MediaMatch {
            tmdb_id: 2,
            kind: MatchKind::TvEpisode {
                series_title: "Breaking Bad".to_string(),
                season: Some(1),
                episode: Some(1),
                episode_title: Some("Pilot".to_string()),
            },
        };
        assert_eq!(m.display_title(), "Breaking Bad");
    }
}
