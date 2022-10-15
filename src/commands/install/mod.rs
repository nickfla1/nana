mod downloader;
mod fetcher;
mod metadata;

use console::style;

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

    let metadata_list = load_dependencies_metadata(&package).await?;
    dowload_metadata_list(&metadata_list).await?;

    println!("{}", style("Done.").green());

    Ok(())
}
