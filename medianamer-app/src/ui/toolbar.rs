use iced::widget::{button, pick_list, row, Space};
use iced::{Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::{MediaType, MovieSource, TvSource};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let media_types = &[MediaType::Movie, MediaType::Tv];

    let has_files = !state.files.is_empty();

    let source_widget: Element<'_, Message> = match state.media_type {
        MediaType::Movie => {
            let opts = &[MovieSource::Tmdb, MovieSource::Omdb];
            pick_list(opts.as_ref(), Some(&state.config.movie_source), Message::MovieSourceChanged).into()
        }
        MediaType::Tv => {
            let opts = &[TvSource::Tmdb, TvSource::Omdb, TvSource::Tvmaze, TvSource::Tvdb];
            pick_list(opts.as_ref(), Some(&state.config.tv_source), Message::TvSourceChanged).into()
        }
    };

    row![
        button("+ Add Files").on_press(Message::OpenFilePicker),
        button("Match All").on_press_maybe(has_files.then_some(Message::MatchAll)),
        button("Clear All").on_press_maybe(has_files.then_some(Message::ClearAll)),
        button("Rename").on_press_maybe(state.any_matched().then_some(Message::Rename)),
        Space::with_width(Length::Fill),
        pick_list(media_types.as_ref(), Some(&state.media_type), Message::MediaTypeChanged),
        source_widget,
        button("⚙").on_press(Message::OpenSettings),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
