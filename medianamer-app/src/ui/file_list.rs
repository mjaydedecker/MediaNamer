use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};
use crate::state::{AppState, MatchState, Message};
use medianamer_core::naming::{format_name, TokenValues};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = row![
        text("Original Filename").width(Length::FillPortion(4)),
        text("New Filename").width(Length::FillPortion(4)),
        text("Status").width(Length::FillPortion(2)),
        text("").width(24), // remove button column
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

    let preview: String;
    let status_widget: Element<'a, Message>;

    match &file.match_state {
        MatchState::Pending | MatchState::Loading => {
            preview = "…".to_string();
            status_widget = text("Loading").color(Color::from_rgb8(180, 180, 180)).size(12).into();
        }
        MatchState::Matched(m) => {
            let template = state.current_template();
            preview = if let Some(info) = &file.media_info {
                let values = TokenValues::from_match_and_info(m, info);
                format_name(template, &values).unwrap_or_else(|e| e.to_string())
            } else {
                "?".to_string()
            };
            status_widget = text("✓").color(Color::from_rgb8(106, 191, 105)).size(12).into();
        }
        MatchState::Ambiguous(_) => {
            preview = "—".to_string();
            status_widget = button(text("Pick…").size(12))
                .on_press(Message::ResolveAmbiguous(idx))
                .into();
        }
        MatchState::Unmatched => {
            preview = "—".to_string();
            status_widget = text("✗ No match").color(Color::from_rgb8(239, 83, 80)).size(12).into();
        }
        MatchState::Error(e) => {
            preview = format!("Error: {e}");
            status_widget = text("⚠").color(Color::from_rgb8(239, 83, 80)).size(12).into();
        }
    };

    container(
        row![
            text(original).width(Length::FillPortion(4)).size(12),
            text(preview).width(Length::FillPortion(4)).size(12),
            container(status_widget).width(Length::FillPortion(2)),
            button(text("✕").size(11))
                .on_press(Message::RemoveFile(idx))
                .width(24),
        ]
        .spacing(8)
        .padding([4, 8])
        .align_y(iced::Alignment::Center),
    )
    .into()
}
