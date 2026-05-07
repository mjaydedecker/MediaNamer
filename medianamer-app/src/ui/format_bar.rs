use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Background, Border, Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::MediaType;
use super::palette;

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
    let pal     = palette::palette(state.is_dark);
    let template = state.current_template().to_string();
    let tokens  = if state.media_type == MediaType::Movie { MOVIE_TOKENS } else { TV_TOKENS };

    let (s2, b, t2, ac, acb) = (pal.surface2, pal.border, pal.text2, pal.accent, pal.accent_bg);
    let (tokens_active, s) = (state.show_tokens, pal.surface);

    let tok_btn_bg = if tokens_active { acb } else { s };
    let tok_btn_tc = if tokens_active { ac  } else { t2 };

    let format_row = container(
        row![
            text("Format").size(12).color(t2),
            text_input("e.g. {title} ({year}).{ext}", &template)
                .on_input(Message::TemplateChanged)
                .padding([8, 10])
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
            .padding([9, 12])
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
                .padding([6, 10])
                .align_y(iced::alignment::Vertical::Center),
                Space::with_width(10),
                text(*ex).size(12).color(t2),
            ]
            .align_y(iced::Alignment::Center)
            .into()
        })
        .collect();

    let panel = container(
        column(token_rows).spacing(10).padding([14, 16]),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(s)),
        ..Default::default()
    });

    column![format_row, panel].into()
}
