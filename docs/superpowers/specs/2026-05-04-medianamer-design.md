# MediaNamer — Functional Specification

**Date:** 2026-05-04
**Status:** Approved

---

## 1. Overview

MediaNamer is a Linux desktop application written in Rust that renames movie and TV episode files using metadata from online sources and technical properties extracted from the files themselves. The user drags files into the application, the app matches them against TMDB, previews the renamed filenames using a user-defined token template, and performs the renames on approval.

---

## 2. Goals & Scope

### In Scope (v1)
- GUI application built with the **iced** toolkit (Elm-style reactive architecture)
- Drag-and-drop file loading
- Metadata source: **TMDB** (movies and TV episodes)
- Technical metadata extraction via **MediaInfo** CLI (codec, resolution, container)
- Token-based filename template system with separate templates for movies and TV episodes
- Confidence-based auto-matching with manual picker for ambiguous results
- Batch rename of all matched files in one action, with per-file preview before committing
- Distribution packages for Debian/Ubuntu, Arch Linux, and Red Hat/Fedora

### Out of Scope (v1)
- Additional metadata sources (IMDB, TheTVDB, TVmaze) — designed for future addition
- Command-line interface
- Windows or macOS support

---

## 3. Architecture

The project is a **Cargo workspace** with two crates:

```
medianamer/
├── Cargo.toml                  # workspace root
├── medianamer-core/            # business logic, no UI dependencies
│   ├── src/
│   │   ├── lib.rs
│   │   ├── sources/            # MediaSource trait + TmdbSource
│   │   ├── mediainfo/          # MediaInfo CLI wrapper
│   │   ├── matcher/            # filename parsing & confidence scoring
│   │   ├── naming/             # token parser & formatter
│   │   └── renamer/            # pipeline orchestration & filesystem ops
│   └── tests/                  # integration tests with HTTP fixtures
└── medianamer-app/             # iced GUI
    ├── src/
    │   ├── main.rs
    │   ├── state.rs            # application state
    │   └── ui/                 # iced views and message handlers
    └── assets/
        ├── icon.png
        └── medianamer.desktop
```

**Configuration** is stored at `~/.config/medianamer/config.toml` and holds the TMDB API key and saved naming templates.

---

## 4. Core Data Models

### `MediaFile`
Represents a file loaded by the user.

| Field | Type | Description |
|---|---|---|
| `path` | `PathBuf` | Absolute path to the file |
| `media_info` | `MediaInfo` | Codec, resolution, container from MediaInfo |
| `match_state` | `MatchState` | Current matching status |

```
MatchState = Unmatched
           | Matched(MediaMatch)
           | Ambiguous(Vec<MediaMatch>)
           | Error(String)
```

### `MediaInfo`
Extracted from the file via the MediaInfo CLI.

| Field | Type | Example |
|---|---|---|
| `codec` | `String` | `"AV1"`, `"H.265"`, `"H.264"` |
| `resolution` | `String` | `"1080p"`, `"4K"`, `"720p"` |
| `extension` | `String` | `"mkv"`, `"mp4"` |

Resolution is derived from video height: ≥2160 → `4K`, ≥1080 → `1080p`, ≥720 → `720p`, otherwise the raw height value.

### `MediaMatch`
A result returned by a `MediaSource`.

**Movie fields:** `tmdb_id`, `title`, `year`

**TV episode fields:** `tmdb_id`, `series_title`, `season`, `episode`, `episode_title`

### `NamingTemplate`
A user-defined format string. Two are stored in config — one for movies, one for TV episodes.

Examples:
- Movie: `{title} ({year}) ({resolution}) ({codec})`
- TV: `{series} - S{season:02}E{episode:02} - {title} ({codec})`

### `RenameJob`
A `MediaFile` paired with a confirmed `MediaMatch`, ready for filesystem execution.

---

## 5. MediaSource Trait

All metadata sources implement this trait in `medianamer-core::sources`:

```rust
#[async_trait]
pub trait MediaSource: Send + Sync {
    fn name(&self) -> &str;
    async fn search_movie(&self, query: &str) -> Result<Vec<MediaMatch>>;
    async fn search_tv(&self, query: &str, season: Option<u32>, episode: Option<u32>) -> Result<Vec<MediaMatch>>;
}
```

`TmdbSource` is the only implementation in v1. Future sources (IMDB, TheTVDB, TVmaze) implement the same trait with no changes to the pipeline.

**TMDB API endpoints used:**
- `GET /search/movie?query=…` — movie search
- `GET /search/tv?query=…` — TV series search
- `GET /tv/{id}/season/{s}/episode/{e}` — episode detail lookup

The TMDB API key is read from config at startup. If absent, the app shows a setup prompt on first launch.

---

## 6. Matching & Confidence Scoring

The matcher (`medianamer-core::matcher`) parses the original filename to extract a search query by stripping common separators (`.`, `_`, `-`), year patterns, resolution tags, and codec tags.

Results from TMDB are ranked by Jaro-Winkler string similarity between the cleaned filename and the TMDB title.

| Confidence | Behaviour |
|---|---|
| ≥ 0.85 | Auto-select top result, status → `Matched` |
| < 0.85 | Status → `Ambiguous`, user must pick from picker |
| No results | Status → `Unmatched` |

---

## 7. Naming Engine

The token parser (`medianamer-core::naming`) scans the template string for `{token}` patterns and substitutes values from `MediaMatch` and `MediaInfo`.

### Available Tokens

| Token | Source | Example Output |
|---|---|---|
| `{title}` | TMDB (movie title or episode title) | `Fire on the Amazon` |
| `{series}` | TMDB (TV only) | `BBC Television Shakespeare` |
| `{year}` | TMDB | `1993` |
| `{season}` | TMDB (TV only) | `4` |
| `{season:02}` | TMDB (TV only, zero-padded) | `04` |
| `{episode}` | TMDB (TV only) | `3` |
| `{episode:02}` | TMDB (TV only, zero-padded) | `03` |
| `{resolution}` | MediaInfo | `1080p` |
| `{codec}` | MediaInfo | `AV1` |
| `{ext}` | MediaInfo | `mkv` |

Unrecognised tokens render as a visible error inline (e.g. `{unknown_token}` → shown in red in the preview, rename blocked for that file).

The output is sanitised for Linux filenames: null bytes and `/` characters are removed; leading dots and trailing spaces are stripped.

A `?` help button in the format bar opens an inline reference panel listing all tokens with examples.

---

## 8. User Interface

### Layout — Single Panel

```
┌─────────────────────────────────────────────────────────────┐
│ [+ Add Files]  [Match All]  [Rename]        Type: Movies ▾  │  ← Toolbar
├─────────────────────────────────────────────────────────────┤
│ FORMAT  [{title} ({year}) ({resolution}) ({codec})]  [?]    │  ← Format bar
├────────────────────────────┬───────────────────────┬────────┤
│ Original Filename          │ New Filename           │ Status │  ← Column headers
├────────────────────────────┼───────────────────────┼────────┤
│ Fire.on.the.Amazon...mkv   │ Fire on the Amazon... │ ✓      │
│ BBC.Shakespeare.S04E03.mkv │ BBC Television...     │ ?      │  ← Click to pick
│ unknown.movie.2019.mkv     │ (not matched)         │ ✗      │
├────────────────────────────┴───────────────────────┴────────┤
│             Drop files here to add                           │  ← Drop zone hint
└─────────────────────────────────────────────────────────────┘
```

### Status Indicators
- `✓ Matched` (green) — auto-matched with high confidence, preview shown
- `? Review` (amber) — multiple candidates found, click row to open picker dialog
- `✗ None` (red) — no TMDB results found
- `⚠ Error` (red) — MediaInfo or API error, hover for detail

### Match Picker Dialog
Shown when a file's status is `Ambiguous`. Displays a list of TMDB candidates with title, year, and (for TV) series poster thumbnail. User selects one; status moves to `Matched`.

### Rename Confirmation
The `Rename` button is enabled only when at least one file is in `Matched` state. Clicking it renames all `Matched` files and removes them from the list. `Ambiguous` and `Unmatched` files remain.

### Media Type Toggle
A dropdown in the toolbar switches between **Movies** and **TV Episodes**. This determines which naming template is active and which TMDB search endpoint is used. All files in the current session share one media type.

### Settings
Accessible via a gear icon in the toolbar. Contains:
- TMDB API key input
- Saved movie naming template
- Saved TV episode naming template

---

## 9. MediaInfo Integration

MediaNamer shells out to the `mediainfo` CLI with `--Output=JSON` and parses the result. MediaInfo is a **required runtime dependency** — if not found on `PATH`, the app shows a clear error on startup explaining how to install it.

MediaInfo is called once per file when it is loaded, before TMDB matching begins.

---

## 10. Configuration File

Location: `~/.config/medianamer/config.toml`

```toml
tmdb_api_key = "your_key_here"

[templates]
movie = "{title} ({year}) ({resolution}) ({codec})"
tv = "{series} - S{season:02}E{episode:02} - {title} ({codec})"
```

Created with defaults on first launch (API key left blank, prompting the setup flow).

---

## 11. Packaging

All packages ship the `medianamer` binary, a `.desktop` launcher entry, and an application icon.

| Distribution | Format | Tool | Runtime Deps |
|---|---|---|---|
| Debian / Ubuntu | `.deb` | `cargo-deb` | `mediainfo`, `libssl` |
| Arch Linux | `PKGBUILD` (AUR) | manual | `mediainfo` |
| Red Hat / Fedora | `.rpm` | `cargo-generate-rpm` | `mediainfo`, `openssl` |

Packages are built in CI on tagged releases.

---

## 12. Testing Strategy

### Unit Tests (`medianamer-core`)
- Token parser: valid substitution, unknown tokens, zero-padding format specifiers, sanitisation edge cases
- Confidence scorer: known-good matches above threshold, ambiguous pairs below threshold
- Filename parser: common naming patterns (dots, underscores, year in parens, resolution tags)
- MediaInfo JSON parser: codec extraction, resolution bucketing

### Integration Tests (`medianamer-core/tests/`)
- `TmdbSource` tested against recorded HTTP fixtures (using `wiremock`) — no live API key required in CI
- Full pipeline test: fixture file → MediaInfo mock → TMDB fixture → formatted filename

### UI
The iced `update` function is a pure function `(State, Message) -> State`. Message-handling logic is covered by unit tests on `AppState` in `medianamer-app`. No automated UI rendering tests.
