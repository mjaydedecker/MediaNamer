use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};
use crate::state::Message;

const TOKENS: &[(&str, &str, &str)] = &[
    ("{title}",      "TMDB",      "Fire on the Amazon  /  A Midsummer Night's Dream"),
    ("{series}",     "TMDB (TV)", "BBC Television Shakespeare"),
    ("{year}",       "TMDB",      "1993"),
    ("{season}",     "TMDB (TV)", "4"),
    ("{season:02}",  "TMDB (TV)", "04"),
    ("{episode}",    "TMDB (TV)", "3"),
    ("{episode:02}", "TMDB (TV)", "03"),
    ("{resolution}", "MediaInfo", "1080p  /  4K  /  720p"),
    ("{codec}",      "MediaInfo", "AV1  /  H.265  /  H.264"),
    ("{ext}",        "MediaInfo", "mkv  /  mp4"),
];

pub fn view(_state: &crate::state::AppState) -> Element<'_, Message> {
    let header = row![
        text("Token").width(160).size(12),
        text("Source").width(120).size(12),
        text("Example").size(12),
    ]
    .spacing(8);

    let rows: Vec<Element<'_, Message>> = TOKENS
        .iter()
        .map(|(token, source, example)| {
            row![
                text(*token).width(160).size(12),
                text(*source).width(120).size(12),
                text(*example).size(12),
            ]
            .spacing(8)
            .into()
        })
        .collect();

    container(
        column![
            text("Token Reference").size(18),
            header,
            column(rows).spacing(6),
            button("Close").on_press(Message::CloseHelp),
        ]
        .spacing(12)
        .padding(24),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
