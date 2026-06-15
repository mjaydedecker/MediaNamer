use std::path::PathBuf;
use medianamer_core::{
    config::Config,
    mediainfo::MediaInfo,
    sources::{MediaMatch, MediaType},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub media_type: MediaType,
    pub files: Vec<MediaFile>,
    pub config: Config,
    pub view: View,
    pub access_token_draft: String,
    pub omdb_api_key_draft: String,
    pub tvdb_api_key_draft: String,
    pub movie_template_draft: String,
    pub tv_template_draft: String,
    pub drag_hover: bool,
    pub is_dark: bool,
    pub sort_col:     Option<SortCol>,
    pub sort_dir:     SortDir,
    pub status_msg:   String,
    pub message_kind: MessageKind,
    pub show_tokens:  bool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            media_type: MediaType::Movie,
            files: vec![],
            access_token_draft: config.tmdb_read_access_token.clone(),
            omdb_api_key_draft: config.omdb_api_key.clone(),
            tvdb_api_key_draft: config.tvdb_api_key.clone(),
            movie_template_draft: config.templates.movie.clone(),
            tv_template_draft: config.templates.tv.clone(),
            config,
            view: View::Main,
            drag_hover: false,
            is_dark: crate::detect_is_dark(),
            sort_col:     None,
            sort_dir:     SortDir::Asc,
            status_msg:   "Ready — add files to get started.".to_string(),
            message_kind: MessageKind::Info,
            show_tokens:  false,
        }
    }
}

impl AppState {
    pub fn any_matched(&self) -> bool {
        self.files.iter().any(|f| matches!(f.match_state, MatchState::Matched(_)))
    }

    pub fn current_template(&self) -> &str {
        match self.media_type {
            MediaType::Movie => &self.config.templates.movie,
            MediaType::Tv => &self.config.templates.tv,
        }
    }

    pub fn remove_renamed(&mut self, indices: &[usize]) {
        let to_remove: std::collections::HashSet<usize> = indices.iter().copied().collect();
        let mut i = 0;
        self.files.retain(|_| {
            let keep = !to_remove.contains(&i);
            i += 1;
            keep
        });
    }
}

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub path: PathBuf,
    pub media_info: Option<MediaInfo>,
    pub match_state: MatchState,
}

#[derive(Debug, Clone)]
pub enum MatchState {
    Pending,
    Loading,
    Matched(MediaMatch),
    Ambiguous(Vec<MediaMatch>),
    Unmatched,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Main,
    Settings,
    Help,
    MatchPicker(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortCol {
    Original,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggled(&self) -> Self {
        match self {
            SortDir::Asc  => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageKind {
    Info,
    Success,
    Warn,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFilePicker,
    FilesDropped(Vec<PathBuf>),
    DragHovered(bool),
    MediaInfoLoaded(usize, Result<MediaInfo, String>),
    MediaTypeChanged(MediaType),
    TemplateChanged(String),
    MatchAll,
    FileMatched(usize, Result<Vec<medianamer_core::sources::MediaMatch>, String>),
    ResolveAmbiguous(usize),
    MatchSelected(usize, MediaMatch),
    Rename,
    RenameComplete(Vec<usize>),
    OpenSettings,
    CloseSettings,
    ApiKeyChanged(String),
    OmdbApiKeyChanged(String),
    TvdbApiKeyChanged(String),
    MovieSourceChanged(medianamer_core::sources::MovieSource),
    TvSourceChanged(medianamer_core::sources::TvSource),
    MovieTemplateChanged(String),
    TvTemplateChanged(String),
    SaveSettings,
    OpenHelp,
    CloseHelp,
    RefreshSystemTheme,
    SystemThemeDetected(bool),
    RemoveFile(usize),
    ClearAll,
    SortBy(SortCol),
    ToggleTokens,
}

#[cfg(test)]
mod tests {
    use super::*;
    use medianamer_core::sources::{MatchKind, MediaMatch};

    fn dummy_match() -> MediaMatch {
        MediaMatch {
            tmdb_id: 1,
            kind: MatchKind::Movie {
                title: "Test Movie".to_string(),
                year: Some(2020),
            },
        }
    }

    #[test]
    fn rename_button_disabled_with_no_matched_files() {
        let state = AppState::default();
        assert!(!state.any_matched());
    }

    #[test]
    fn rename_button_enabled_after_match() {
        let mut state = AppState::default();
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/test.mkv"),
            media_info: None,
            match_state: MatchState::Matched(dummy_match()),
        });
        assert!(state.any_matched());
    }

    #[test]
    fn matched_files_removed_after_rename_complete() {
        let mut state = AppState::default();
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/a.mkv"),
            media_info: None,
            match_state: MatchState::Matched(dummy_match()),
        });
        state.files.push(MediaFile {
            path: PathBuf::from("/tmp/b.mkv"),
            media_info: None,
            match_state: MatchState::Unmatched,
        });
        state.remove_renamed(&[0]);
        assert_eq!(state.files.len(), 1);
        assert_eq!(state.files[0].path, PathBuf::from("/tmp/b.mkv"));
    }
}
