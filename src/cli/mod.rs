mod install;
mod run;
mod version;

use clap::Command;
use console::style;

pub fn init() {
    let cmd = Command::new("nana")
        .about("Superfast node package manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true);

    // Version
    let cmd = cmd.subcommand(Command::new("version").about("Output nana version"));

    // Install
    let cmd = cmd.subcommand(Command::new("install").about("Install project dependencies"));

    let matches = cmd.get_matches();

    let result = match matches.subcommand() {
        Some(("version", _)) => version::exec(),
        Some(("install", arg_matches)) => install::exec(arg_matches),
        Some((ext, arg_matches)) => run::exec(ext, arg_matches),
        _ => unreachable!(),
    };

    if let Err(error) = result {
        println!("Nana execution halted with error:");
        println!("{}", style(error).red());
    }
}
