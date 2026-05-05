use iced::widget::{button, column, row, text, text_input};
use iced::Element;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    column![
        text("Settings").size(18),
        row![
            text("TMDB Read Access Token").width(200),
            text_input("Paste your TMDB Read Access Token (eyJ...)", &state.access_token_draft)
                .on_input(Message::ApiKeyChanged)
                .padding(4),
        ]
        .spacing(8),
        row![
            text("OMDB API Key").width(200),
            text_input("Paste your OMDB API key", &state.omdb_api_key_draft)
                .on_input(Message::OmdbApiKeyChanged)
                .padding(4),
        ]
        .spacing(8),
        row![
            text("TheTVDB API Key").width(200),
            text_input("Paste your TheTVDB API key", &state.tvdb_api_key_draft)
                .on_input(Message::TvdbApiKeyChanged)
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
