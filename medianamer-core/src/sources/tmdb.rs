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

#[derive(Deserialize)]
struct MovieDetail {
    id: u64,
    title: String,
    release_date: Option<String>,
}

#[derive(Deserialize)]
struct TvDetail {
    id: u64,
    name: String,
}

/// Response of `/3/find/{external_id}` (external-id reverse lookup).
#[derive(Deserialize)]
struct FindResponse {
    #[serde(default)]
    movie_results: Vec<MovieResult>,
    #[serde(default)]
    tv_results: Vec<TvResult>,
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

    async fn lookup_movie(
        &self,
        imdb_id: Option<&str>,
        tmdb_id: Option<u64>,
    ) -> Result<Vec<MediaMatch>> {
        if let Some(id) = tmdb_id {
            let url = format!("{}/3/movie/{}", self.base_url, id);
            let m: MovieDetail = self
                .client
                .get(&url)
                .bearer_auth(&self.api_key)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| Error::Tmdb(e.to_string()))?
                .json()
                .await?;
            return Ok(vec![MediaMatch {
                tmdb_id: m.id,
                kind: MatchKind::Movie {
                    title: m.title,
                    year: year_from_date(&m.release_date),
                },
            }]);
        }

        if let Some(imdb) = imdb_id {
            let url = format!("{}/3/find/{}", self.base_url, imdb);
            let resp: FindResponse = self
                .client
                .get(&url)
                .bearer_auth(&self.api_key)
                .query(&[("external_source", "imdb_id")])
                .send()
                .await?
                .error_for_status()
                .map_err(|e| Error::Tmdb(e.to_string()))?
                .json()
                .await?;
            return Ok(resp
                .movie_results
                .into_iter()
                .map(|r| MediaMatch {
                    tmdb_id: r.id,
                    kind: MatchKind::Movie {
                        title: r.title,
                        year: year_from_date(&r.release_date),
                    },
                })
                .collect());
        }

        Ok(vec![])
    }

    async fn lookup_tv(
        &self,
        imdb_id: Option<&str>,
        tmdb_id: Option<u64>,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> Result<Vec<MediaMatch>> {
        // Resolve the series (id + name) from whichever id we have.
        let series = if let Some(id) = tmdb_id {
            let url = format!("{}/3/tv/{}", self.base_url, id);
            let d: TvDetail = self
                .client
                .get(&url)
                .bearer_auth(&self.api_key)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| Error::Tmdb(e.to_string()))?
                .json()
                .await?;
            Some((d.id, d.name))
        } else if let Some(imdb) = imdb_id {
            let url = format!("{}/3/find/{}", self.base_url, imdb);
            let resp: FindResponse = self
                .client
                .get(&url)
                .bearer_auth(&self.api_key)
                .query(&[("external_source", "imdb_id")])
                .send()
                .await?
                .error_for_status()
                .map_err(|e| Error::Tmdb(e.to_string()))?
                .json()
                .await?;
            resp.tv_results.into_iter().next().map(|t| (t.id, t.name))
        } else {
            None
        };

        let Some((series_id, series_name)) = series else {
            return Ok(vec![]);
        };

        if let (Some(s), Some(e)) = (season, episode) {
            let ep_url = format!(
                "{}/3/tv/{}/season/{}/episode/{}",
                self.base_url, series_id, s, e
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
                tmdb_id: series_id,
                kind: MatchKind::TvEpisode {
                    series_title: series_name,
                    season: Some(ep.season_number),
                    episode: Some(ep.episode_number),
                    episode_title: Some(ep.name),
                },
            }]);
        }

        Ok(vec![MediaMatch {
            tmdb_id: series_id,
            kind: MatchKind::TvEpisode {
                series_title: series_name,
                season: None,
                episode: None,
                episode_title: None,
            },
        }])
    }
}
