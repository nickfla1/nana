mod downloader;
mod fetcher;
mod metadata;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{package::load_package, result::NanaResult};

use self::{downloader::dowload_metadata_list, metadata::load_dependencies_metadata};

pub async fn exec() -> NanaResult<()> {
    let package = load_package()?;

    if let Some(name) = &package.name {
        println!(
            "Installing dependencies for package {}.",
            style(name).italic()
        );
    }

    let metadata_list = {
        let pb = ProgressBar::new(0);
        pb.set_message("Downloading metadata");

        load_dependencies_metadata(&package, Box::new(pb)).await?
    };

    {
        let pb = ProgressBar::new(0);
        pb.set_message("Downloading dependencies");
        pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?.progress_chars("#>-"));

        dowload_metadata_list(&metadata_list, Box::new(pb)).await?;
    }

    println!("{}", style("Done.").green());

    Ok(())
}
