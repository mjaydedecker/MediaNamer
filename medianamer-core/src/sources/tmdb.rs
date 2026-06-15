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

fn year_from_date(date: &Option<String>) -> Option<u32> {
    date.as_deref()
        .and_then(|d| d.split('-').next())
        .and_then(|y| y.parse().ok())
}

#[async_trait]
impl MediaSource for TmdbSource {
    fn name(&self) -> &str { "TMDB" }

    async fn search_movie(&self, query: &str, year: Option<u32>) -> Result<Vec<MediaMatch>> {
        let url = format!("{}/3/search/movie", self.base_url);
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];
        if let Some(y) = year {
            params.push(("primary_release_year", y.to_string()));
        }
        let resp: MovieSearchResponse = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .query(&params)
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
        year: Option<u32>,
    ) -> Result<Vec<MediaMatch>> {
        let url = format!("{}/3/search/tv", self.base_url);
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];
        if let Some(y) = year {
            params.push(("first_air_date_year", y.to_string()));
        }
        let resp: TvSearchResponse = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .query(&params)
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

        if let (Some(s), Some(e)) = (season, episode) {
            let ep_url = format!(
                "{}/3/tv/{}/season/{}/episode/{}",
                self.base_url, series.id, s, e
            );
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
                    season: Some(ep.season_number),
                    episode: Some(ep.episode_number),
                    episode_title: Some(ep.name),
                },
            }]);
        }

        Ok(resp.results.into_iter().map(|r| MediaMatch {
            tmdb_id: r.id,
            kind: MatchKind::TvEpisode {
                series_title: r.name,
                season: None,
                episode: None,
                episode_title: None,
            },
        }).collect())
    }
}
