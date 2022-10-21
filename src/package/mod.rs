pub mod metadata;

use std::path::Path;

use package_json_schema::PackageJson;
use validator::Validate;

use crate::result::{NanaError, NanaResult, PackageError};

const PACKAGE_NAME: &str = "package.json";

pub struct Package {
    package: PackageJson,
}

impl Package {
    pub fn from_local_package() -> NanaResult<Self> {
        match package_in_dir() {
            Ok(_) => {
                let raw = std::fs::read_to_string(PACKAGE_NAME)?;
                let package = PackageJson::try_from(raw)?;

                package.validate()?;

                Ok(Package { package })
            }
            Err(error) => Err(error),
        }
    }

    pub fn dependencies(&self) -> Vec<(String, String)> {
        match &self.package.dependencies {
            Some(dependencies) => dependencies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            None => vec![],
        }
    }

    pub fn script(&self, name: &str) -> Option<String> {
        match &self.package.scripts {
            Some(scripts) => match scripts.get(name) {
                Some(Some(cmd)) => Some(cmd.clone()),
                _ => None,
            },
            None => None,
        }
    }

    pub fn has_dependencies(&self) -> bool {
        match &self.package.dependencies {
            Some(dependencies) => !dependencies.is_empty(),
            None => false,
        }
    }
}

fn package_in_dir() -> NanaResult<()> {
    match Path::new(PACKAGE_NAME).exists() {
        true => Ok(()),
        false => Err(NanaError::Package(PackageError::NotFound)),
    }
}
