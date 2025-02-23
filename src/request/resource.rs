use std::{fmt, fs, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use reqwest::Client;
use tokio::sync::Semaphore;
use tracing::info;

const MAX_CONNECTIONS: usize = 8;
const RESOURCE_PATH: &str = "res";

#[derive(Debug, Clone)]
pub struct ResourceRequestor {
    client: Client,
    sem: Arc<Semaphore>,
}

#[derive(Debug)]
pub enum ImageSize {
    Hd,
}

impl fmt::Display for ImageSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "t_{}",
            match self {
                Self::Hd => "720p",
            }
        )
    }
}

impl ResourceRequestor {
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            sem: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        }
    }

    pub async fn get(&self, size: ImageSize, url: &str) -> Result<Vec<u8>> {
        let mut url_parts = url.split('/').collect::<Vec<_>>();
        let is_igdb = url_parts[url_parts.len() - 2] == "t_thumb";
        let size = size.to_string();

        let filename = if is_igdb {
            let ext = "png";
            let mut filename = url_parts[url_parts.len() - 1]
                .split('.')
                .collect::<Vec<_>>();
            if let Some(filename_ext) = filename.last_mut() {
                *filename_ext = ext;
            }
            filename.join(".")
        } else {
            url_parts[url_parts.len() - 1].to_string()
        };
        let idx = url_parts.len() - 1;
        url_parts[idx] = filename.as_str();

        if is_igdb {
            let idx = url_parts.len() - 2;
            url_parts[idx] = size.as_str();
        }

        let mut path = PathBuf::new();
        path.push(RESOURCE_PATH);
        if is_igdb {
            path.push(&size);
        }
        path.push(&filename);

        info!("Obtaining file {}", path.to_string_lossy());

        if path.exists() {
            return Ok(fs::read(path)?);
        }

        let req_url = format!("https:{}", url_parts.join("/"));
        let request = self.client.get(&req_url);

        let res = {
            let _permit = self.sem.acquire().await?;
            info!("Downloading file at {req_url}");
            request.send().await?
        }
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();
        info!("Downloaded file at {req_url}");

        fs::create_dir_all(path.parent().ok_or_else(|| anyhow!("Error"))?)?;
        fs::write(path, &res)?;

        Ok(res)
    }
}
