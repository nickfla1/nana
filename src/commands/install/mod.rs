mod downloader;
mod fetcher;
mod metadata;

use std::{collections::HashMap, path::Path};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{commands::install::metadata::Dependency, package::load_package, result::NanaResult};

use self::{downloader::dowload_metadata_list, metadata::load_dependencies_metadata};

pub async fn exec() -> NanaResult<()> {
    let package = load_package()?;

    if let Some(name) = &package.name {
        println!(
            "Installing dependencies for package {}.",
            style(name).italic()
        );
    }

    let dependency_tree = {
        match Path::new("nana.lock.yml").exists() {
            true => {
                let data = std::fs::read_to_string("nana.lock.yml")?;
                let result: HashMap<String, Dependency> = serde_yaml::from_str(&data)?;

                std::sync::Arc::new(std::sync::Mutex::new(result))
            }
            false => {
                let pb = ProgressBar::new(0);
                pb.set_message("Downloading metadata");
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(
                            "{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.green/darkgreen}] {pos}/{len}",
                        )?
                        .progress_chars("█░"),
                );

                let result = load_dependencies_metadata(&package, Box::new(pb)).await?;

                let lock_data = serde_yaml::to_string(&result)?;
                std::fs::write("nana.lock.yml", lock_data)?;

                result
            }
        }
    };

    {
        let pb = ProgressBar::new(0);
        pb.set_message("Downloading dependencies");
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.green/darkgreen}] {bytes}/{total_bytes} ({bytes_per_sec} - {eta})",
                )?
                .progress_chars("█░"),
        );

        dowload_metadata_list(dependency_tree.clone(), Box::new(pb)).await?;
    }

    println!("{}", style("Done.").green());

    Ok(())
}
