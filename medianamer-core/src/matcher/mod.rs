use strsim::jaro_winkler;

const NOISE_TOKENS: &[&str] = &[
    "REPACK", "PROPER", "EXTENDED", "UNRATED", "THEATRICAL", "LIMITED",
    "RERIP", "READNFO", "INTERNAL", "RETAIL", "DIRECTORS", "CUT", "COMPLETE",
    "DUBBED", "SUBBED", "HYBRID",
];

pub const CONFIDENCE_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFilename {
    pub title_query: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub year: Option<u32>,
    /// IMDb id embedded in the filename (e.g. `{imdb-tt0133093}`), if any.
    pub imdb_id: Option<String>,
    /// TMDB id embedded in the filename (e.g. `{tmdb-603}`), if any.
    pub tmdb_id: Option<u64>,
}

pub fn parse_filename(filename: &str) -> ParsedFilename {
    let stem = strip_video_ext(filename);
    // IDs are scanned across the whole stem — they usually sit at the end,
    // after the SxxExx marker (Plex/Sonarr/Radarr style).
    let imdb_id = extract_imdb_id(stem);
    let tmdb_id = extract_tmdb_id(stem);
    let (season, episode, se_end) = extract_season_episode(stem);
    let pre_se = &stem[..se_end];

    // Strip parentheses so "(2006)" becomes "2006" and is recognised as the year
    // boundary. This handles both scene releases (bare year) and our own output
    // format where year, resolution and codec are wrapped in parens.
    let normalized = pre_se.replace(['(', ')'], " ");

    let tokens: Vec<&str> = normalized
        .split(['.', '_', '-', ' '])
        .filter(|t| !t.is_empty())
        .collect();

    // The title ends at the first year token; that token doubles as the release
    // year we hand to the metadata APIs to disambiguate remakes / same-named
    // series (e.g. "The Lion King" 1994 vs 2019).
    let year = tokens.iter().find(|t| is_year(t)).and_then(|t| t.parse::<u32>().ok());

    let title_tokens: Vec<&str> = tokens
        .into_iter()
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
        year,
        imdb_id,
        tmdb_id,
    }
}

pub fn score(query: &str, candidate: &str) -> f64 {
    jaro_winkler(&query.to_lowercase(), &candidate.to_lowercase())
}

/// Title similarity adjusted by a release-year penalty. When both the parsed
/// filename and the candidate carry a year, a mismatch lowers the score so that
/// the right entry from a same-named set (remakes, reboots) ranks first without
/// hard-filtering candidates out entirely. Missing years leave the base score
/// untouched (no evidence either way).
pub fn match_confidence(
    query: &str,
    query_year: Option<u32>,
    candidate: &str,
    candidate_year: Option<u32>,
) -> f64 {
    let base = score(query, candidate);
    match (query_year, candidate_year) {
        (Some(q), Some(c)) => {
            let penalty = match q.abs_diff(c) {
                0 => 0.0,
                1 => 0.05,
                _ => 0.20,
            };
            (base - penalty).max(0.0)
        }
        _ => base,
    }
}

/// Extracts an embedded IMDb id (e.g. `tt0133093`) from anywhere in the stem.
/// Requires 7-8 digits after `tt` and a non-alphanumeric boundary before it so
/// it doesn't fire inside ordinary words.
fn extract_imdb_id(s: &str) -> Option<String> {
    let lower = s.to_ascii_lowercase();
    let b = lower.as_bytes();
    let mut i = 0;
    while i + 2 < b.len() {
        let boundary = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        if boundary && b[i] == b't' && b[i + 1] == b't' && b[i + 2].is_ascii_digit() {
            let mut j = i + 2;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            let digits = j - (i + 2);
            if (7..=8).contains(&digits) {
                return Some(lower[i..j].to_string());
            }
        }
        i += 1;
    }
    None
}

/// Extracts an embedded TMDB id from Plex/Sonarr/Radarr style tags such as
/// `{tmdb-603}`, `[tmdbid-603]` or `tmdb-603`. A separator (or the `id`
/// suffix) is required between `tmdb` and the digits so a bare "tmdb" word
/// followed by a space + year can't be misread as an id.
fn extract_tmdb_id(s: &str) -> Option<u64> {
    let lower = s.to_ascii_lowercase();
    let idx = lower.find("tmdb")?;
    let rest = &lower[idx + 4..];
    let rest = rest.strip_prefix("id").unwrap_or(rest);
    let rest = rest.strip_prefix(['-', '_', ':']).unwrap_or(rest);
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
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
        assert_eq!(p.year, Some(1993));
    }

    #[test]
    fn captures_year_from_own_output_format() {
        let p = parse_filename("Top Gun: Maverick (2022) (1080p) (AV1).mkv");
        assert_eq!(p.title_query, "top gun: maverick");
        assert_eq!(p.year, Some(2022));
    }

    #[test]
    fn year_is_none_when_absent() {
        let p = parse_filename("Some.Untagged.Movie.1080p.mkv");
        assert_eq!(p.year, None);
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

    #[test]
    fn preserves_dc_prefix_title() {
        let p = parse_filename("DC.League.of.Super.Pets.2022.1080p.mkv");
        assert_eq!(p.title_query, "dc league of super pets");
    }

    #[test]
    fn extracts_imdb_id_from_plex_tag() {
        let p = parse_filename("The Matrix (1999) {imdb-tt0133093}.mkv");
        assert_eq!(p.imdb_id, Some("tt0133093".to_string()));
        assert_eq!(p.title_query, "the matrix");
        assert_eq!(p.year, Some(1999));
    }

    #[test]
    fn extracts_imdb_id_8_digits() {
        let p = parse_filename("Some.Show.S01E01.tt12345678.mkv");
        assert_eq!(p.imdb_id, Some("tt12345678".to_string()));
    }

    #[test]
    fn no_imdb_id_when_absent() {
        let p = parse_filename("The Matrix (1999).mkv");
        assert_eq!(p.imdb_id, None);
    }

    #[test]
    fn imdb_id_not_matched_inside_word() {
        // "tt" embedded in a word with too few/no trailing digits must not match
        let p = parse_filename("Scott.Pilgrim.2010.1080p.mkv");
        assert_eq!(p.imdb_id, None);
    }

    #[test]
    fn extracts_tmdb_id_plex_tag() {
        let p = parse_filename("The Matrix (1999) {tmdb-603}.mkv");
        assert_eq!(p.tmdb_id, Some(603));
    }

    #[test]
    fn extracts_tmdb_id_radarr_tag() {
        let p = parse_filename("The Matrix (1999) [tmdbid-603].mkv");
        assert_eq!(p.tmdb_id, Some(603));
    }

    #[test]
    fn no_tmdb_id_when_absent() {
        let p = parse_filename("The Matrix (1999).mkv");
        assert_eq!(p.tmdb_id, None);
    }

    #[test]
    fn tmdb_word_with_space_year_is_not_an_id() {
        let p = parse_filename("tmdb 2019 documentary.mkv");
        assert_eq!(p.tmdb_id, None);
    }

    #[test]
    fn match_confidence_same_year_unpenalised() {
        let same = match_confidence("the lion king", Some(1994), "The Lion King", Some(1994));
        assert!(same >= CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn match_confidence_year_mismatch_demotes_remake() {
        let original = match_confidence("the lion king", Some(1994), "The Lion King", Some(1994));
        let remake = match_confidence("the lion king", Some(1994), "The Lion King", Some(2019));
        assert!(original > remake);
        assert!((original - remake - 0.20).abs() < 1e-9);
    }

    #[test]
    fn match_confidence_off_by_one_small_penalty() {
        let exact = match_confidence("foo", Some(2000), "foo", Some(2000));
        let off_by_one = match_confidence("foo", Some(2000), "foo", Some(2001));
        assert!((exact - off_by_one - 0.05).abs() < 1e-9);
    }

    #[test]
    fn match_confidence_missing_year_uses_base_score() {
        let with = match_confidence("foo bar", None, "Foo Bar", Some(2019));
        let base = score("foo bar", "Foo Bar");
        assert!((with - base).abs() < 1e-9);
    }
}
