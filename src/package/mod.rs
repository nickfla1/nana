pub mod metadata;

use std::path::Path;

use package_json_schema::PackageJson;
use validator::Validate;

use crate::result::{NanaError, NanaResult, PackageError};
const PACKAGE_NAME: &str = "package.json";

pub fn load_package() -> NanaResult<PackageJson> {
    match package_in_dir() {
        Ok(_) => {
            let raw = std::fs::read_to_string(PACKAGE_NAME)?;
            let package = PackageJson::try_from(raw)?;

            package.validate()?;

            Ok(package)
        }
        Err(error) => Err(error),
    }
}

fn package_in_dir() -> NanaResult<()> {
    match Path::new(PACKAGE_NAME).exists() {
        true => Ok(()),
        false => Err(NanaError::Package(PackageError::NotFound)),
    }
}
