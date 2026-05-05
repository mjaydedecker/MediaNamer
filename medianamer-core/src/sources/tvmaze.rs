use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::{Error, Result};
use super::{MatchKind, MediaMatch, MediaSource};

pub struct TvmazeSource {
    base_url: String,
    client: Client,
}

impl TvmazeSource {
    pub fn new() -> Self {
        Self { base_url: "https://api.tvmaze.com".to_string(), client: Client::new() }
    }
    pub fn new_with_base_url(base_url: &str) -> Self {
        Self { base_url: base_url.trim_end_matches('/').to_string(), client: Client::new() }
    }
}

#[derive(Deserialize)]
struct ShowResult { show: ShowInfo }
#[derive(Deserialize)]
struct ShowInfo { id: u64, name: String }
#[derive(Deserialize)]
struct EpisodeInfo { name: String, season: u32, number: u32 }

#[async_trait]
impl MediaSource for TvmazeSource {
    fn name(&self) -> &str { "TVmaze" }

    async fn search_movie(&self, _query: &str) -> Result<Vec<MediaMatch>> {
        Ok(vec![])
    }

    async fn search_tv(&self, query: &str, season: Option<u32>, episode: Option<u32>) -> Result<Vec<MediaMatch>> {
        let url = format!("{}/search/shows", self.base_url);
        let results: Vec<ShowResult> = self.client
            .get(&url)
            .query(&[("q", query)])
            .send().await?
            .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
            .json().await?;

        if results.is_empty() { return Ok(vec![]); }
        let show = &results[0].show;

        if let (Some(s), Some(e)) = (season, episode) {
            let ep_url = format!("{}/shows/{}/episodebynumber", self.base_url, show.id);
            let ep: EpisodeInfo = self.client
                .get(&ep_url)
                .query(&[("season", s), ("number", e)])
                .send().await?
                .error_for_status().map_err(|e| Error::Tmdb(e.to_string()))?
                .json().await?;
            return Ok(vec![MediaMatch {
                tmdb_id: show.id,
                kind: MatchKind::TvEpisode {
                    series_title: show.name.clone(),
                    season: Some(ep.season),
                    episode: Some(ep.number),
                    episode_title: Some(ep.name),
                },
            }]);
        }

        Ok(results.into_iter().map(|r| MediaMatch {
            tmdb_id: r.show.id,
            kind: MatchKind::TvEpisode {
                series_title: r.show.name,
                season: None, episode: None, episode_title: None,
            },
        }).collect())
    }
}
