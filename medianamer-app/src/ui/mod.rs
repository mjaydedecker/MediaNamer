use iced::Element;
use crate::state::{AppState, Message, View};

mod file_list;
mod format_bar;
mod help_panel;
mod match_picker;
mod settings;
mod toolbar;

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.view {
        View::Settings => settings::view(state),
        View::Help => help_panel::view(state),
        View::MatchPicker(idx) => match_picker::view(state, *idx),
        View::Main => main_view(state),
    }
}

fn main_view(state: &AppState) -> Element<'_, Message> {
    use iced::widget::{column, text};
    column![
        toolbar::view(state),
        format_bar::view(state),
        file_list::view(state),
        text("Drop files here to add").size(12),
    ]
    .into()
}
