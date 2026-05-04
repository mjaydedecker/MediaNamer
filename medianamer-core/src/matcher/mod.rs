use strsim::jaro_winkler;

pub const CONFIDENCE_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, PartialEq)]
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

// Returns a byte offset of the SxxExx marker's start position. This is safe to use
// as a str slice boundary because S/s is ASCII (single byte), which is always a
// valid UTF-8 char boundary even in filenames with multi-byte characters before it.
fn extract_season_episode(s: &str) -> (Option<u32>, Option<u32>, usize) {
    let bytes = s.as_bytes();
    let mut i = 0;
    // Only SxxExx patterns are matched; bare Exx (no season) is not supported in v1.
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
        assert_eq!(p.episode, None);
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
