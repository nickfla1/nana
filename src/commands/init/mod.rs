use git2::Repository;
use package_json_schema::PackageJson;

use crate::result::{NanaError, NanaResult};

const INITIAL_VERSION: &str = "0.1.0";
const INITIAL_MAIN: &str = "main.js";

pub struct Init {}

impl Init {
    pub fn new() -> Self {
        Self {}
    }
}

impl Init {
    pub fn run(&self, name: Option<&String>) -> NanaResult<()> {
        let name = if let Some(name) = name {
            name.clone()
        } else {
            self.directory_name()?
        };

        let repository = if let Ok(repository) = self.repository_url() {
            repository
        } else {
            "".into()
        };

        let package = PackageJson::builder()
            .name(name)
            .version(INITIAL_VERSION)
            .main(INITIAL_MAIN)
            .repository(repository)
            .build();

        let raw_json = serde_json::to_string_pretty(&package)?;
        std::fs::write("package.json", raw_json)?;

        Ok(())
    }

    fn directory_name(&self) -> NanaResult<String> {
        let cwd = std::env::current_dir()?;
        if let Some(name) = cwd.file_name() {
            if let Some(name) = name.to_str() {
                return Ok(name.to_string());
            }
        }

        Err(crate::result::NanaError::IO(
            "Unable to retrieve current directory name".into(),
        ))
    }

    fn repository_url(&self) -> NanaResult<String> {
        let repo = Repository::open("./")?;
        let remote = repo.find_remote("origin")?;
        let url = remote.url();

        if let Some(url) = url {
            Ok(url.into())
        } else {
            Err(NanaError::Runtime("Could not parse remote URL".into()))
        }
    }
}
