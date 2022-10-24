use clap::ArgMatches;

use crate::{commands::init::Init, result::NanaResult};

pub fn exec(arg_matches: &ArgMatches) -> NanaResult<()> {
    let name = arg_matches.get_one::<String>("name");

    let cmd = Init::new();
    cmd.run(name)
}
