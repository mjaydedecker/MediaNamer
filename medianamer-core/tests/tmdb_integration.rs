use medianamer_core::sources::{tmdb::TmdbSource, MatchKind, MediaSource};
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn search_movie_returns_match() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/movie"))
        .and(query_param("query", "Fire on the Amazon"))
        .and(query_param("primary_release_year", "1993"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"id": 18898, "title": "Fire on the Amazon", "release_date": "1993-03-14"}
            ]
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.search_movie("Fire on the Amazon", Some(1993)).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_title(), "Fire on the Amazon");
    assert!(matches!(&results[0].kind, MatchKind::Movie { year: Some(1993), .. }));
}

#[tokio::test]
async fn search_tv_fetches_episode_detail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/tv"))
        .and(query_param("query", "BBC Television Shakespeare"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": 5555, "name": "BBC Television Shakespeare"}]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/3/tv/5555/season/4/episode/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "A Midsummer Night's Dream",
            "season_number": 4,
            "episode_number": 3
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source
        .search_tv("BBC Television Shakespeare", Some(4), Some(3), None)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    if let MatchKind::TvEpisode { episode_title, season, episode, .. } = &results[0].kind {
        assert_eq!(episode_title, &Some("A Midsummer Night's Dream".to_string()));
        assert_eq!(season, &Some(4));
        assert_eq!(episode, &Some(3));
    } else {
        panic!("expected TvEpisode");
    }
}

#[tokio::test]
async fn lookup_movie_by_tmdb_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/movie/603"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 603, "title": "The Matrix", "release_date": "1999-03-30"
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.lookup_movie(None, Some(603)).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_title(), "The Matrix");
    assert!(matches!(&results[0].kind, MatchKind::Movie { year: Some(1999), .. }));
}

#[tokio::test]
async fn lookup_movie_by_imdb_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/find/tt0133093"))
        .and(query_param("external_source", "imdb_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "movie_results": [
                {"id": 603, "title": "The Matrix", "release_date": "1999-03-30"}
            ],
            "tv_results": []
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.lookup_movie(Some("tt0133093"), None).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_title(), "The Matrix");
}

#[tokio::test]
async fn search_movie_empty_results() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/3/search/movie"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": []
        })))
        .mount(&server)
        .await;

    let source = TmdbSource::new_with_base_url("dummy_key", &server.uri());
    let results = source.search_movie("xyznotarealfilm", None).await.unwrap();
    assert!(results.is_empty());
}
