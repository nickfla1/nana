use clap::ArgMatches;

use crate::result::NanaResult;

pub fn exec(name: &str, _arg_matches: &ArgMatches) -> NanaResult<()> {
    crate::commands::run::exec(name)
}
