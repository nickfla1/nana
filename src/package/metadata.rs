use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

pub type Dependencies = IndexMap<String, String>;

// source: https://github.com/npm/registry/blob/master/docs/responses/package-metadata.md#dist

#[derive(Deserialize, Debug, Clone)]
pub struct Dist {
    pub tarball: String,
    pub shasum: Option<String>,
    pub integrity: Option<String>,
    #[serde(rename(deserialize = "npm-signature"))]
    pub npm_signature: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
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
