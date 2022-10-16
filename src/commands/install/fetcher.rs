use std::sync::Arc;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use tokio::sync::Mutex;

use crate::{package::metadata::Metadata, progress::ProgressHandler, result::NanaResult};

const CACHE_DIR: &str = ".nana/cache/http";
const REGISTRY_URL: &str = "https://registry.npmjs.org/";

const HEADER_ACCEPT: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

pub async fn fetch_metadata(
    name: &String,
    pb: Option<Arc<Mutex<Box<dyn ProgressHandler>>>>,
) -> NanaResult<Metadata> {
    let client = ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: CACHE_DIR.into(),
            },
            options: None,
        }))
        .build();

    if let Some(pb) = &pb {
        pb.lock().await.progress_increment_length(1);
    }

    let result = client
        .get(format!("{}{}", REGISTRY_URL, name))
        .header("accept", HEADER_ACCEPT)
        .send()
        .await?
        .json::<Metadata>()
        .await?;

    if let Some(pb) = &pb {
        pb.lock().await.progress_increment(1);
    }

    Ok(result)
}
