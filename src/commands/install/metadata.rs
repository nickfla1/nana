use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use indicatif::ProgressBar;
use package_json_schema::PackageJson;
use semver_rs::{Range, Version};
use tokio::sync::Mutex;

use crate::{
    package::metadata::{Dependencies, Metadata, MetadataVersion},
    result::NanaResult,
};

use super::fetcher;

type ParsedMetadata = HashMap<String, MetadataVersion>;

pub async fn load_dependencies_metadata(package: &PackageJson) -> NanaResult<ParsedMetadata> {
    let mut metadata = ParsedMetadata::new();

    let pb = ProgressBar::new(0);
    // pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?.progress_chars("#>-"));
    pb.set_message("Downloading metadata");

    let pb = Arc::new(Mutex::new(pb));

    if let Some(dependencies) = &package.dependencies {
        do_load_dependencies_metadata(dependencies, &mut metadata, pb.clone()).await;
    }

    pb.lock_owned().await.finish_and_clear();

    Ok(metadata)
}

#[async_recursion]
async fn do_load_dependencies_metadata(
    dependencies: &Dependencies,
    metadata: &mut ParsedMetadata,
    pb: Arc<Mutex<ProgressBar>>,
) {
    let mut tasks = vec![];

    for (name, version) in dependencies {
        let pb = pb.clone();
        tasks.push(async move { fetch_dependencies_metadata(name, version, pb).await });
    }

    let joined = futures::future::join_all(tasks).await;

    for res in joined {
        match res {
            Ok(mut list) => {
                let meta_version = find_best_matching_version(&mut list);
                let key = meta_version.get_key();
                metadata.insert(key, meta_version.clone());

                if let Some(deps) = &meta_version.dependencies {
                    do_load_dependencies_metadata(deps, metadata, pb.clone()).await;
                }
            }
            Err(error) => {
                panic!("Error: {}", error);
            }
        }
    }
}

async fn fetch_dependencies_metadata(
    name: &String,
    version: &str,
    pb: Arc<Mutex<ProgressBar>>,
) -> NanaResult<Vec<MetadataVersion>> {
    let metadata = fetcher::fetch_metadata(name, Some(pb.clone())).await?;
    let version_list = parse_metadata(metadata, version)?;

    Ok(version_list)
}

fn parse_metadata(metadata: Metadata, raw_range: &str) -> NanaResult<Vec<MetadataVersion>> {
    let range = Range::new(raw_range).parse()?;
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
