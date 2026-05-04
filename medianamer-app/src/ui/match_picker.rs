use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};
use crate::state::{AppState, MatchState, Message};
use medianamer_core::sources::MatchKind;

pub fn view(state: &AppState, file_idx: usize) -> Element<'_, Message> {
    let file = match state.files.get(file_idx) {
        Some(f) => f,
        None => return text("Error: file not found").into(),
    };

    let candidates = match &file.match_state {
        MatchState::Ambiguous(list) => list.clone(),
        _ => return text("No candidates").into(),
    };

    let filename = file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?");

    let candidate_rows: Vec<Element<'_, Message>> = candidates
        .into_iter()
        .map(|m| {
            let label = match &m.kind {
                MatchKind::Movie { title, year } => {
                    let year_str = year.map(|y| y.to_string()).unwrap_or_default();
                    if year_str.is_empty() {
                        title.clone()
                    } else {
                        format!("{} ({})", title, year_str)
                    }
                }
                MatchKind::TvEpisode { series_title, season, episode, episode_title } => {
                    let s = season.unwrap_or(0);
                    let e = episode.unwrap_or(0);
                    let ep_title = episode_title.as_deref().unwrap_or("");
                    format!("{} S{:02}E{:02} — {}", series_title, s, e, ep_title)
                }
            };
            let m_clone = m.clone();
            button(text(label).size(13))
                .on_press(Message::MatchSelected(file_idx, m_clone))
                .width(Length::Fill)
                .into()
        })
        .collect();

    container(
        column![
            text(format!("Select match for: {}", filename)).size(14),
            scrollable(column(candidate_rows).spacing(4)).height(Length::Fill),
            button("Cancel").on_press(Message::CloseSettings),
        ]
        .spacing(12)
        .padding(16),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
