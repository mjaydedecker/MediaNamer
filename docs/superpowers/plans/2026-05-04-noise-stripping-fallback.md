# Noise Stripping and Progressive Fallback Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix "✗ No match" failures caused by edition/version tags (e.g. `EXTENDED`, `REPACK`) bleeding into TMDB search queries, and add a progressive fallback that retries with shorter queries when no results are returned.

**Architecture:** Two layers in `medianamer-core/src/matcher/mod.rs`: (1) a compile-time `NOISE_TOKENS` blocklist filters known scene tags from `parse_filename` output at zero API cost; (2) a `fallback_queries` function generates progressively shorter query variants used by `medianamer-app/src/main.rs` in a retry loop that stops at the first non-empty result.

**Tech Stack:** Rust, Iced (app layer), existing `strsim`/`jaro_winkler` for scoring (unchanged), `cargo test` for testing.

---

### Task 1: Add noise token blocklist to `parse_filename`

**Files:**
- Modify: `medianamer-core/src/matcher/mod.rs`

- [ ] **Step 1: Write the failing tests**

Add these tests inside the existing `#[cfg(test)] mod tests` block at the bottom of `medianamer-core/src/matcher/mod.rs`:

```rust
#[test]
fn strips_extended_noise_token() {
    let p = parse_filename("Movie.Title.EXTENDED.2020.1080p.mkv");
    assert_eq!(p.title_query, "movie title");
}

#[test]
fn strips_repack_noise_token() {
    let p = parse_filename("Some.Movie.REPACK.2019.BluRay.mkv");
    assert_eq!(p.title_query, "some movie");
}

#[test]
fn strips_proper_noise_token() {
    let p = parse_filename("Another.Film.PROPER.2021.mkv");
    assert_eq!(p.title_query, "another film");
}

#[test]
fn preserves_non_noise_tokens() {
    // "fire on the amazon" has no noise tokens — must not be altered
    let p = parse_filename("Fire.on.the.Amazon.1993.1080p.AV1.mkv");
    assert_eq!(p.title_query, "fire on the amazon");
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test -p medianamer-core strips_extended_noise_token strips_repack_noise_token strips_proper_noise_token
```

Expected: three FAIL with `assertion failed` (the noise tokens are still in the query).

- [ ] **Step 3: Add `NOISE_TOKENS` constant and filter**

At the top of `medianamer-core/src/matcher/mod.rs`, directly below the `use strsim::jaro_winkler;` line, add:

```rust
const NOISE_TOKENS: &[&str] = &[
    "REPACK", "PROPER", "EXTENDED", "UNRATED", "THEATRICAL", "DC", "LIMITED",
    "RERIP", "READNFO", "INTERNAL", "RETAIL", "DIRECTORS", "CUT", "COMPLETE",
    "DUBBED", "SUBBED", "HYBRID",
];
```

Then in `parse_filename`, replace the `title_tokens` binding:

```rust
    let title_tokens: Vec<&str> = normalized
        .split(['.', '_', '-', ' '])
        .filter(|t| !t.is_empty())
        .take_while(|t| !is_year(t))
        .collect();
```

with:

```rust
    let title_tokens: Vec<&str> = normalized
        .split(['.', '_', '-', ' '])
        .filter(|t| !t.is_empty())
        .take_while(|t| !is_year(t))
        .filter(|t| {
            let upper = t.to_uppercase();
            !NOISE_TOKENS.contains(&upper.as_str())
        })
        .collect();
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p medianamer-core
```

Expected: all tests pass, including the four new ones and all pre-existing ones.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/matcher/mod.rs
git commit -m "feat(matcher): strip known scene noise tokens from title query"
```

---

### Task 2: Add `fallback_queries` function

**Files:**
- Modify: `medianamer-core/src/matcher/mod.rs`

- [ ] **Step 1: Write the failing tests**

Add these tests inside the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn fallback_queries_single_word() {
    assert_eq!(fallback_queries("avatar"), vec!["avatar"]);
}

#[test]
fn fallback_queries_two_words() {
    assert_eq!(
        fallback_queries("movie title"),
        vec!["movie title", "movie"]
    );
}

#[test]
fn fallback_queries_three_words() {
    assert_eq!(
        fallback_queries("the dark knight"),
        vec!["the dark knight", "the dark", "the"]
    );
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test -p medianamer-core fallback_queries
```

Expected: three FAIL with `cannot find function 'fallback_queries' in this scope`.

- [ ] **Step 3: Implement `fallback_queries`**

Add this public function in `medianamer-core/src/matcher/mod.rs`, after the `score` function:

```rust
pub fn fallback_queries(query: &str) -> Vec<String> {
    let words: Vec<&str> = query.split_whitespace().collect();
    (1..=words.len()).rev().map(|n| words[..n].join(" ")).collect()
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p medianamer-core
```

Expected: all tests pass, including the three new ones.

- [ ] **Step 5: Commit**

```bash
git add medianamer-core/src/matcher/mod.rs
git commit -m "feat(matcher): add fallback_queries for progressive search retry"
```

---

### Task 3: Use `fallback_queries` in the `MatchAll` search loop

**Files:**
- Modify: `medianamer-app/src/main.rs`

- [ ] **Step 1: Update the import**

At the top of `medianamer-app/src/main.rs`, line 6, change:

```rust
use medianamer_core::{
    matcher::{parse_filename, score, CONFIDENCE_THRESHOLD},
```

to:

```rust
use medianamer_core::{
    matcher::{fallback_queries, parse_filename, score, CONFIDENCE_THRESHOLD},
```

- [ ] **Step 2: Replace the single search call with a fallback loop**

In `medianamer-app/src/main.rs`, inside `Message::MatchAll`, replace the `Task::perform` async block (lines 196–205):

```rust
                tasks.push(Task::perform(
                    async move {
                        match mt {
                            MediaType::Movie => src.search_movie(&query).await,
                            MediaType::Tv    => src.search_tv(&query, season, episode).await,
                        }
                        .map_err(|e| e.to_string())
                    },
                    move |result| Message::FileMatched(idx, result),
                ));
```

with:

```rust
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
```

- [ ] **Step 3: Verify it builds**

```bash
cargo build -p medianamer-app
```

Expected: compiles with no errors.

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add medianamer-app/src/main.rs
git commit -m "feat(app): retry search with progressively shorter queries on empty results"
```
