use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::{Client, Request, Response, StatusCode};
use serde::Deserialize;
use tracing::{info, warn};

use crate::data::{GameId, Meta, Metas};

#[derive(Debug)]
pub struct IgdbRequestor {
    client: Client,
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginResponsePayload {
    access_token: String,
}

impl IgdbRequestor {
    #[must_use]
    pub fn new(client: Client, client_id: &str, client_secret: &str) -> Self {
        Self {
            client,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            access_token: None,
        }
    }

    async fn request(&self, request: Request) -> Result<Response> {
        let request_clone = request.try_clone();
        let resp = self.client.execute(request).await?;
        if resp.status().is_success() {
            return Ok(resp);
        }
        if resp.status() != StatusCode::TOO_MANY_REQUESTS {
            resp.error_for_status()?;
            unreachable!();
        }
        let Some(request) = request_clone else {
            return Err(anyhow!("Failed to clone request"));
        };
        warn!("Reached IGDB API rate limit. Sleeping.");
        tokio::time::sleep(Duration::from_secs(60)).await;
        let resp = self.client.execute(request).await?.error_for_status()?;
        Ok(resp)
    }

    async fn login(&mut self) -> Result<()> {
        info!("Logging in to IGDB API");
        let req = self
            .client
            .post("https://id.twitch.tv/oauth2/token")
            .query(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .build()?;
        self.access_token = Some(
            self.request(req)
                .await?
                .json::<LoginResponsePayload>()
                .await?
                .access_token,
        );
        info!("Logged in to IGDB API");
        Ok(())
    }

    pub async fn games(&mut self, ids: &[GameId]) -> Result<Metas> {
        info!("Fetching games from IGDB");
        let access_token = if let Some(access_token) = self.access_token.as_ref() {
            access_token
        } else {
            self.login().await?;
            self.access_token
                .as_ref()
                .ok_or_else(|| anyhow!("Missing access token"))?
        };
        let limit = ids.len();
        let ids = ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let req = self.client.post("https://api.igdb.com/v4/games").bearer_auth(access_token).header("Client-ID", &self.client_id).body(format!("fields age_ratings.category,age_ratings.rating,age_ratings.rating_cover_url,aggregated_rating,aggregated_rating_count,cover.url,first_release_date,franchise.name,game_engines.name,game_engines.logo.url,game_modes.name,genres.name,involved_companies.developer,involved_companies.porting,involved_companies.publisher,involved_companies.supporting,involved_companies.company.country,involved_companies.company.logo.url,involved_companies.company.name,involved_companies.company.start_date,keywords.name,multiplayer_modes.campaigncoop,multiplayer_modes.lancoop,multiplayer_modes.offlinecoop,multiplayer_modes.onlinecoop,name,platforms.category,platforms.name,platforms.generation,platforms.platform_logo.url,player_perspectives.name,release_dates.date,themes.name,rating,rating_count,total_rating,total_rating_count; where id=({ids}); limit {limit};")).build()?;
        let resp = self
            .request(req)
            .await?
            .error_for_status()?
            .json::<Vec<Meta>>()
            .await?;
        Ok(Metas(
            resp.into_iter()
                .map(|meta| (meta.id.clone(), meta))
                .collect(),
        ))
    }
}
