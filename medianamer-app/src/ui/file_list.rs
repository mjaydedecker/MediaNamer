use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};
use crate::state::{AppState, MatchState, Message};
use medianamer_core::naming::{format_name, TokenValues};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = row![
        text("Original Filename").width(Length::FillPortion(4)),
        text("New Filename").width(Length::FillPortion(4)),
        text("Status").width(Length::FillPortion(1)),
    ]
    .padding([4, 8])
    .spacing(8);

    let rows: Vec<Element<'_, Message>> = state
        .files
        .iter()
        .enumerate()
        .map(|(idx, file)| file_row(state, idx, file))
        .collect();

    scrollable(
        column![header]
            .extend(rows)
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn file_row<'a>(
    state: &'a AppState,
    idx: usize,
    file: &'a crate::state::MediaFile,
) -> Element<'a, Message> {
    let original = file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let (preview, status_text, status_color) = match &file.match_state {
        MatchState::Pending | MatchState::Loading => {
            ("…".to_string(), "Loading", Color::from_rgb8(180, 180, 180))
        }
        MatchState::Matched(m) => {
            let template = state.current_template();
            let preview = if let Some(info) = &file.media_info {
                let values = TokenValues::from_match_and_info(m, info);
                format_name(template, &values).unwrap_or_else(|e| e.to_string())
            } else {
                "?".to_string()
            };
            (preview, "✓", Color::from_rgb8(106, 191, 105))
        }
        MatchState::Ambiguous(_) => {
            ("(click to resolve)".to_string(), "?", Color::from_rgb8(255, 167, 38))
        }
        MatchState::Unmatched => {
            ("(not matched)".to_string(), "✗", Color::from_rgb8(239, 83, 80))
        }
        MatchState::Error(e) => {
            (format!("Error: {}", e), "⚠", Color::from_rgb8(239, 83, 80))
        }
    };

    let row_content = row![
        text(original).width(Length::FillPortion(4)).size(12),
        text(preview).width(Length::FillPortion(4)).size(12),
        text(status_text).color(status_color).width(Length::FillPortion(1)).size(12),
    ]
    .spacing(8)
    .padding([4, 8]);

    if matches!(file.match_state, MatchState::Ambiguous(_)) {
        button(row_content)
            .on_press(Message::ResolveAmbiguous(idx))
            .into()
    } else {
        container(row_content).into()
    }
}
