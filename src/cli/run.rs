use clap::ArgMatches;

use crate::{commands::run::RunScript, result::NanaResult};

pub fn exec(name: &str, _arg_matches: &ArgMatches) -> NanaResult<()> {
    let cmd = RunScript::new();
    cmd.run(name)
}
