# Design: Noise Stripping and Progressive Fallback Search

**Date:** 2026-05-04  
**Status:** Approved

## Problem

Files with years in their filenames are landing on "✗ No match" because edition/version tags that scene releases place between the title and the year bleed into the search query. For example:

- `Movie.Title.EXTENDED.2020.1080p.mkv` → query `"movie title extended"` → TMDB returns no results

The existing `parse_filename` correctly stops at the year, but includes everything before it — including noise tokens like `EXTENDED`, `REPACK`, `PROPER`, etc.

The existing `MatchState::Ambiguous` / "Pick…" picker was designed for low-confidence results but never fires in practice because TMDB's own ranking means the top result's title string always closely matches the search query, keeping Jaro-Winkler confidence above 0.85.

## Scope

This change addresses the "✗ No match" failure mode only. The picker and `Ambiguous` path are not changed.

## Solution: Option C — Blocklist + Progressive Fallback

Two layers of defence, both in the core, invisible to the UI when they succeed.

### Layer 1 — Noise Token Blocklist (`matcher/mod.rs`)

Add a compile-time `NOISE_TOKENS` list of common pre-year scene tags:

```
REPACK, PROPER, EXTENDED, UNRATED, THEATRICAL, DC, LIMITED,
RERIP, READNFO, INTERNAL, RETAIL, DIRECTORS, CUT, COMPLETE,
DUBBED, SUBBED, HYBRID
```

In `parse_filename`, after collecting title tokens (before joining into `title_query`), filter out any token whose uppercase form exactly matches an entry in this list. All other logic — year detection, separator splitting, season/episode extraction — is unchanged.

**Cost:** zero API calls. Handles the vast majority of real-world scene releases.

### Layer 2 — Progressive Fallback Search (`matcher/mod.rs` + `main.rs`)

Add a `fallback_queries(query: &str) -> Vec<String>` function in `matcher/mod.rs`. It returns the original query followed by progressively shorter versions, each dropping the last word, stopping when only one word remains.

Example (after blocklist already cleaned `extended`):
```
"movie title" → ["movie title", "movie"]
```

In `main.rs`, the async search task becomes a loop over `fallback_queries(query)`. It tries each query in order and returns the first non-empty result set. If all queries return empty, it returns `Ok(vec![])`, which produces `MatchState::Unmatched` as before.

**Cost:** 1–2 extra API calls only for files that initially return no results.

## File Changes

| File | Change |
|------|--------|
| `medianamer-core/src/matcher/mod.rs` | Add `NOISE_TOKENS` blocklist, filter in `parse_filename`, add `fallback_queries` function |
| `medianamer-app/src/main.rs` | Replace single `search_movie` / `search_tv` call with loop over `fallback_queries` |

## Testing

- Unit tests for `parse_filename` covering each noise token variant
- Unit tests for `fallback_queries` verifying correct sequence and minimum length
- Existing tests must continue to pass unchanged
