use anyhow::Result;
use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Config, Credentials, OAuth};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

pub fn token_cache_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("spot-tty")
        .join("token.json")
}

fn build_client(client_id: &str, _client_secret: &str, redirect_uri: &str) -> AuthCodePkceSpotify {
    let creds = Credentials::new_pkce(client_id);
    let oauth = OAuth {
        redirect_uri: redirect_uri.to_string(),
        scopes: scopes!(
            "user-read-private",
            "user-read-email",
            "user-library-read",
            "user-library-modify",
            "playlist-read-private",
            "playlist-read-collaborative",
            "playlist-modify-public",
            "playlist-modify-private",
            "user-follow-read",
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-currently-playing"
        ),
        ..Default::default()
    };
    let config = Config {
        token_cached: true,
        token_refreshing: true,
        cache_path: token_cache_path(),
        ..Default::default()
    };
    AuthCodePkceSpotify::with_config(creds, oauth, config)
}

pub async fn authenticate(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<AuthCodePkceSpotify> {
    if let Some(parent) = token_cache_path().parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut spotify = build_client(client_id, client_secret, redirect_uri);

    let required_scopes = [
        "user-read-private",
        "user-read-email",
        "user-library-read",
        "user-library-modify",
        "playlist-read-private",
        "playlist-read-collaborative",
        "playlist-modify-public",
        "playlist-modify-private",
        "user-follow-read",
        "user-read-playback-state",
        "user-modify-playback-state",
        "user-read-currently-playing",
    ];

    if token_cache_path().exists() {
        match spotify.read_token_cache(true).await {
            Ok(Some(token)) => {
                info!("Loaded token from cache");
                let flat: String = token.scopes.iter().cloned().collect::<Vec<_>>().join(" ");
                let has_all_scopes = required_scopes
                    .iter()
                    .all(|s| flat.split_whitespace().any(|t| t == *s));
                info!("Token scopes: {flat}");
                if !has_all_scopes {
                    warn!("Cached token missing scopes — re-authenticating");
                    let _ = std::fs::remove_file(token_cache_path());
                } else {
                    *spotify.token.lock().await.unwrap() = Some(token);
                    match spotify.current_user().await {
                        Ok(_) => {
                            info!("Cached token valid");
                            return Ok(spotify);
                        }
                        Err(e) => {
                            warn!("Cached token invalid ({e}), re-authenticating");
                        }
                    }
                }
            }
            Ok(None) => info!("No cached token"),
            Err(e) => warn!("Failed to read token cache: {e}"),
        }
    }

    // Fresh OAuth flow
    let url = spotify.get_authorize_url(None)?;
    info!("Opening auth URL: {url}");
    if let Err(e) = open::that(&url) {
        eprintln!("Could not open browser: {e}");
        eprintln!("Open this URL manually:\n  {url}");
    }

    let redirect_url = crate::services::auth_server::wait_for_callback(redirect_uri).await?;
    spotify
        .parse_response_code(&redirect_url)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse OAuth callback"))?;
    spotify.request_token(&redirect_url).await?;

    if let Err(e) = spotify.write_token_cache().await {
        warn!("Could not write token cache: {e}");
    }

    Ok(spotify)
}
