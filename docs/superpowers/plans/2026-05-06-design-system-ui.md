# Design System UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Claude Design handoff (MediaNamer.html) in the Rust/Iced 0.13 codebase — new color palette, status chips, redesigned file table with sortable columns and new-filename preview, summary bar, message bar, styled header, token reference panel, and settings as a modal-style overlay.

**Architecture:** A new `ui/palette.rs` module provides typed light/dark colors used by all UI files. New state fields (`sort_col`, `sort_dir`, `status_msg`, `message_kind`, `show_tokens`) drive the sort indicators, bottom message bar, and token panel toggle. All visual changes are contained to `medianamer-app/src/ui/`; no backend logic changes.

**Tech Stack:** Rust, Iced 0.13 (`container::Style`, `button::Style`, `Border`, `Background`), existing `medianamer-core` naming/state unchanged.

---

### Task 1: Create color palette module

**Files:**
- Create: `medianamer-app/src/ui/palette.rs`
- Modify: `medianamer-app/src/ui/mod.rs` (add `pub mod palette;`)

- [ ] **Step 1: Create `medianamer-app/src/ui/palette.rs`**

```rust
use iced::Color;

pub struct Palette {
    pub bg:         Color,
    pub surface:    Color,
    pub surface2:   Color,
    pub border:     Color,
    pub text:       Color,
    pub text2:      Color,
    pub text3:      Color,
    pub accent:     Color,
    pub accent_bg:  Color,
    pub success:    Color,
    pub success_bg: Color,
    pub warn:       Color,
    pub warn_bg:    Color,
    pub danger:     Color,
    pub danger_bg:  Color,
}

pub fn palette(is_dark: bool) -> Palette {
    if is_dark {
        Palette {
            bg:         Color::from_rgb8(26,  25,  23),
            surface:    Color::from_rgb8(35,  34,  32),
            surface2:   Color::from_rgb8(42,  40,  38),
            border:     Color::from_rgb8(58,  56,  53),
            text:       Color::from_rgb8(240, 237, 232),
            text2:      Color::from_rgb8(154, 149, 144),
            text3:      Color::from_rgb8(106, 101, 96),
            accent:     Color::from_rgb8(91,  138, 228),
            accent_bg:  Color::from_rgb8(30,  42,  69),
            success:    Color::from_rgb8(58,  175, 107),
            success_bg: Color::from_rgb8(14,  42,  28),
            warn:       Color::from_rgb8(212, 122, 32),
            warn_bg:    Color::from_rgb8(42,  30,  10),
            danger:     Color::from_rgb8(224, 92,  76),
            danger_bg:  Color::from_rgb8(58,  26,  24),
        }
    } else {
        Palette {
            bg:         Color::from_rgb8(244, 243, 241),
            surface:    Color::from_rgb8(255, 255, 255),
            surface2:   Color::from_rgb8(240, 238, 236),
            border:     Color::from_rgb8(216, 212, 207),
            text:       Color::from_rgb8(28,  25,  23),
            text2:      Color::from_rgb8(107, 101, 96),
            text3:      Color::from_rgb8(156, 150, 144),
            accent:     Color::from_rgb8(59,  111, 212),
            accent_bg:  Color::from_rgb8(232, 237, 248),
            success:    Color::from_rgb8(26,  127, 75),
            success_bg: Color::from_rgb8(230, 244, 236),
            warn:       Color::from_rgb8(180, 83,  9),
            warn_bg:    Color::from_rgb8(254, 243, 226),
            danger:     Color::from_rgb8(192, 57,  43),
            danger_bg:  Color::from_rgb8(253, 236, 234),
        }
    }
}
```

- [ ] **Step 2: Register the module in `medianamer-app/src/ui/mod.rs`**

Add `pub mod palette;` as the first line of `mod.rs` (before the existing `mod file_list;` etc.).

- [ ] **Step 3: Build to verify it compiles**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

Expected: no output (no errors).

- [ ] **Step 4: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/palette.rs medianamer-app/src/ui/mod.rs && git commit -m "feat(ui): add design-system color palette module"
```

---

### Task 2: Extend state with sort, message, and token-panel fields

**Files:**
- Modify: `medianamer-app/src/state.rs`

- [ ] **Step 1: Add new enums to `state.rs`**

Insert the following block after the `View` enum definition (around line 84):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SortCol {
    Original,
    Status,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggled(&self) -> Self {
        match self {
            SortDir::Asc  => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Info,
    Success,
    Warn,
}
```

- [ ] **Step 2: Add new fields to `AppState`**

Add to the `AppState` struct (after `is_dark: bool`):

```rust
    pub sort_col:     Option<SortCol>,
    pub sort_dir:     SortDir,
    pub status_msg:   String,
    pub message_kind: MessageKind,
    pub show_tokens:  bool,
```

- [ ] **Step 3: Initialise new fields in `AppState::default()`**

Add to the `Self { ... }` block inside `impl Default for AppState` (after `is_dark: crate::detect_is_dark()`):

```rust
            sort_col:     None,
            sort_dir:     SortDir::Asc,
            status_msg:   "Ready — add files to get started.".to_string(),
            message_kind: MessageKind::Info,
            show_tokens:  false,
```

- [ ] **Step 4: Add new `Message` variants**

Add to the `Message` enum (after `ClearAll`):

```rust
    SortBy(SortCol),
    ToggleTokens,
```

- [ ] **Step 5: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/state.rs && git commit -m "feat(state): add sort, message-bar, and token-panel state"
```

---

### Task 3: Handle new messages and set status messages in `main.rs`

**Files:**
- Modify: `medianamer-app/src/main.rs`

- [ ] **Step 1: Add new imports**

Change the existing state import line (line 4) from:

```rust
use state::{AppState, Message, MatchState, View};
```

to:

```rust
use state::{AppState, Message, MatchState, MessageKind, SortCol, SortDir, View};
```

- [ ] **Step 2: Handle `SortBy` and `ToggleTokens`**

Add the following two match arms in the `update` function, after the `Message::ClearAll` arm:

```rust
        Message::SortBy(col) => {
            if state.sort_col.as_ref() == Some(&col) {
                state.sort_dir = state.sort_dir.toggled();
            } else {
                state.sort_col = Some(col);
                state.sort_dir = SortDir::Asc;
            }
            Task::none()
        }

        Message::ToggleTokens => {
            state.show_tokens = !state.show_tokens;
            Task::none()
        }
```

- [ ] **Step 3: Set status message when files are added**

In the `Message::FilesDropped` arm, after the `Task::batch(tasks)` line is assembled (but before returning it), insert:

```rust
            let added = state.files.len() - start_index;
            state.status_msg   = format!("Added {} file(s). Press Match All to fetch metadata.", added);
            state.message_kind = MessageKind::Info;
```

- [ ] **Step 4: Set status message when matching starts**

In the `Message::MatchAll` arm, before `Task::batch(tasks)`, insert:

```rust
            state.status_msg   = "Searching for matches…".to_string();
            state.message_kind = MessageKind::Info;
```

- [ ] **Step 5: Set status message when all matches complete**

In the `Message::FileMatched` arm, after the `file.match_state = ...` block (before `Task::none()`), insert:

```rust
            let still_loading = state.files.iter().any(|f| matches!(f.match_state, MatchState::Loading));
            if !still_loading && !state.files.is_empty() {
                let matched = state.files.iter()
                    .filter(|f| matches!(f.match_state, MatchState::Matched(_)))
                    .count();
                let unresolved = state.files.len() - matched;
                state.status_msg = format!(
                    "Match complete — {} matched, {} unresolved.", matched, unresolved
                );
                state.message_kind = if matched > 0 { MessageKind::Success } else { MessageKind::Warn };
            }
```

- [ ] **Step 6: Set status message for rename lifecycle**

In the `Message::Rename` arm, before `Task::batch(tasks)`, insert:

```rust
            state.status_msg   = format!("Renaming {} file(s)…", jobs.len());
            state.message_kind = MessageKind::Info;
```

In the `Message::RenameComplete` arm, before `Task::none()`, insert:

```rust
            state.status_msg   = format!("Renamed {} file(s) successfully.", indices.len());
            state.message_kind = MessageKind::Success;
```

In the `Message::ClearAll` arm, before `Task::none()`, insert:

```rust
            state.status_msg   = "File list cleared.".to_string();
            state.message_kind = MessageKind::Info;
```

- [ ] **Step 7: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

Expected: no errors.

- [ ] **Step 8: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/main.rs && git commit -m "feat(app): handle sort/token messages and drive status message bar"
```

---

### Task 4: Redesign header (`ui/toolbar.rs`)

**Files:**
- Modify: `medianamer-app/src/ui/toolbar.rs` (full rewrite)

- [ ] **Step 1: Replace the entire content of `toolbar.rs`**

```rust
use iced::widget::{button, container, pick_list, row, text, Space};
use iced::{Background, Border, Color, Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::{MediaType, MovieSource, TvSource};
use super::palette::Palette;
use super::palette;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let pal = palette::palette(state.is_dark);
    let has_files  = !state.files.is_empty();
    let can_rename = state.any_matched();

    // Logo: accent square + app name
    let ac = pal.accent;
    let logo = row![
        container(text("▶").size(12).color(Color::WHITE))
            .width(28).height(28)
            .center(28)
            .style(move |_| container::Style {
                background: Some(Background::Color(ac)),
                border: Border { radius: 7.0.into(), ..Default::default() },
                ..Default::default()
            }),
        text("MediaNamer").size(14).color(pal.text),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Provider selector
    let source_widget: Element<'_, Message> = match state.media_type {
        MediaType::Movie => {
            let opts: &[MovieSource] = &[MovieSource::Tmdb, MovieSource::Omdb];
            pick_list(opts, Some(&state.config.movie_source), Message::MovieSourceChanged).into()
        }
        MediaType::Tv => {
            let opts: &[TvSource] = &[TvSource::Tmdb, TvSource::Omdb, TvSource::Tvmaze, TvSource::Tvdb];
            pick_list(opts, Some(&state.config.tv_source), Message::TvSourceChanged).into()
        }
    };

    let media_types: &[MediaType] = &[MediaType::Movie, MediaType::Tv];

    let (s, _bg) = (pal.surface, pal.bg);

    let inner = row![
        logo,
        vdivider(&pal),
        ghost_btn("+ Add Files", Message::OpenFilePicker, false,       &pal),
        ghost_btn("⌕ Match All", Message::MatchAll,       !has_files,  &pal),
        ghost_btn("✕ Clear All", Message::ClearAll,       !has_files,  &pal),
        primary_btn("✎ Rename",  Message::Rename,         !can_rename, &pal),
        Space::with_width(Length::Fill),
        text("Type").size(12).color(pal.text2),
        pick_list(media_types, Some(&state.media_type), Message::MediaTypeChanged),
        text("Provider").size(12).color(pal.text2),
        source_widget,
        vdivider(&pal),
        icon_btn("⚙", Message::OpenSettings, &pal),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .padding([0, 16]);

    container(inner)
        .height(52)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(s)),
            ..Default::default()
        })
        .into()
}

fn ghost_btn<'a>(label: &str, msg: Message, disabled: bool, pal: &Palette) -> Element<'a, Message> {
    let (s, b) = (pal.surface, pal.border);
    let tc = if disabled { pal.text3 } else { pal.text };
    button(text(label.to_owned()).size(13).color(tc))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(s)),
            border: Border { color: b, width: 1.0, radius: 6.0.into() },
            text_color: tc,
            ..Default::default()
        })
        .padding([0, 14])
        .height(34)
        .on_press_maybe((!disabled).then_some(msg))
        .into()
}

fn primary_btn<'a>(label: &str, msg: Message, disabled: bool, pal: &Palette) -> Element<'a, Message> {
    let (ac, acb) = (pal.accent, pal.accent_bg);
    let (bg, tc) = if disabled {
        (acb, pal.text3)
    } else {
        (ac, Color::WHITE)
    };
    button(text(label.to_owned()).size(13).color(tc))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 6.0.into(), ..Default::default() },
            text_color: tc,
            ..Default::default()
        })
        .padding([0, 14])
        .height(34)
        .on_press_maybe((!disabled).then_some(msg))
        .into()
}

fn icon_btn<'a>(icon: &str, msg: Message, pal: &Palette) -> Element<'a, Message> {
    let (s, b, t2) = (pal.surface, pal.border, pal.text2);
    button(text(icon.to_owned()).size(16).color(t2))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(s)),
            border: Border { color: b, width: 1.0, radius: 6.0.into() },
            text_color: t2,
            ..Default::default()
        })
        .width(34).height(34)
        .on_press(msg)
        .into()
}

fn vdivider<'a>(pal: &Palette) -> Element<'a, Message> {
    let b = pal.border;
    container(Space::new(1.0, 24.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(b)),
            ..Default::default()
        })
        .into()
}
```

**Note on `.center(28)`:** If `container::center()` doesn't exist in this Iced version, replace:
```rust
container(text("▶").size(12).color(Color::WHITE))
    .width(28).height(28)
    .center(28)
```
with:
```rust
container(text("▶").size(12).color(Color::WHITE))
    .width(28).height(28)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
```

- [ ] **Step 2: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

Fix any errors (most likely `button::Style` field names — if `..Default::default()` fails, the missing field is `shadow: iced::Shadow::default()`).

- [ ] **Step 3: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/toolbar.rs && git commit -m "feat(ui): redesign header with logo, styled buttons, and layout"
```

---

### Task 5: Redesign format bar with togglable token panel (`ui/format_bar.rs`)

**Files:**
- Modify: `medianamer-app/src/ui/format_bar.rs` (full rewrite)

The token panel is a collapsible reference section below the format input. It reads `state.show_tokens` and `state.media_type` to show the right token set. The actual supported tokens are taken from `medianamer-core/src/naming/mod.rs` — the `substitute()` function matches: `title`, `series`, `year`, `season`, `season:02`, `episode`, `episode:02`, `resolution`, `codec`, `ext`.

- [ ] **Step 1: Replace the entire content of `format_bar.rs`**

```rust
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Background, Border, Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::MediaType;
use super::palette;

// (token, example)
const MOVIE_TOKENS: &[(&str, &str)] = &[
    ("{title}",      "The Dark Knight"),
    ("{year}",       "2008"),
    ("{resolution}", "1080p"),
    ("{codec}",      "AV1"),
    ("{ext}",        "mkv"),
];

const TV_TOKENS: &[(&str, &str)] = &[
    ("{title}",       "Pilot"),
    ("{series}",      "Breaking Bad"),
    ("{season:02}",   "01"),
    ("{episode:02}",  "05"),
    ("{resolution}",  "1080p"),
    ("{codec}",       "AV1"),
    ("{ext}",         "mkv"),
];

pub fn view(state: &AppState) -> Element<'_, Message> {
    let pal    = palette::palette(state.is_dark);
    let template = state.current_template().to_string();
    let tokens = if state.media_type == MediaType::Movie { MOVIE_TOKENS } else { TV_TOKENS };

    let (s2, b, t2, ac, acb) = (pal.surface2, pal.border, pal.text2, pal.accent, pal.accent_bg);
    let (tokens_active, s) = (state.show_tokens, pal.surface);

    let tok_btn_bg = if tokens_active { acb } else { s };
    let tok_btn_tc = if tokens_active { ac  } else { t2 };

    let format_row = container(
        row![
            text("Format").size(12).color(t2),
            text_input("e.g. {title} ({year}).{ext}", &template)
                .on_input(Message::TemplateChanged)
                .padding([0, 10])
                .size(13),
            button(
                row![
                    text("Tokens").size(12).color(tok_btn_tc),
                    text(if tokens_active { " ▲" } else { " ▼" }).size(11).color(tok_btn_tc),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center)
            )
            .style(move |_, _| button::Style {
                background: Some(Background::Color(tok_btn_bg)),
                border: Border { color: b, width: 1.0, radius: 6.0.into() },
                text_color: tok_btn_tc,
                ..Default::default()
            })
            .padding([0, 12])
            .height(34)
            .on_press(Message::ToggleTokens),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .padding([0, 16]),
    )
    .height(56)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(s2)),
        ..Default::default()
    });

    if !state.show_tokens {
        return format_row.into();
    }

    // Token reference panel
    let token_rows: Vec<Element<'_, Message>> = tokens
        .iter()
        .map(|(tok, ex)| {
            row![
                container(
                    text(*tok).size(12).color(ac)
                )
                .style(move |_| container::Style {
                    background: Some(Background::Color(acb)),
                    border: Border { radius: 4.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .padding([2, 6]),
                Space::with_width(8),
                text(*ex).size(12).color(t2),
            ]
            .align_y(iced::Alignment::Center)
            .into()
        })
        .collect();

    let panel = container(
        column(token_rows).spacing(6).padding([12, 16]),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(s)),
        ..Default::default()
    });

    column![format_row, panel].into()
}
```

- [ ] **Step 2: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/format_bar.rs && git commit -m "feat(ui): add token reference panel to format bar"
```

---

### Task 6: Redesign file table (`ui/file_list.rs`)

**Files:**
- Modify: `medianamer-app/src/ui/file_list.rs` (full rewrite)

Key changes: 4-column layout (original | new filename | status chip | delete), sortable header buttons for Original and Status columns, coloured pill status chips replacing plain text, empty-state illustration, alternating row backgrounds.

- [ ] **Step 1: Replace the entire content of `file_list.rs`**

```rust
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Background, Border, Color, Element, Length};
use crate::state::{AppState, MatchState, Message, SortCol, SortDir};
use medianamer_core::naming::{format_name, TokenValues};
use super::palette::{self, Palette};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let pal      = palette::palette(state.is_dark);
    let template = state.current_template().to_string();

    // Empty state
    if state.files.is_empty() {
        let t3 = pal.text3;
        return container(
            column![
                text("◎").size(40).color(t3),
                text("No files added. Click + Add Files to begin.").size(14).color(t3),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();
    }

    // Build sorted index
    let mut indexed: Vec<(usize, &crate::state::MediaFile)> =
        state.files.iter().enumerate().collect();

    if let Some(col) = &state.sort_col {
        indexed.sort_by(|(_, a), (_, b)| {
            let ord = match col {
                SortCol::Original => {
                    let an = a.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let bn = b.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    an.cmp(bn)
                }
                SortCol::Status => {
                    status_sort_key(&a.match_state).cmp(&status_sort_key(&b.match_state))
                }
            };
            if state.sort_dir == SortDir::Desc { ord.reverse() } else { ord }
        });
    }

    // Header
    let header = table_header(state, &pal);

    // Rows
    let rows: Vec<Element<'_, Message>> = indexed
        .into_iter()
        .enumerate()
        .map(|(display_idx, (real_idx, file))| {
            file_row(real_idx, file, display_idx, &template, &pal)
        })
        .collect();

    let (s, b) = (pal.surface, pal.border);
    column![
        container(header)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(s)),
                ..Default::default()
            }),
        scrollable(column(rows).width(Length::Fill))
            .height(Length::Fill),
    ]
    .height(Length::Fill)  // required so the scrollable expands to fill the window
    .into()
}

fn table_header<'a>(state: &AppState, pal: &Palette) -> Element<'a, Message> {
    let (t2, b) = (pal.text2, pal.border);

    row![
        sort_col_btn("Original Filename", SortCol::Original, &state.sort_col, &state.sort_dir, pal),
        sort_col_btn("New Filename",       SortCol::Status,  &state.sort_col, &state.sort_dir, pal),
        text("Status").size(11).color(t2)
            .width(Length::Fixed(130.0)),
        Space::with_width(Length::Fixed(36.0)),
    ]
    .spacing(0)
    .padding([0, 16])
    .height(36)
    .align_y(iced::Alignment::Center)
    .into()
}

fn sort_col_btn<'a>(
    label: &str,
    col: SortCol,
    current: &Option<SortCol>,
    dir: &SortDir,
    pal: &Palette,
) -> Element<'a, Message> {
    let indicator = if current.as_ref() == Some(&col) {
        if *dir == SortDir::Asc { " ↑" } else { " ↓" }
    } else {
        " ⇅"
    };
    let t2 = pal.text2;
    let label_str = format!("{}{}", label, indicator);
    button(text(label_str).size(11).color(t2))
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            text_color: t2,
            ..Default::default()
        })
        .width(Length::Fill)
        .on_press(Message::SortBy(col))
        .into()
}

fn file_row<'a>(
    idx: usize,
    file: &'a crate::state::MediaFile,
    display_idx: usize,
    template: &str,
    pal: &Palette,
) -> Element<'a, Message> {
    let even = display_idx % 2 == 0;
    let bg = if even { pal.surface } else { pal.surface2 };

    let original = file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let new_name: String = match &file.match_state {
        MatchState::Matched(m) => {
            if let Some(info) = &file.media_info {
                let values = TokenValues::from_match_and_info(m, info);
                format_name(template, &values).unwrap_or_default()
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };

    let (t, t3) = (pal.text, pal.text3);
    let new_name_el: Element<'_, Message> = if new_name.is_empty() {
        text("—").size(12).color(t3).width(Length::Fill).into()
    } else {
        text(new_name).size(12).color(t).width(Length::Fill).into()
    };

    let chip = status_chip(idx, &file.match_state, pal);
    let del  = delete_btn(idx, pal);

    container(
        row![
            text(original).size(12).color(t).width(Length::Fill),
            new_name_el,
            container(chip).width(Length::Fixed(130.0)),
            container(del).width(Length::Fixed(36.0)),
        ]
        .spacing(0)
        .padding([0, 16])
        .align_y(iced::Alignment::Center),
    )
    .height(40)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    })
    .into()
}

fn status_chip<'a>(idx: usize, state: &MatchState, pal: &Palette) -> Element<'a, Message> {
    match state {
        MatchState::Ambiguous(_) => {
            let (ac, acb) = (pal.accent, pal.accent_bg);
            button(text("◎ Pick…").size(12).color(ac))
                .style(move |_, _| button::Style {
                    background: Some(Background::Color(acb)),
                    border: Border { radius: 20.0.into(), ..Default::default() },
                    text_color: ac,
                    ..Default::default()
                })
                .padding([3, 10])
                .on_press(Message::ResolveAmbiguous(idx))
                .into()
        }
        _ => {
            let (label, bg, color) = chip_style(state, pal);
            container(text(label).size(12).color(color))
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg)),
                    border: Border { radius: 20.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .padding([3, 10])
                .into()
        }
    }
}

fn chip_style(state: &MatchState, pal: &Palette) -> (&'static str, Color, Color) {
    match state {
        MatchState::Matched(_)   => ("✓ Matched",  pal.success_bg, pal.success),
        MatchState::Unmatched    => ("⚠ No Match", pal.warn_bg,    pal.warn),
        MatchState::Error(_)     => ("✗ Error",    pal.danger_bg,  pal.danger),
        MatchState::Loading      => ("Loading…",   pal.surface2,   pal.text2),
        MatchState::Pending      => ("Pending",    pal.surface2,   pal.text3),
        MatchState::Ambiguous(_) => unreachable!(),
    }
}

fn delete_btn<'a>(idx: usize, pal: &Palette) -> Element<'a, Message> {
    let t3 = pal.text3;
    button(text("✕").size(12).color(t3))
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            text_color: t3,
            ..Default::default()
        })
        .width(28).height(28)
        .on_press(Message::RemoveFile(idx))
        .into()
}

fn status_sort_key(state: &MatchState) -> u8 {
    match state {
        MatchState::Matched(_)   => 0,
        MatchState::Ambiguous(_) => 1,
        MatchState::Pending      => 2,
        MatchState::Loading      => 3,
        MatchState::Unmatched    => 4,
        MatchState::Error(_)     => 5,
    }
}
```

**Note on the "New Filename" sort column button:** The second `sort_col_btn` call passes `SortCol::Status` as the column key — this makes the "New Filename" header click sort by status (a reasonable proxy). If you prefer it to be a non-clickable label instead, replace that call with:
```rust
text("New Filename").size(11).color(t2).width(Length::Fill),
```

- [ ] **Step 2: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/file_list.rs && git commit -m "feat(ui): redesign file table with status chips, new-name column, and sort"
```

---

### Task 7: Add summary bar and message bar, update overall layout (`ui/mod.rs`)

**Files:**
- Modify: `medianamer-app/src/ui/mod.rs` (full rewrite)

Removes the `drop_hint` (drag-and-drop note is gone — X11 limitation is documented elsewhere). Adds a summary bar (file/match/error counts) above the message bar. Sets the app background color.

- [ ] **Step 1: Replace the entire content of `ui/mod.rs`**

```rust
use iced::widget::{column, container, row, text, Space};
use iced::{Background, Element, Length};
use crate::state::{AppState, MatchState, Message, MessageKind, View};

pub mod palette;
mod file_list;
mod format_bar;
mod help_panel;
mod match_picker;
mod settings;
mod toolbar;

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.view {
        View::Settings      => settings::view(state),
        View::Help          => help_panel::view(state),
        View::MatchPicker(idx) => match_picker::view(state, *idx),
        View::Main          => main_view(state),
    }
}

fn main_view(state: &AppState) -> Element<'_, Message> {
    let pal = palette::palette(state.is_dark);
    let bg  = pal.bg;

    container(
        column![
            toolbar::view(state),
            format_bar::view(state),
            file_list::view(state),
            summary_bar(state),
            message_bar(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    })
    .into()
}

fn summary_bar(state: &AppState) -> Element<'_, Message> {
    if state.files.is_empty() {
        return Space::with_height(0).into();
    }

    let pal = palette::palette(state.is_dark);
    let matched   = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Matched(_))).count();
    let ambiguous = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Ambiguous(_))).count();
    let unmatched = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Unmatched)).count();
    let errors    = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Error(_))).count();

    let mut items: Vec<Element<'_, Message>> = vec![
        text(format!("{} file(s)", state.files.len()))
            .size(11).color(pal.text2).into(),
    ];
    if matched > 0 {
        items.push(
            text(format!("✓ {} matched", matched)).size(11).color(pal.success).into()
        );
    }
    if ambiguous > 0 {
        items.push(
            text(format!("◎ {} to pick", ambiguous)).size(11).color(pal.accent).into()
        );
    }
    if unmatched > 0 {
        items.push(
            text(format!("⚠ {} unmatched", unmatched)).size(11).color(pal.warn).into()
        );
    }
    if errors > 0 {
        items.push(
            text(format!("✗ {} error(s)", errors)).size(11).color(pal.danger).into()
        );
    }

    let s = pal.surface;
    container(
        row(items).spacing(16).align_y(iced::Alignment::Center),
    )
    .height(28)
    .width(Length::Fill)
    .padding([0, 16])
    .style(move |_| container::Style {
        background: Some(Background::Color(s)),
        ..Default::default()
    })
    .into()
}

fn message_bar(state: &AppState) -> Element<'_, Message> {
    let pal = palette::palette(state.is_dark);
    let (icon, color) = match state.message_kind {
        MessageKind::Success => ("✓ ", pal.success),
        MessageKind::Warn    => ("⚠ ", pal.warn),
        MessageKind::Info    => ("ℹ ", pal.text2),
    };
    let s2 = pal.surface2;
    container(
        text(format!("{}{}", icon, state.status_msg))
            .size(12)
            .color(color),
    )
    .height(32)
    .width(Length::Fill)
    .padding([0, 16])
    .style(move |_| container::Style {
        background: Some(Background::Color(s2)),
        ..Default::default()
    })
    .into()
}
```

- [ ] **Step 2: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Run all tests**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo test 2>&1 | grep -E "^test result"
```

Expected: all ok.

- [ ] **Step 4: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/mod.rs && git commit -m "feat(ui): add summary bar and message bar, set palette background"
```

---

### Task 8: Redesign settings as a centered modal overlay (`ui/settings.rs`)

**Files:**
- Modify: `medianamer-app/src/ui/settings.rs` (full rewrite)

The settings view is still triggered by `View::Settings` (replaces main view), but is now rendered as a centered card on a dark scrim, giving the appearance of a modal dialog. Templates are moved to the format bar; this modal focuses on API keys only (matching the design).

- [ ] **Step 1: Replace the entire content of `settings.rs`**

```rust
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length};
use crate::state::{AppState, Message};
use super::palette;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let pal = palette::palette(state.is_dark);

    let (s, b, t, t2, ac) = (pal.surface, pal.border, pal.text, pal.text2, pal.accent);

    let close_btn = button(text("✕").size(16).color(t2))
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            text_color: t2,
            ..Default::default()
        })
        .on_press(Message::CloseSettings);

    let card = container(
        column![
            // Header
            row![
                text("Settings").size(15).color(t),
                Space::with_width(Length::Fill),
                close_btn,
            ]
            .align_y(iced::Alignment::Center),

            // API Keys
            api_field(
                "TMDB Read Access Token",
                "eyJ…",
                &state.access_token_draft,
                Message::ApiKeyChanged,
                &pal,
            ),
            api_field(
                "OMDB API Key",
                "abc123…",
                &state.omdb_api_key_draft,
                Message::OmdbApiKeyChanged,
                &pal,
            ),
            api_field(
                "TheTVDB API Key",
                "abc123…",
                &state.tvdb_api_key_draft,
                Message::TvdbApiKeyChanged,
                &pal,
            ),

            // Footer buttons
            row![
                Space::with_width(Length::Fill),
                button(text("Cancel").size(13).color(t))
                    .style(move |_, _| button::Style {
                        background: Some(Background::Color(s)),
                        border: Border { color: b, width: 1.0, radius: 6.0.into() },
                        text_color: t,
                        ..Default::default()
                    })
                    .padding([7, 16])
                    .on_press(Message::CloseSettings),
                button(text("Save Settings").size(13).color(Color::WHITE))
                    .style(move |_, _| button::Style {
                        background: Some(Background::Color(ac)),
                        border: Border { radius: 6.0.into(), ..Default::default() },
                        text_color: Color::WHITE,
                        ..Default::default()
                    })
                    .padding([7, 16])
                    .on_press(Message::SaveSettings),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(16)
        .padding(24),
    )
    .width(460)
    .style(move |_| container::Style {
        background: Some(Background::Color(s)),
        border: Border { color: b, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    });

    // Dark scrim + centered card
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color {
                r: 0.0, g: 0.0, b: 0.0, a: 0.4,
            })),
            ..Default::default()
        })
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

fn api_field<'a>(
    label: &str,
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    pal: &super::palette::Palette,
) -> Element<'a, Message> {
    let (b, t2) = (pal.border, pal.text2);
    column![
        text(label).size(12).color(t2),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding([7, 10])
            .size(13),
        // Note: add .password() or .secure(true) here if Iced 0.13 supports it;
        // omit if it doesn't compile.
    ]
    .spacing(4)
    .into()
}
```

- [ ] **Step 2: Build**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo build -p medianamer-app 2>&1 | grep -E "^error"
```

- [ ] **Step 3: Run all tests to confirm nothing regressed**

```bash
cd /home/matt/MediaNamer && ~/.cargo/bin/cargo test 2>&1 | grep -E "^test result"
```

Expected: all ok.

- [ ] **Step 4: Commit**

```bash
cd /home/matt/MediaNamer && git add medianamer-app/src/ui/settings.rs && git commit -m "feat(ui): redesign settings as centered modal overlay"
```
