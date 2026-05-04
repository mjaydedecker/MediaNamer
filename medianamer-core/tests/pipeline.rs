use medianamer_core::{
    mediainfo::MediaInfo,
    renamer::{build_new_path, RenameJob},
    sources::{MatchKind, MediaMatch},
};
use std::path::PathBuf;

fn movie_match() -> MediaMatch {
    MediaMatch {
        tmdb_id: 18898,
        kind: MatchKind::Movie {
            title: "Fire on the Amazon".to_string(),
            year: Some(1993),
        },
    }
}

fn fake_info() -> MediaInfo {
    MediaInfo {
        codec: "AV1".to_string(),
        resolution: "1080p".to_string(),
        extension: "mkv".to_string(),
    }
}

#[test]
fn builds_correct_new_path() {
    let job = RenameJob {
        source: PathBuf::from("/media/Fire.on.the.Amazon.1993.mkv"),
        media_match: movie_match(),
        media_info: fake_info(),
        template: "{title} ({year}) ({resolution}) ({codec}).{ext}".to_string(),
    };
    let new_path = build_new_path(&job).unwrap();
    assert_eq!(
        new_path,
        PathBuf::from("/media/Fire on the Amazon (1993) (1080p) (AV1).mkv")
    );
}

#[test]
fn new_path_preserves_directory() {
    let job = RenameJob {
        source: PathBuf::from("/home/user/movies/something.mkv"),
        media_match: movie_match(),
        media_info: fake_info(),
        template: "{title}.{ext}".to_string(),
    };
    let new_path = build_new_path(&job).unwrap();
    assert_eq!(new_path.parent().unwrap(), PathBuf::from("/home/user/movies").as_path());
}
