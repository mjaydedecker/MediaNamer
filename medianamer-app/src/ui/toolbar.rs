use iced::widget::{button, pick_list, row, Space};
use iced::{Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::MediaType;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let media_types = &[MediaType::Movie, MediaType::Tv];

    row![
        button("+ Add Files").on_press(Message::OpenFilePicker),
        button("Match All").on_press(Message::MatchAll),
        button("Rename").on_press_maybe(state.any_matched().then_some(Message::Rename)),
        Space::with_width(Length::Fill),
        pick_list(media_types.as_ref(), Some(&state.media_type), Message::MediaTypeChanged),
        button("⚙").on_press(Message::OpenSettings),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
