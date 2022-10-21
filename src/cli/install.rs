use crate::{commands::install::Install, result::NanaResult};
use clap::ArgMatches;

#[tokio::main]
pub async fn exec(_arg_matches: &ArgMatches) -> NanaResult<()> {
    let mut cmd = Install::new();
    cmd.run().await
}
