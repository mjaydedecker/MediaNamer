use iced::widget::{button, container, pick_list, row, text, Space};
use iced::{Background, Border, Color, Element, Length};
use crate::state::{AppState, Message};
use medianamer_core::sources::{MediaType, MovieSource, TvSource};
use super::palette::{self, Palette};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let pal = palette::palette(state.is_dark);
    let has_files  = !state.files.is_empty();
    let can_rename = state.any_matched();

    // Logo: accent square + app name
    let ac = pal.accent;
    let logo = row![
        container(text("▶").size(12).color(Color::WHITE))
            .width(28).height(28)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_| container::Style {
                background: Some(Background::Color(ac)),
                border: Border { radius: 7.0.into(), ..Default::default() },
                ..Default::default()
            }),
        text("MediaNamer").size(14).color(pal.text),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Provider selector
    let source_widget: Element<'_, Message> = match state.media_type {
        MediaType::Movie => {
            let opts: &[MovieSource] = &[MovieSource::Tmdb, MovieSource::Omdb];
            pick_list(opts, Some(&state.config.movie_source), Message::MovieSourceChanged).into()
        }
        MediaType::Tv => {
            let opts: &[TvSource] = &[TvSource::Tmdb, TvSource::Omdb, TvSource::Tvmaze, TvSource::Tvdb];
            pick_list(opts, Some(&state.config.tv_source), Message::TvSourceChanged).into()
        }
    };

    let media_types: &[MediaType] = &[MediaType::Movie, MediaType::Tv];
    let s = pal.surface;

    let inner = row![
        logo,
        vdivider(&pal),
        ghost_btn("+ Add Files", Message::OpenFilePicker, false,       &pal),
        ghost_btn("⌕ Match All", Message::MatchAll,       !has_files,  &pal),
        ghost_btn("✕ Clear All", Message::ClearAll,       !has_files,  &pal),
        primary_btn("✎ Rename",  Message::Rename,         !can_rename, &pal),
        Space::new().width(Length::Fill),
        text("Type").size(12).color(pal.text2),
        pick_list(media_types, Some(&state.media_type), Message::MediaTypeChanged),
        text("Provider").size(12).color(pal.text2),
        source_widget,
        vdivider(&pal),
        icon_btn("⚙", Message::OpenSettings, &pal),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .padding([10, 16]);

    container(inner)
        .height(60)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(s)),
            ..Default::default()
        })
        .into()
}

fn ghost_btn<'a>(label: &str, msg: Message, disabled: bool, pal: &Palette) -> Element<'a, Message> {
    let (s, b) = (pal.surface, pal.border);
    let tc = if disabled { pal.text3 } else { pal.text };
    button(text(label.to_owned()).size(13).color(tc))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(s)),
            border: Border { color: b, width: 1.0, radius: 6.0.into() },
            text_color: tc,
            ..Default::default()
        })
        .padding([9, 14])
        .on_press_maybe((!disabled).then_some(msg))
        .into()
}

fn primary_btn<'a>(label: &str, msg: Message, disabled: bool, pal: &Palette) -> Element<'a, Message> {
    let (ac, acb) = (pal.accent, pal.accent_bg);
    let (bg, tc) = if disabled { (acb, pal.text3) } else { (ac, Color::WHITE) };
    button(text(label.to_owned()).size(13).color(tc))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 6.0.into(), ..Default::default() },
            text_color: tc,
            ..Default::default()
        })
        .padding([9, 14])
        .on_press_maybe((!disabled).then_some(msg))
        .into()
}

fn icon_btn<'a>(icon: &str, msg: Message, pal: &Palette) -> Element<'a, Message> {
    let (s, b, t2) = (pal.surface, pal.border, pal.text2);
    button(text(icon.to_owned()).size(16).color(t2))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(s)),
            border: Border { color: b, width: 1.0, radius: 6.0.into() },
            text_color: t2,
            ..Default::default()
        })
        .padding(9)
        .on_press(msg)
        .into()
}

fn vdivider<'a>(pal: &Palette) -> Element<'a, Message> {
    let b = pal.border;
    container(Space::new().width(1.0).height(24.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(b)),
            ..Default::default()
        })
        .into()
}
