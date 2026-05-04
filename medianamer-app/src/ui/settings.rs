use iced::widget::{button, column, row, text, text_input};
use iced::Element;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    column![
        text("Settings").size(18),
        row![
            text("TMDB API Key").width(160),
            text_input("Paste your TMDB API key here", &state.api_key_draft)
                .on_input(Message::ApiKeyChanged)
                .padding(4),
        ]
        .spacing(8),
        row![
            text("Movie template").width(160),
            text_input(
                "{title} ({year}) ({resolution}) ({codec})",
                &state.movie_template_draft,
            )
            .on_input(Message::MovieTemplateChanged)
            .padding(4),
        ]
        .spacing(8),
        row![
            text("TV template").width(160),
            text_input(
                "{series} - S{season:02}E{episode:02} - {title} ({codec})",
                &state.tv_template_draft,
            )
            .on_input(Message::TvTemplateChanged)
            .padding(4),
        ]
        .spacing(8),
        row![
            button("Save").on_press(Message::SaveSettings),
            button("Cancel").on_press(Message::CloseSettings),
        ]
        .spacing(8),
    ]
    .spacing(16)
    .padding(24)
    .into()
}
