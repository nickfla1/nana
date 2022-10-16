use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_recursion::async_recursion;
use indexmap::IndexMap;
use package_json_schema::PackageJson;
use semver_rs::{Range, Version};
use serde::{Deserialize, Serialize};

use crate::{
    package::metadata::{Dependencies, Metadata, MetadataVersion},
    progress::ProgressHandler,
    result::NanaResult,
};

use super::fetcher;

type DependencyTree = HashMap<String, Dependency>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependency {
    pub versions: HashMap<String, MetadataVersion>,
    pub parents: Vec<String>,
}

pub async fn load_dependencies_metadata<'a>(
    package: &PackageJson,
    progress_handler: Box<dyn ProgressHandler>,
) -> NanaResult<Arc<Mutex<DependencyTree>>> {
    let dependency_tree = Arc::new(Mutex::new(HashMap::<String, Dependency>::new()));
    let pb = Arc::new(Mutex::new(progress_handler));

    if let Some(dependencies) = &package.dependencies {
        do_load_dependencies_metadata(
            "__root__",
            dependencies,
            dependency_tree.clone(),
            pb.clone(),
        )
        .await;
    }

    pb.lock()?.progress_done();

    Ok(dependency_tree.clone())
}

#[async_recursion]
async fn do_load_dependencies_metadata(
    parent: &str,
    dependencies: &Dependencies,
    dependency_tree: Arc<Mutex<DependencyTree>>,
    pb: Arc<Mutex<Box<dyn ProgressHandler>>>,
) {
    let mut tasks = vec![];

    for (name, version) in dependencies {
        let pb = pb.clone();
        let dependency_tree = dependency_tree.clone();
        tasks.push(async move {
            fetch_dependencies_metadata(parent, name, version, dependency_tree, pb).await
        });
    }

    let joined = futures::future::join_all(tasks).await;
    for res in joined {
        match res {
            Ok((Some(name), Some(dependencies))) => {
                do_load_dependencies_metadata(
                    &name,
                    &dependencies,
                    dependency_tree.clone(),
                    pb.clone(),
                )
                .await;
            }
            Ok((_, _)) => {}
            Err(error) => {
                panic!("Error: {}", error);
            }
        }
    }
}

async fn fetch_dependencies_metadata(
    parent: &str,
    name: &String,
    version: &str,
    dependency_tree: Arc<Mutex<DependencyTree>>,
    pb: Arc<Mutex<Box<dyn ProgressHandler>>>,
) -> NanaResult<(Option<String>, Option<IndexMap<String, String>>)> {
    if let Some(dep) = dependency_tree.lock()?.get_mut(name) {
        let range = Range::new(version).parse()?;

        for version in dep.versions.keys() {
            if range.test(&Version::new(version).parse()?) {
                dep.parents.push(parent.into());

                return Ok((None, None));
            }
        }
    }

    let metadata = fetcher::fetch_metadata(name, Some(pb.clone())).await?;
    let mut version_list = parse_metadata(metadata, version)?;
    let meta_version = find_best_matching_version(&mut version_list);

    let mut tree_lock = dependency_tree.lock()?;
    if let Some(dep) = tree_lock.get_mut(name) {
        dep.versions
            .insert(meta_version.version.clone(), meta_version.clone());
    } else {
        tree_lock.insert(
            name.clone(),
            Dependency {
                versions: HashMap::from([(meta_version.version.clone(), meta_version.clone())]),
                parents: vec![parent.into()],
            },
        );
    }

    Ok((Some(name.clone()), meta_version.dependencies.clone()))
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
