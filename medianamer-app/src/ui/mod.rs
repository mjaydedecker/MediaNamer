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
    use iced::widget::{column, container, text};
    use iced::{Color, Length};

    let drop_hint = if state.drag_hover {
        container(text("Release to add files").size(13))
            .style(|_theme| iced::widget::container::Style {
                border: iced::Border {
                    color: Color::from_rgb8(100, 180, 255),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .padding(6)
            .width(Length::Fill)
    } else {
        container(text("Use + Add Files to browse, or drag and drop (X11/XWayland only)").size(12))
            .padding(6)
            .width(Length::Fill)
    };

    column![
        toolbar::view(state),
        format_bar::view(state),
        file_list::view(state),
        drop_hint,
    ]
    .into()
}
