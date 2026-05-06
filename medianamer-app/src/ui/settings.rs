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
                t2,
            ),
            api_field(
                "OMDB API Key",
                "abc123…",
                &state.omdb_api_key_draft,
                Message::OmdbApiKeyChanged,
                t2,
            ),
            api_field(
                "TheTVDB API Key",
                "abc123…",
                &state.tvdb_api_key_draft,
                Message::TvdbApiKeyChanged,
                t2,
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
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    t2: iced::Color,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(t2),
        text_input(placeholder, value)
            .on_input(on_input)
            .padding([7, 10])
            .size(13),
    ]
    .spacing(4)
    .into()
}
