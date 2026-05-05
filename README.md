# MediaNamer

A Linux desktop application that renames movie and TV episode files using online metadata and technical properties extracted from the files themselves.

![MediaNamer icon](medianamer-app/assets/icon.png)

## What it does

Drop your media files into MediaNamer, click **Match All**, and it looks up the correct title, year, season, episode, and episode title from your chosen metadata provider. It reads the video codec and resolution directly from the file using MediaInfo, then previews the renamed filename using a configurable token template before writing anything to disk.

## Features

- **Multiple metadata providers** — TMDB, OMDB, TVmaze, and TheTVDB, selectable per session
- **MediaInfo integration** — reads codec (AV1, H.265, H.264) and resolution (720p, 1080p, 2160p) directly from the file; ultrawide formats like 3840×1600 correctly classified as 2160p
- **Confidence-based auto-matching** — high-confidence matches are applied automatically; ambiguous matches open a picker dialog
- **Token templates** — fully configurable naming format for both movies and TV episodes
- **Filename sanitisation** — strips characters illegal on FAT32, NTFS, and SMB shares; colons become hyphens
- **System theme** — follows the GNOME light/dark colour scheme automatically
- **Native file picker** — opens the system file chooser via XDG portal; also supports drag and drop on X11/XWayland
- **Per-file and bulk removal** — ✕ button on each row; Clear All in the toolbar

## Screenshots

> Add screenshots here once the UI is finalised.

## Requirements

- Linux (x86\_64)
- [MediaInfo](https://mediaarea.net/en/MediaInfo) CLI — install with your package manager
- At least one metadata provider API key (see [Configuration](#configuration))

```bash
# Debian/Ubuntu
sudo apt install mediainfo

# Fedora/RHEL
sudo dnf install mediainfo

# Arch
sudo pacman -S mediainfo
```

## Installation

### Debian / Ubuntu

```bash
sudo dpkg -i medianamer_0.11.0-1_amd64.deb
```

### RPM (Fedora / openSUSE)

```bash
sudo rpm -i medianamer-app-0.11.0-1.x86_64.rpm
```

### From source

Requires Rust 1.75+ and the `mediainfo` CLI.

```bash
git clone https://github.com/mjaydedecker/MediaNamer.git
cd MediaNamer
cargo build --release -p medianamer-app
# Binary is at target/release/medianamer
```

## Configuration

The config file is created automatically when you first save settings inside the app (⚙ button). You can also create it manually:

**`~/.config/medianamer/config.toml`**

```toml
tmdb_read_access_token = "eyJ..."   # TMDB Read Access Token
omdb_api_key = ""                   # OMDB API key (optional)
tvdb_api_key = ""                   # TheTVDB API key (optional)

movie_source = "Tmdb"               # Tmdb | Omdb
tv_source    = "Tmdb"               # Tmdb | Omdb | Tvmaze | Tvdb

[templates]
movie = "{title} ({year}) ({resolution}) ({codec})"
tv    = "{series} - S{season:02}E{episode:02} - {title} ({codec})"
```

### API keys

| Provider | Scope | Key required | Where to get one |
|---|---|---|---|
| **TMDB** | Movies + TV | Yes — Read Access Token (the long `eyJ…` JWT) | [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api) |
| **OMDB** | Movies + TV | Yes | [omdbapi.com/apikey.aspx](https://www.omdbapi.com/apikey.aspx) — free tier: 1,000 req/day |
| **TVmaze** | TV only | No | — |
| **TheTVDB** | TV only | Yes | [thetvdb.com/dashboard/account/apikey](https://www.thetvdb.com/dashboard/account/apikey) |

## Token reference

| Token | Source | Example |
|---|---|---|
| `{title}` | Metadata | `Fire on the Amazon` / `A Midsummer Night's Dream` |
| `{series}` | Metadata (TV) | `BBC Television Shakespeare` |
| `{year}` | Metadata | `1993` |
| `{season}` | Metadata (TV) | `4` |
| `{season:02}` | Metadata (TV) | `04` |
| `{episode}` | Metadata (TV) | `3` |
| `{episode:02}` | Metadata (TV) | `03` |
| `{resolution}` | MediaInfo | `1080p` / `2160p` / `720p` |
| `{codec}` | MediaInfo | `AV1` / `H.265` / `H.264` |
| `{ext}` | MediaInfo | `mkv` / `mp4` |

## Filename sanitisation

Characters that are illegal on FAT32, NTFS, and SMB shares are handled automatically:

- `:` (colon) → `-` (hyphen)
- `\ * ? " < > |` → removed

## Architecture

The project is a Cargo workspace with two crates:

- **`medianamer-core`** — pure business logic with no GUI dependency: MediaInfo wrapper, metadata sources, filename parser, naming engine, renamer pipeline
- **`medianamer-app`** — [iced](https://github.com/iced-rs/iced) 0.13 GUI that wires the core together

All HTTP calls are async via Tokio and reqwest (rustls — no OpenSSL system dependency). TheTVDB JWT tokens are cached per session using `Arc<tokio::sync::Mutex>` shared across concurrent search tasks.

## License

MIT
