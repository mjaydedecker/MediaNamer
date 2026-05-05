use iced::{application, event, time, window, Event, Subscription, Task, Theme};
use std::sync::Arc;
use std::time::Duration;
use state::{AppState, Message, MatchState, View};
use medianamer_core::{
    matcher::{fallback_queries, parse_filename, score, CONFIDENCE_THRESHOLD},
    mediainfo::MediaInfo,
    renamer::{execute_rename, RenameJob},
    sources::{
        tmdb::TmdbSource,
        omdb::OmdbSource,
        tvmaze::TvmazeSource,
        tvdb::TvdbSource,
        MediaSource, MediaType,
        MovieSource, TvSource,
    },
};

mod state;
mod ui;

fn main() -> iced::Result {
    application("MediaNamer", update, ui::view)
        .window(window::Settings {
            icon: app_icon(),
            // application_id becomes the Wayland xdg-toplevel app_id and X11
            // WM_CLASS. Must match the .desktop filename and StartupWMClass so
            // GNOME dock can associate the running window with the launcher.
            platform_specific: window::settings::PlatformSpecific {
                application_id: "medianamer".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .theme(theme)
        .subscription(subscription)
        .run_with(|| (AppState::default(), Task::none()))
}

fn app_icon() -> Option<window::Icon> {
    const BYTES: &[u8] = include_bytes!("../assets/icon.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(BYTES));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb  => buf[..info.buffer_size()]
            .chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        _ => return None,
    };
    window::icon::from_rgba(rgba, info.width, info.height).ok()
}

fn theme(state: &AppState) -> Theme {
    if state.is_dark { Theme::Dark } else { Theme::Light }
}

// dark-light 1.1.1 falls back to the gtk-theme *name* for GNOME, which
// doesn't reflect the modern color-scheme GSettings key used by Ubuntu 26.10.
// Read the key directly; fall back to dark-light for non-GNOME desktops.
fn detect_is_dark() -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("prefer-dark"))
        .unwrap_or_else(|| dark_light::detect() == dark_light::Mode::Dark)
}

fn subscription(_state: &AppState) -> Subscription<Message> {
    Subscription::batch([
        event::listen_with(|event, _status, _id| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::FilesDropped(vec![path]))
            }
            Event::Window(window::Event::FileHovered(_)) => Some(Message::DragHovered(true)),
            Event::Window(window::Event::FilesHoveredLeft) => Some(Message::DragHovered(false)),
            _ => None,
        }),
        time::every(Duration::from_secs(3)).map(|_| Message::RefreshSystemTheme),
    ])
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::OpenFilePicker => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter(
                        "Video files",
                        &["mkv", "mp4", "avi", "mov", "m4v", "wmv", "flv", "webm"],
                    )
                    .pick_files()
                    .await
                    .map(|handles| {
                        handles
                            .into_iter()
                            .map(|h| h.path().to_path_buf())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            },
            Message::FilesDropped,
        ),

        Message::DragHovered(hovering) => {
            state.drag_hover = hovering;
            Task::none()
        }

        Message::FilesDropped(paths) => {
            state.drag_hover = false;
            let start_index = state.files.len();
            let mut tasks = vec![];
            for (i, path) in paths.into_iter().enumerate() {
                state.files.push(state::MediaFile {
                    path: path.clone(),
                    media_info: None,
                    match_state: MatchState::Loading,
                });
                let idx = start_index + i;
                tasks.push(Task::perform(
                    async move {
                        MediaInfo::from_file(&path).map_err(|e| e.to_string())
                    },
                    move |result| Message::MediaInfoLoaded(idx, result),
                ));
            }
            Task::batch(tasks)
        }

        Message::MediaInfoLoaded(idx, result) => {
            if let Some(file) = state.files.get_mut(idx) {
                match result {
                    Ok(info) => {
                        file.media_info = Some(info);
                        file.match_state = MatchState::Pending;
                    }
                    Err(e) => file.match_state = MatchState::Error(e),
                }
            }
            Task::none()
        }

        Message::MediaTypeChanged(mt) => {
            state.media_type = mt;
            Task::none()
        }

        Message::TemplateChanged(t) => {
            match state.media_type {
                MediaType::Movie => state.config.templates.movie = t,
                MediaType::Tv => state.config.templates.tv = t,
            }
            Task::none()
        }

        Message::MatchAll => {
            let media_type = state.media_type.clone();
            let source: Arc<dyn MediaSource> = match media_type {
                MediaType::Movie => match state.config.movie_source {
                    MovieSource::Tmdb => Arc::new(TmdbSource::new(state.config.tmdb_read_access_token.clone())),
                    MovieSource::Omdb => Arc::new(OmdbSource::new(state.config.omdb_api_key.clone())),
                },
                MediaType::Tv => match state.config.tv_source {
                    TvSource::Tmdb   => Arc::new(TmdbSource::new(state.config.tmdb_read_access_token.clone())),
                    TvSource::Omdb   => Arc::new(OmdbSource::new(state.config.omdb_api_key.clone())),
                    TvSource::Tvmaze => Arc::new(TvmazeSource::new()),
                    TvSource::Tvdb   => Arc::new(TvdbSource::new(state.config.tvdb_api_key.clone())),
                },
            };
            let mut tasks = vec![];

            for (idx, file) in state.files.iter_mut().enumerate() {
                if !matches!(file.match_state, MatchState::Pending) {
                    continue;
                }
                file.match_state = MatchState::Loading;

                let parsed = parse_filename(
                    file.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(""),
                );
                let query = parsed.title_query.clone();
                let season = parsed.season;
                let episode = parsed.episode;
                let mt = media_type.clone();
                let src = source.clone();

                tasks.push(Task::perform(
                    async move {
                        let queries = fallback_queries(&query);
                        for q in &queries {
                            let result = match &mt {
                                MediaType::Movie => src.search_movie(q).await,
                                MediaType::Tv    => src.search_tv(q, season, episode).await,
                            };
                            match result {
                                Ok(matches) if !matches.is_empty() => return Ok(matches),
                                Ok(_) => {}
                                Err(e) => return Err(e.to_string()),
                            }
                        }
                        Ok(vec![])
                    },
                    move |result| Message::FileMatched(idx, result),
                ));
            }
            Task::batch(tasks)
        }

        Message::FileMatched(idx, result) => {
            if let Some(file) = state.files.get_mut(idx) {
                file.match_state = match result {
                    Err(e) => MatchState::Error(e),
                    Ok(matches) if matches.is_empty() => MatchState::Unmatched,
                    Ok(mut matches) => {
                        let filename = file
                            .path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
                        let query = parse_filename(filename).title_query;
                        let top = matches.remove(0);
                        let confidence = score(&query, top.display_title());
                        if confidence >= CONFIDENCE_THRESHOLD {
                            MatchState::Matched(top)
                        } else {
                            let mut all = vec![top];
                            all.extend(matches);
                            MatchState::Ambiguous(all)
                        }
                    }
                };
            }
            Task::none()
        }

        Message::ResolveAmbiguous(idx) => {
            state.view = View::MatchPicker(idx);
            Task::none()
        }

        Message::MatchSelected(idx, media_match) => {
            if let Some(file) = state.files.get_mut(idx) {
                file.match_state = MatchState::Matched(media_match);
            }
            state.view = View::Main;
            Task::none()
        }

        Message::Rename => {
            let template = state.current_template().to_string();
            let mut jobs: Vec<(usize, RenameJob)> = vec![];

            for (idx, file) in state.files.iter().enumerate() {
                if let (MatchState::Matched(m), Some(info)) =
                    (&file.match_state, &file.media_info)
                {
                    jobs.push((
                        idx,
                        RenameJob {
                            source: file.path.clone(),
                            media_match: m.clone(),
                            media_info: info.clone(),
                            template: template.clone(),
                        },
                    ));
                }
            }

            let renamed_indices: Vec<usize> = jobs.iter().map(|(i, _)| *i).collect();
            for (_, job) in &jobs {
                let _ = execute_rename(job);
            }

            Task::perform(async move { renamed_indices }, Message::RenameComplete)
        }

        Message::RenameComplete(indices) => {
            state.remove_renamed(&indices);
            Task::none()
        }

        Message::OpenSettings => {
            state.view = View::Settings;
            Task::none()
        }
        Message::CloseSettings => {
            state.view = View::Main;
            Task::none()
        }
        Message::ApiKeyChanged(k) => {
            state.access_token_draft = k;
            Task::none()
        }
        Message::OmdbApiKeyChanged(k) => {
            state.omdb_api_key_draft = k;
            Task::none()
        }
        Message::TvdbApiKeyChanged(k) => {
            state.tvdb_api_key_draft = k;
            Task::none()
        }
        Message::MovieSourceChanged(s) => {
            state.config.movie_source = s;
            Task::none()
        }
        Message::TvSourceChanged(s) => {
            state.config.tv_source = s;
            Task::none()
        }
        Message::MovieTemplateChanged(t) => {
            state.movie_template_draft = t;
            Task::none()
        }
        Message::TvTemplateChanged(t) => {
            state.tv_template_draft = t;
            Task::none()
        }
        Message::SaveSettings => {
            state.config.tmdb_read_access_token = state.access_token_draft.clone();
            state.config.omdb_api_key = state.omdb_api_key_draft.clone();
            state.config.tvdb_api_key = state.tvdb_api_key_draft.clone();
            state.config.templates.movie = state.movie_template_draft.clone();
            state.config.templates.tv = state.tv_template_draft.clone();
            let _ = state.config.save();
            state.view = View::Main;
            Task::none()
        }
        Message::OpenHelp => {
            state.view = View::Help;
            Task::none()
        }
        Message::CloseHelp => {
            state.view = View::Main;
            Task::none()
        }
        Message::RefreshSystemTheme => {
            state.is_dark = detect_is_dark();
            Task::none()
        }
        Message::RemoveFile(idx) => {
            if idx < state.files.len() {
                state.files.remove(idx);
            }
            Task::none()
        }
        Message::ClearAll => {
            state.files.clear();
            Task::none()
        }
    }
}
