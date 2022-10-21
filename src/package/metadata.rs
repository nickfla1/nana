use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type Dependencies = IndexMap<String, String>;

// source: https://github.com/npm/registry/blob/master/docs/responses/package-metadata.md#dist

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dist {
    pub tarball: String,
    pub shasum: Option<String>,
    pub integrity: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataVersion {
    pub name: String,
    pub version: String,
    pub dist: Dist,
    pub dependencies: Option<Dependencies>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub modified: String,
    pub versions: HashMap<String, MetadataVersion>,
}

impl MetadataVersion {
    pub fn dependencies(&self) -> Vec<(String, String)> {
        match &self.dependencies {
            Some(dependencies) => dependencies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            None => vec![],
        }
    }

    pub fn key(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}
