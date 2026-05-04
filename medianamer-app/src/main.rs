use iced::{application, event, window, Event, Subscription, Task};
use state::{AppState, Message, MatchState, View};
use medianamer_core::{
    matcher::{parse_filename, score, CONFIDENCE_THRESHOLD},
    mediainfo::MediaInfo,
    renamer::{execute_rename, RenameJob},
    sources::{tmdb::TmdbSource, MediaSource, MediaType},
};

mod state;
mod ui;

fn main() -> iced::Result {
    application("MediaNamer", update, ui::view)
        .subscription(subscription)
        .run_with(|| (AppState::default(), Task::none()))
}

fn subscription(_state: &AppState) -> Subscription<Message> {
    event::listen_with(|event, _status, _id| match event {
        Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::FilesDropped(vec![path]))
        }
        Event::Window(window::Event::FileHovered(_)) => Some(Message::DragHovered(true)),
        Event::Window(window::Event::FilesHoveredLeft) => Some(Message::DragHovered(false)),
        _ => None,
    })
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
            let api_key = state.config.tmdb_api_key.clone();
            let media_type = state.media_type.clone();
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
                let key = api_key.clone();
                let mt = media_type.clone();

                tasks.push(Task::perform(
                    async move {
                        let source = TmdbSource::new(key);
                        match mt {
                            MediaType::Movie => source.search_movie(&query).await,
                            MediaType::Tv => source.search_tv(&query, season, episode).await,
                        }
                        .map_err(|e| e.to_string())
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
            state.api_key_draft = k;
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
            state.config.tmdb_api_key = state.api_key_draft.clone();
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
    }
}
