use iced::widget::{button, row, text, text_input};
use iced::Element;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let template = state.current_template().to_string();

    row![
        text("FORMAT").size(11),
        text_input("", &template)
            .on_input(Message::TemplateChanged)
            .padding(4),
        button("?").on_press(Message::OpenHelp),
    ]
    .spacing(8)
    .padding([4, 8])
    .into()
}
