use crate::sources::{MatchKind, MediaMatch};
use crate::mediainfo::MediaInfo;

#[derive(Debug, Clone, PartialEq)]
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
                year: *year,
                season: None,
                episode: None,
                resolution: media_info.resolution.clone(),
                codec: media_info.codec.clone(),
                ext: media_info.extension.clone(),
            },
            MatchKind::TvEpisode { series_title, season, episode, episode_title } => TokenValues {
                title: episode_title.clone(),
                series: Some(series_title.clone()),
                year: None,
                season: *season,
                episode: *episode,
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
        "title"      => v.title.clone().ok_or_else(|| "{title} not available for this media type".to_string()),
        "series"     => v.series.clone().ok_or_else(|| "{series} not available for this media type".to_string()),
        "year"       => v.year.map(|y| y.to_string()).ok_or_else(|| "{year} not available for this media type".to_string()),
        "season"     => v.season.map(|s| s.to_string()).ok_or_else(|| "{season} not available for this media type".to_string()),
        "season:02"  => v.season.map(|s| format!("{:02}", s)).ok_or_else(|| "{season:02} not available".to_string()),
        "episode"    => v.episode.map(|e| e.to_string()).ok_or_else(|| "{episode} not available".to_string()),
        "episode:02" => v.episode.map(|e| format!("{:02}", e)).ok_or_else(|| "{episode:02} not available".to_string()),
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
        assert!(err.contains("bogus"));
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
