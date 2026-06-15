use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::{Error, Result};
use super::{MatchKind, MediaMatch, MediaSource};

pub struct OmdbSource {
    api_key: String,
    base_url: String,
    client: Client,
}

impl OmdbSource {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), base_url: "https://www.omdbapi.com".to_string(), client: Client::new() }
    }
    pub fn new_with_base_url(api_key: impl Into<String>, base_url: &str) -> Self {
        Self { api_key: api_key.into(), base_url: base_url.trim_end_matches('/').to_string(), client: Client::new() }
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(rename = "Search")]
    search: Option<Vec<SearchResult>>,
}

#[derive(Deserialize)]
struct SearchResult {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "imdbID")]
    imdb_id: String,
    #[serde(rename = "Type")]
    media_type: String,
}

#[derive(Deserialize)]
struct EpisodeResponse {
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Season")]
    season: Option<String>,
    #[serde(rename = "Episode")]
    episode: Option<String>,
    #[serde(rename = "seriesID")]
    series_id: Option<String>,
    #[serde(rename = "Response")]
    response: String,
}

fn imdb_to_id(imdb_id: &str) -> u64 {
    imdb_id.trim_start_matches("tt").parse().unwrap_or(0)
}

fn year_from_str(s: &Option<String>) -> Option<u32> {
    s.as_deref()?.chars().take(4).collect::<String>().parse().ok()
}

#[async_trait]
impl MediaSource for OmdbSource {
    fn name(&self) -> &str { "OMDB" }

    async fn search_movie(&self, query: &str, year: Option<u32>) -> Result<Vec<MediaMatch>> {
        let mut params: Vec<(&str, String)> = vec![
            ("apikey", self.api_key.clone()),
            ("s", query.to_string()),
            ("type", "movie".to_string()),
        ];
        if let Some(y) = year {
            params.push(("y", y.to_string()));
        }
        let resp: SearchResponse = self.client
            .get(&self.base_url)
            .query(&params)
            .send().await?
            .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
            .json().await?;

        Ok(resp.search.unwrap_or_default().into_iter().map(|r| MediaMatch {
            tmdb_id: imdb_to_id(&r.imdb_id),
            kind: MatchKind::Movie { title: r.title, year: year_from_str(&r.year) },
        }).collect())
    }

    async fn search_tv(&self, query: &str, season: Option<u32>, episode: Option<u32>, year: Option<u32>) -> Result<Vec<MediaMatch>> {
        let mut params: Vec<(&str, String)> = vec![
            ("apikey", self.api_key.clone()),
            ("s", query.to_string()),
            ("type", "series".to_string()),
        ];
        if let Some(y) = year {
            params.push(("y", y.to_string()));
        }
        let resp: SearchResponse = self.client
            .get(&self.base_url)
            .query(&params)
            .send().await?
            .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
            .json().await?;

        let results = resp.search.unwrap_or_default();
        if results.is_empty() { return Ok(vec![]); }
        let series = &results[0];

        if let (Some(s), Some(e)) = (season, episode) {
            let ep: EpisodeResponse = self.client
                .get(&self.base_url)
                .query(&[
                    ("apikey", self.api_key.clone()),
                    ("i", series.imdb_id.clone()),
                    ("Season", s.to_string()),
                    ("Episode", e.to_string()),
                ])
                .send().await?
                .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
                .json().await?;

            if ep.response != "True" { return Ok(vec![]); }
            return Ok(vec![MediaMatch {
                tmdb_id: imdb_to_id(&series.imdb_id),
                kind: MatchKind::TvEpisode {
                    series_title: series.title.clone(),
                    season: ep.season.as_deref().and_then(|s| s.parse().ok()),
                    episode: ep.episode.as_deref().and_then(|e| e.parse().ok()),
                    episode_title: ep.title,
                },
            }]);
        }

        Ok(results.into_iter().map(|r| MediaMatch {
            tmdb_id: imdb_to_id(&r.imdb_id),
            kind: MatchKind::TvEpisode {
                series_title: r.title,
                season: None, episode: None, episode_title: None,
            },
        }).collect())
    }
}
