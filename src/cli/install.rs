use crate::{commands::install::exec as exec_install, result::NanaResult};
use clap::ArgMatches;

#[tokio::main]
pub async fn exec(_arg_matches: &ArgMatches) -> NanaResult<()> {
    exec_install().await?;

    Ok(())
}
