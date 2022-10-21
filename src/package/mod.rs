pub mod metadata;

use std::path::Path;

use package_json_schema::PackageJson;
use sha1::{Digest, Sha1};
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

    pub fn calculate_integrity(&mut self) -> String {
        if self.package.dependencies.is_none() {
            return "0".into();
        }

        let mut data = String::new();

        if let Some(dependencies) = &mut self.package.dependencies {
            let mut tmp = dependencies
                .iter()
                .map(|(k, v)| format!("{}@{}", k, v))
                .collect::<Vec<String>>();
            tmp.sort();

            println!("PKG {:?}", tmp);

            let key = tmp.join("");
            data.push_str(&key);
        }

        let mut hasher = Sha1::new();
        hasher.update(data);

        format!("{:x}", hasher.finalize())
    }
}

fn package_in_dir() -> NanaResult<()> {
    match Path::new(PACKAGE_NAME).exists() {
        true => Ok(()),
        false => Err(NanaError::Package(PackageError::NotFound)),
    }
}
