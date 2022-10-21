use crate::{
    package::metadata::{Metadata, MetadataVersion},
    result::NanaResult,
};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use semver_rs::{Range, Version};

use super::InstallCommand;

const CACHE_DIR: &str = ".nana/cache/http";
const REGISTRY_URL: &str = "https://registry.npmjs.org/";

const HEADER_ACCEPT: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

pub async fn fetch_metadata(
    name: &String,
    version_range: &String,
    tx: tokio::sync::mpsc::UnboundedSender<InstallCommand>,
) -> NanaResult<()> {
    let version = fetch_dependency(name, version_range).await?;

    tx.send(InstallCommand::AddPackage(
        name.clone(),
        version_range.clone(),
        version,
    ))?;

    Ok(())
}

async fn fetch_dependency(name: &String, version_range: &String) -> NanaResult<MetadataVersion> {
    let metadata = fetch_package_metadata(name).await?;
    let mut meta_versions = parse_metadata(&metadata, version_range)?;
    let best_version = find_best_matching_version(&mut meta_versions);

    Ok(best_version.clone())
}

fn parse_metadata(metadata: &Metadata, version_range: &String) -> NanaResult<Vec<MetadataVersion>> {
    let range = Range::new(version_range).parse()?;
    let mut result: Vec<MetadataVersion> = vec![];

    for (raw_version, meta_version) in metadata.versions.iter() {
        let version = Version::new(raw_version).parse()?;

        if range.test(&version) {
            result.push((*meta_version).clone());
        }
    }

    Ok(result)
}

fn find_best_matching_version(list: &'_ mut Vec<MetadataVersion>) -> &'_ MetadataVersion {
    match list.len() {
        1 => list.first().unwrap(),
        _ => {
            list.sort_by(|a, b| {
                Version::new(&b.version)
                    .parse()
                    .unwrap()
                    .partial_cmp(&Version::new(&a.version).parse().unwrap())
                    .unwrap()
            });

            list.first().unwrap()
        }
    }
}

async fn fetch_package_metadata(name: &String) -> NanaResult<Metadata> {
    let client = ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: CACHE_DIR.into(),
            },
            options: None,
        }))
        .build();

    let result = client
        .get(format!("{}{}", REGISTRY_URL, name))
        .header("accept", HEADER_ACCEPT)
        .send()
        .await?
        .json::<Metadata>()
        .await?;

    Ok(result)
}
