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
        let ac = pal.accent;
        let (msg, color) = if state.drag_hover {
            ("Release to add files", ac)
        } else {
            ("No files added. Click + Add Files to begin.", t3)
        };
        let border_color = if state.drag_hover { ac } else { pal.bg };
        return container(
            column![
                text("◎").size(40).color(color),
                text(msg).size(14).color(color),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_| container::Style {
            border: iced::Border { color: border_color, width: if state.drag_hover { 2.0 } else { 0.0 }, radius: 4.0.into() },
            ..Default::default()
        })
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

    let s = pal.surface;
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
    .height(Length::Fill)
    .into()
}

fn table_header<'a>(state: &AppState, pal: &Palette) -> Element<'a, Message> {
    let t2 = pal.text2;
    row![
        sort_col_btn("Original Filename", SortCol::Original, &state.sort_col, &state.sort_dir, pal),
        text("New Filename").size(11).color(t2).width(Length::Fill),
        container(sort_col_btn("Status", SortCol::Status, &state.sort_col, &state.sort_dir, pal))
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
