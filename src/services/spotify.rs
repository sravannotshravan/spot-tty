//! Spotify data fetching.
//!
//! Uses rspotify only for auth/token management.  All actual API calls go
//! through reqwest directly so we are not constrained by rspotify's model
//! structs, which predate Spotify's February 2026 dev-mode breaking changes.
//!
//! Key Feb-2026 dev-mode changes we must handle:
//!   • playlist.tracks  → playlist.items  (field rename)
//!   • GET /playlists/{id}/items only works for playlists you own/collaborate on
//!   • GET /me/following still works, but followers/popularity stripped
//!   • Batch fetch endpoints removed (GET /tracks, /artists, /albums)
//!   • GET /me/saved/tracks still available via GET /me/tracks

use anyhow::{bail, Context, Result};
use rspotify::{prelude::*, AuthCodePkceSpotify};
use serde::Deserialize;
use tracing::info;

const BASE: &str = "https://api.spotify.com/v1";
const PAGE: u32 = 50;

// ─────────────────────────────────────────────────────────────────────────────
// Public summary types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub display_name: String,
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct PlaylistSummary {
    pub id: String,
    pub name: String,
    pub track_count: u32,
    pub owner: bool, // true = user owns/collaborates → tracks fetchable
}

#[derive(Clone, Debug)]
pub struct TrackSummary {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ArtistSummary {
    pub id: String,
    pub name: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal raw JSON shapes (post Feb-2026 field names)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Page<T> {
    items: Vec<T>,
    next: Option<String>,
    total: Option<u32>,
}

#[derive(Deserialize)]
struct RawPlaylist {
    id: String,
    name: String,
    // Feb-2026: field renamed from "tracks" to "items"
    // We try both so the code works if rspotify ever updates too.
    items: Option<RawPlaylistItemsMeta>,
    tracks: Option<RawPlaylistItemsMeta>, // legacy fallback
    owner: Option<RawOwner>,
}

#[derive(Deserialize)]
struct RawPlaylistItemsMeta {
    total: u32,
}

#[derive(Deserialize)]
struct RawOwner {
    id: String,
}

#[derive(Deserialize)]
struct RawPlaylistItem {
    // Feb-2026: field renamed from "track" to "item"
    item: Option<RawTrack>,
    track: Option<RawTrack>, // legacy fallback
}

#[derive(Deserialize, Clone)]
struct RawTrack {
    id: Option<String>,
    name: String,
    duration_ms: u32,
    artists: Vec<RawArtist>,
    album: Option<RawAlbum>,
}

#[derive(Deserialize, Clone)]
struct RawArtist {
    id: Option<String>,
    name: String,
}

#[derive(Deserialize, Clone)]
struct RawAlbum {
    name: String,
}

#[derive(Deserialize)]
struct RawSavedTrack {
    track: RawTrack,
}

#[derive(Deserialize)]
struct RawArtistObj {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct RawCursors {
    after: Option<String>,
}

#[derive(Deserialize)]
struct RawCursorPage {
    items: Vec<RawArtistObj>,
    cursors: Option<RawCursors>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct RawArtistAlbum {
    name: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: get a valid access token string from rspotify
// ─────────────────────────────────────────────────────────────────────────────

async fn token(spotify: &AuthCodePkceSpotify) -> Result<String> {
    // Trigger a refresh if needed
    spotify.auto_reauth().await.ok();

    let guard = spotify.token.lock().await.unwrap();
    let tok = guard.as_ref().context("No token available")?;
    Ok(tok.access_token.clone())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: authenticated GET returning parsed JSON
// ─────────────────────────────────────────────────────────────────────────────

async fn get<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
    access_token: &str,
) -> Result<T> {
    let resp = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("GET {url} → {status}: {body}");
    }

    resp.json::<T>()
        .await
        .with_context(|| format!("Parsing response from {url}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// User profile
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_user(spotify: &AuthCodePkceSpotify) -> Result<UserProfile> {
    #[derive(Deserialize)]
    struct Me {
        id: String,
        display_name: Option<String>,
    }

    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let me: Me = get(&client, &format!("{BASE}/me"), &tok).await?;

    Ok(UserProfile {
        display_name: me.display_name.unwrap_or_else(|| me.id.clone()),
        id: me.id,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Playlists  — GET /me/playlists
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_playlists(
    spotify: &AuthCodePkceSpotify,
    user_id: &str,
) -> Result<Vec<PlaylistSummary>> {
    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut offset = 0u32;

    loop {
        let url = format!("{BASE}/me/playlists?limit={PAGE}&offset={offset}");
        let page: Page<RawPlaylist> = get(&client, &url, &tok).await?;

        for pl in &page.items {
            // Feb-2026: "items" field; fall back to "tracks" if absent
            let total = pl
                .items
                .as_ref()
                .or(pl.tracks.as_ref())
                .map(|m| m.total)
                .unwrap_or(0);

            let owner = pl.owner.as_ref().map(|o| o.id == user_id).unwrap_or(false);

            results.push(PlaylistSummary {
                id: pl.id.clone(),
                name: pl.name.clone(),
                track_count: total,
                owner,
            });
        }

        offset += page.items.len() as u32;
        if page.next.is_none() || page.items.is_empty() {
            break;
        }
    }

    info!("Fetched {} playlists", results.len());
    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Playlist tracks  — GET /playlists/{id}/items
// Only works for playlists the user owns/collaborates on (Feb-2026 restriction)
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_playlist_tracks(
    spotify: &AuthCodePkceSpotify,
    playlist_id: &str,
) -> Result<Vec<TrackSummary>> {
    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut offset = 0u32;

    loop {
        // Feb-2026: endpoint renamed from /tracks to /items
        let url = format!("{BASE}/playlists/{playlist_id}/items?limit={PAGE}&offset={offset}");

        let page: Page<RawPlaylistItem> = match get(&client, &url, &tok).await {
            Ok(p) => p,
            Err(e) => {
                // 403 = user doesn't own this playlist → return empty gracefully
                if e.to_string().contains("403") {
                    info!("Playlist {playlist_id}: 403 (not owner), skipping tracks");
                    return Ok(vec![]);
                }
                return Err(e);
            }
        };

        for item in &page.items {
            // Feb-2026: field renamed "track" → "item"; try both
            let track = item.item.as_ref().or(item.track.as_ref());
            if let Some(t) = track {
                results.push(raw_track_to_summary(t));
            }
        }

        offset += page.items.len() as u32;
        if page.next.is_none() || page.items.is_empty() {
            break;
        }
    }

    info!(
        "Fetched {} tracks for playlist {}",
        results.len(),
        playlist_id
    );
    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Liked / saved tracks  — GET /me/tracks
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_liked_tracks(spotify: &AuthCodePkceSpotify) -> Result<Vec<TrackSummary>> {
    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut offset = 0u32;

    loop {
        let url = format!("{BASE}/me/tracks?limit={PAGE}&offset={offset}");
        let page: Page<RawSavedTrack> = get(&client, &url, &tok).await?;

        for saved in &page.items {
            results.push(raw_track_to_summary(&saved.track));
        }

        offset += page.items.len() as u32;
        if page.next.is_none() || page.items.is_empty() {
            break;
        }
    }

    info!("Fetched {} liked tracks", results.len());
    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Followed artists  — GET /me/following?type=artist  (cursor-based)
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_followed_artists(spotify: &AuthCodePkceSpotify) -> Result<Vec<ArtistSummary>> {
    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut after: Option<String> = None;

    loop {
        let url = match &after {
            Some(cursor) => format!("{BASE}/me/following?type=artist&limit={PAGE}&after={cursor}"),
            None => format!("{BASE}/me/following?type=artist&limit={PAGE}"),
        };

        // Response shape: { "artists": { "items": [...], "cursors": {...}, "next": ... } }
        #[derive(Deserialize)]
        struct Wrapper {
            artists: RawCursorPage,
        }

        let wrapper: Wrapper = get(&client, &url, &tok).await?;
        let page = wrapper.artists;

        if page.items.is_empty() {
            break;
        }

        for a in &page.items {
            results.push(ArtistSummary {
                id: a.id.clone(),
                name: a.name.clone(),
            });
        }

        match page.cursors.and_then(|c| c.after) {
            Some(cursor) => after = Some(cursor),
            None => break,
        }
    }

    info!("Fetched {} followed artists", results.len());
    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Artist albums  — GET /artists/{id}/albums
// ─────────────────────────────────────────────────────────────────────────────

pub async fn fetch_artist_albums(
    spotify: &AuthCodePkceSpotify,
    artist_id: &str,
) -> Result<Vec<String>> {
    let tok = token(spotify).await?;
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut offset = 0u32;

    loop {
        let url = format!(
            "{BASE}/artists/{artist_id}/albums?limit={PAGE}&offset={offset}&include_groups=album,single"
        );
        let page: Page<RawArtistAlbum> = get(&client, &url, &tok).await?;

        for album in &page.items {
            results.push(album.name.clone());
        }

        offset += page.items.len() as u32;
        if page.next.is_none() || page.items.is_empty() {
            break;
        }
    }

    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────────────────────────

fn raw_track_to_summary(t: &RawTrack) -> TrackSummary {
    TrackSummary {
        id: t.id.clone().unwrap_or_default(),
        name: t.name.clone(),
        artist: t
            .artists
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default(),
        album: t.album.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
        duration_ms: t.duration_ms,
    }
}
