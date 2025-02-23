#![deny(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::future_not_send
)]

mod data;
mod plot;
mod request;

use std::{fs, sync::Arc};

use anyhow::{Error, Result};
use data::{Data, RatingKind};
use reqwest::Client;
use tokio::task::{JoinSet, LocalSet};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

macro_rules! spawn_blocking_tasks {
    ($set:ident, $data:ident, $($f:expr),+) => {
        $({
            let $data = $data.clone();
            $set.spawn_blocking(move || $f);
        })+
    };
}

pub async fn join_local(mut set: JoinSet<Result<()>>, local_set: LocalSet) -> Result<()> {
    tokio::try_join!(
        async {
            local_set.await;
            Ok::<_, Error>(())
        },
        async {
            while let Some(res) = set.join_next().await {
                res??;
            }
            Ok(())
        }
    )?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish(),
    )?;
    dotenvy::dotenv()?;
    let client = Client::new();
    let data = Arc::new(Data::new(client.clone()).await?);

    fs::create_dir_all("out")?;

    let mut plots = JoinSet::new();
    let local_plots = LocalSet::new();

    spawn_blocking_tasks!(
        plots,
        data,
        plot::list_over_time("out/list_over_time_scaled.png", true, &data),
        plot::list_over_time("out/list_over_time.png", false, &data),
        plot::release_dates("out/release_dates.png", &data),
        plot::ranking_difference("out/rating_differences_user.png", RatingKind::User, &data),
        plot::ranking_difference(
            "out/rating_differences_critic.png",
            RatingKind::Critic,
            &data
        )
    );
    plots.spawn_local_on(
        async move { plot::summary("out/summary.png", data).await },
        &local_plots,
    );

    join_local(plots, local_plots).await?;

    Ok(())
}
