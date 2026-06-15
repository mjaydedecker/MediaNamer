use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{Error, Result};
use super::{MatchKind, MediaMatch, MediaSource};

pub struct TvdbSource {
    api_key: String,
    base_url: String,
    client: Client,
    token: Arc<Mutex<Option<String>>>,
}

impl TvdbSource {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api4.thetvdb.com".to_string(),
            client: Client::new(),
            token: Arc::new(Mutex::new(None)),
        }
    }
    pub fn new_with_base_url(api_key: impl Into<String>, base_url: &str) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            token: Arc::new(Mutex::new(None)),
        }
    }

    async fn bearer_token(&self) -> Result<String> {
        let mut guard = self.token.lock().await;
        if let Some(t) = guard.as_ref() {
            return Ok(t.clone());
        }
        #[derive(Serialize)]
        struct LoginReq<'a> { apikey: &'a str }
        #[derive(Deserialize)]
        struct LoginResp { data: TokenData }
        #[derive(Deserialize)]
        struct TokenData { token: String }

        let resp: LoginResp = self.client
            .post(format!("{}/v4/login", self.base_url))
            .json(&LoginReq { apikey: &self.api_key })
            .send().await?
            .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
            .json().await?;

        *guard = Some(resp.data.token.clone());
        Ok(resp.data.token)
    }
}

#[derive(Deserialize)]
struct SearchResp { data: Vec<SearchItem> }
#[derive(Deserialize)]
struct SearchItem {
    tvdb_id: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct EpisodesResp { data: EpisodesData }
#[derive(Deserialize)]
struct EpisodesData { episodes: Option<Vec<EpisodeItem>> }
#[derive(Deserialize)]
struct EpisodeItem {
    name: Option<String>,
    #[serde(rename = "seasonNumber")]
    season_number: Option<u32>,
    number: Option<u32>,
}

#[async_trait]
impl MediaSource for TvdbSource {
    fn name(&self) -> &str { "TheTVDB" }

    async fn search_movie(&self, _query: &str, _year: Option<u32>) -> Result<Vec<MediaMatch>> {
        Ok(vec![])
    }

    async fn search_tv(&self, query: &str, season: Option<u32>, episode: Option<u32>, year: Option<u32>) -> Result<Vec<MediaMatch>> {
        let token = self.bearer_token().await?;

        let search_url = format!("{}/v4/search", self.base_url);
        let mut params: Vec<(&str, String)> = vec![
            ("query", query.to_string()),
            ("type", "series".to_string()),
        ];
        if let Some(y) = year {
            params.push(("year", y.to_string()));
        }
        let resp: SearchResp = self.client
            .get(&search_url)
            .bearer_auth(&token)
            .query(&params)
            .send().await?
            .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
            .json().await?;

        if resp.data.is_empty() { return Ok(vec![]); }
        let item = &resp.data[0];
        let series_id: u64 = item.tvdb_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
        let series_name = item.name.clone().unwrap_or_default();

        if let (Some(s), Some(e)) = (season, episode) {
            let ep_url = format!("{}/v4/series/{}/episodes/default", self.base_url, series_id);
            let ep_resp: EpisodesResp = self.client
                .get(&ep_url)
                .bearer_auth(&token)
                .query(&[("season", s)])
                .send().await?
                .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
                .json().await?;

            let matched = ep_resp.data.episodes.unwrap_or_default()
                .into_iter()
                .find(|ep| ep.number == Some(e));

            return Ok(vec![MediaMatch {
                tmdb_id: series_id,
                kind: MatchKind::TvEpisode {
                    series_title: series_name,
                    season: Some(s),
                    episode: Some(e),
                    episode_title: matched.and_then(|ep| ep.name),
                },
            }]);
        }

        Ok(resp.data.into_iter().map(|r| MediaMatch {
            tmdb_id: r.tvdb_id.as_deref().unwrap_or("0").parse().unwrap_or(0),
            kind: MatchKind::TvEpisode {
                series_title: r.name.unwrap_or_default(),
                season: None, episode: None, episode_title: None,
            },
        }).collect())
    }
}
