use std::path::Path;

use indexmap::IndexMap;
use semver_rs::{Range, Version};
use serde::{Deserialize, Serialize};

use crate::{
    package::{metadata::MetadataVersion, Package},
    result::{LockError, NanaError, NanaResult},
};

const LOCK_NAME: &str = "nana.lock.yml";

type Dependencies = IndexMap<String, MetadataVersion>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Lock {
    dependencies: Option<Dependencies>,
    #[serde(skip)]
    is_dirty: bool,
}

impl Default for Lock {
    fn default() -> Self {
        Self {
            dependencies: None,
            is_dirty: true,
        }
    }
}

impl Lock {
    pub fn from_local_lock() -> NanaResult<Self> {
        match lock_in_dir() {
            true => {
                let raw = std::fs::read_to_string(LOCK_NAME)?;
                let lock: Lock = serde_yaml::from_str(&raw)?;

                Ok(lock)
            }
            false => Err(NanaError::Lock(LockError::NotFound)),
        }
    }

    pub fn save(&mut self) -> NanaResult<()> {
        let data = serde_yaml::to_string(self)?;

        std::fs::write(LOCK_NAME, &data)?;

        self.is_dirty = false;

        Ok(())
    }

    pub fn save_if_dirty(&mut self) -> NanaResult<()> {
        if !self.is_dirty {
            return Ok(());
        }

        self.save()
    }

    pub fn matches(&self, package: &mut Package) -> NanaResult<bool> {
        if (self.dependencies.is_none() && package.has_dependencies())
            || (self.dependencies.is_some() && !package.has_dependencies())
        {
            return Ok(false);
        }

        let deps = &package.dependencies();

        Ok(deps
            .iter()
            .filter_map(|(name, version_range)| {
                if self.contains_matching_version(name, version_range).unwrap() {
                    Some(true)
                } else {
                    None
                }
            })
            .count()
            == deps.len())
    }

    fn contains_matching_version(&self, name: &String, version_range: &str) -> NanaResult<bool> {
        match &self.dependencies {
            Some(dependencies) => {
                let range = Range::new(version_range).parse()?;

                Ok(dependencies
                    .iter()
                    .filter(|(_, meta_version)| meta_version.name.eq(name))
                    .filter_map(|(_, meta_version)| {
                        let version = Version::new(&meta_version.version).parse().unwrap();
                        if range.test(&version) {
                            Some(true)
                        } else {
                            None
                        }
                    })
                    .count()
                    > 0)
            }
            None => Ok(false),
        }
    }

    pub fn flat_dependencies(&self) -> Vec<MetadataVersion> {
        match &self.dependencies {
            Some(dependencies) => dependencies.iter().map(|(_, v)| v.clone()).collect(),
            None => vec![],
        }
    }

    pub fn set_dependencies(&mut self, deps: &[(String, MetadataVersion)]) {
        let mut result = Dependencies::new();

        deps.iter().for_each(|(name, version)| {
            result.insert(name.clone(), version.clone());
        });

        self.dependencies = Some(result);
        self.is_dirty = true;
    }
}

fn lock_in_dir() -> bool {
    Path::new(LOCK_NAME).exists()
}
