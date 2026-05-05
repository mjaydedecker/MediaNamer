use strsim::jaro_winkler;

const NOISE_TOKENS: &[&str] = &[
    "REPACK", "PROPER", "EXTENDED", "UNRATED", "THEATRICAL", "DC", "LIMITED",
    "RERIP", "READNFO", "INTERNAL", "RETAIL", "DIRECTORS", "CUT", "COMPLETE",
    "DUBBED", "SUBBED", "HYBRID",
];

pub const CONFIDENCE_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFilename {
    pub title_query: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
}

pub fn parse_filename(filename: &str) -> ParsedFilename {
    let stem = strip_video_ext(filename);
    let (season, episode, se_end) = extract_season_episode(stem);
    let pre_se = &stem[..se_end];

    // Strip parentheses so "(2006)" becomes "2006" and is recognised as the year
    // boundary. This handles both scene releases (bare year) and our own output
    // format where year, resolution and codec are wrapped in parens.
    let normalized = pre_se.replace(['(', ')'], " ");

    // Split on common separators and stop at the first year token.
    let title_tokens: Vec<&str> = normalized
        .split(['.', '_', '-', ' '])
        .filter(|t| !t.is_empty())
        .take_while(|t| !is_year(t))
        // noise filter runs after year boundary so no NOISE_TOKEN can shadow is_year
        .filter(|t| {
            let upper = t.to_uppercase();
            !NOISE_TOKENS.contains(&upper.as_str())
        })
        .collect();

    ParsedFilename {
        title_query: title_tokens.join(" ").to_lowercase(),
        season,
        episode,
    }
}

pub fn score(query: &str, candidate: &str) -> f64 {
    jaro_winkler(&query.to_lowercase(), &candidate.to_lowercase())
}

pub fn fallback_queries(query: &str) -> Vec<String> {
    let words: Vec<&str> = query.split_whitespace().collect();
    (1..=words.len()).rev().map(|n| words[..n].join(" ")).collect()
}

fn strip_video_ext(filename: &str) -> &str {
    const VIDEO_EXTS: &[&str] = &["mkv", "mp4", "avi", "mov", "m4v", "wmv", "flv", "webm"];
    if let Some((stem, ext)) = filename.rsplit_once('.') {
        if VIDEO_EXTS.iter().any(|&v| v.eq_ignore_ascii_case(ext)) {
            return stem;
        }
    }
    filename
}

fn is_year(t: &str) -> bool {
    t.len() == 4
        && t.chars().all(|c| c.is_ascii_digit())
        && (t.starts_with("19") || t.starts_with("20"))
}

// Returns (season, episode, byte_offset_of_SxxExx_start).
// The byte offset is safe as a str slice boundary because S/s is ASCII.
fn extract_season_episode(s: &str) -> (Option<u32>, Option<u32>, usize) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] == b'S' || bytes[i] == b's') && i + 2 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() { j += 1; }
            if j < bytes.len() && (bytes[j] == b'E' || bytes[j] == b'e') {
                let season_str = &s[i + 1..j];
                let mut k = j + 1;
                while k < bytes.len() && bytes[k].is_ascii_digit() { k += 1; }
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
    fn parses_scene_release_with_tags() {
        let p = parse_filename("Mission.Impossible.III.2006.4K.HDR.DV.2160p.BDRemux.Ita.Eng.x265-NAHOM.mkv");
        assert_eq!(p.title_query, "mission impossible iii");
        assert_eq!(p.season, None);
    }

    #[test]
    fn parses_scene_release_imax_atmos() {
        let p = parse_filename("Top.Gun.Maverick.2022.IMAX.2160p.BDRip.TrueHD.7.1.Atmos.DV.HDR10Plus.x265.10bit-MarkII.mkv");
        assert_eq!(p.title_query, "top gun maverick");
        assert_eq!(p.season, None);
    }

    #[test]
    fn parses_scene_release_no_ext() {
        // File stem already stripped by file_stem() before reaching parse_filename
        let p = parse_filename("Transformers.Rise.of.the.Beasts.2023.2160p.BDRip.TrueHD.7.1.Atmos.DV.HDR10.x265.10bit-MarkII");
        assert_eq!(p.title_query, "transformers rise of the beasts");
    }

    #[test]
    fn high_confidence_above_threshold() {
        assert!(score("fire on the amazon", "Fire on the Amazon") >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn low_confidence_below_threshold() {
        assert!(score("bbc shakespeare", "Shakespeare Uncovered") < CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn parses_own_output_format() {
        // Files already renamed by MediaNamer — year is in parens
        let p = parse_filename("Mission: Impossible III (2006) (1080p) (AV1).mkv");
        assert_eq!(p.title_query, "mission: impossible iii");

        let p = parse_filename("Top Gun: Maverick (2022) (1080p) (AV1).mkv");
        assert_eq!(p.title_query, "top gun: maverick");

        let p = parse_filename("Transformers: Rise of the Beasts (2023) (1080p) (AV1)");
        assert_eq!(p.title_query, "transformers: rise of the beasts");
    }

    #[test]
    fn confidence_with_colon_in_tmdb_title() {
        // TMDB titles often have colons; score should still pass threshold
        assert!(score("top gun maverick", "Top Gun: Maverick") >= CONFIDENCE_THRESHOLD);
        assert!(score("mission impossible iii", "Mission: Impossible III") >= CONFIDENCE_THRESHOLD);
        assert!(score("transformers rise of the beasts", "Transformers: Rise of the Beasts") >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn strips_extended_noise_token() {
        let p = parse_filename("Movie.Title.EXTENDED.2020.1080p.mkv");
        assert_eq!(p.title_query, "movie title");
    }

    #[test]
    fn strips_repack_noise_token() {
        let p = parse_filename("Some.Movie.REPACK.2019.BluRay.mkv");
        assert_eq!(p.title_query, "some movie");
    }

    #[test]
    fn strips_proper_noise_token() {
        let p = parse_filename("Another.Film.PROPER.2021.mkv");
        assert_eq!(p.title_query, "another film");
    }

    #[test]
    fn preserves_non_noise_tokens() {
        let p = parse_filename("Fire.on.the.Amazon.1993.1080p.AV1.mkv");
        assert_eq!(p.title_query, "fire on the amazon");
    }

    #[test]
    fn fallback_queries_single_word() {
        assert_eq!(fallback_queries("avatar"), vec!["avatar"]);
    }

    #[test]
    fn fallback_queries_two_words() {
        assert_eq!(
            fallback_queries("movie title"),
            vec!["movie title", "movie"]
        );
    }

    #[test]
    fn fallback_queries_three_words() {
        assert_eq!(
            fallback_queries("the dark knight"),
            vec!["the dark knight", "the dark", "the"]
        );
    }

    #[test]
    fn fallback_queries_empty_string() {
        assert_eq!(fallback_queries(""), Vec::<String>::new());
    }

    #[test]
    fn fallback_queries_whitespace_only() {
        assert_eq!(fallback_queries("   "), Vec::<String>::new());
    }
}
