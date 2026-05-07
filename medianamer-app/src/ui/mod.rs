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
        View::Settings         => settings::view(state),
        View::Help             => help_panel::view(state),
        View::MatchPicker(idx) => match_picker::view(state, *idx),
        View::Main             => main_view(state),
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

    let pal       = palette::palette(state.is_dark);
    let matched   = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Matched(_))).count();
    let ambiguous = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Ambiguous(_))).count();
    let unmatched = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Unmatched)).count();
    let errors    = state.files.iter().filter(|f| matches!(f.match_state, MatchState::Error(_))).count();

    let mut items: Vec<Element<'_, Message>> = vec![
        text(format!("{} file(s)", state.files.len())).size(11).color(pal.text2).into(),
    ];
    if matched > 0 {
        items.push(text(format!("✓ {} matched", matched)).size(11).color(pal.success).into());
    }
    if ambiguous > 0 {
        items.push(text(format!("◎ {} to pick", ambiguous)).size(11).color(pal.accent).into());
    }
    if unmatched > 0 {
        items.push(text(format!("⚠ {} unmatched", unmatched)).size(11).color(pal.warn).into());
    }
    if errors > 0 {
        items.push(text(format!("✗ {} error(s)", errors)).size(11).color(pal.danger).into());
    }

    let s = pal.surface;
    container(
        row(items).spacing(16).align_y(iced::Alignment::Center),
    )
    .height(28)
    .width(Length::Fill)
    .padding([0, 16])
    .align_y(iced::alignment::Vertical::Center)
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
        text(format!("{}{}", icon, state.status_msg)).size(12).color(color),
    )
    .height(32)
    .width(Length::Fill)
    .padding([0, 16])
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(s2)),
        ..Default::default()
    })
    .into()
}
