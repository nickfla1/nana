use std::process::Command;

use console::style;
use package_json_schema::PackageJson;

use crate::{
    package::load_package,
    result::{NanaError::Package, NanaResult, PackageError::ScriptNotFound},
};

pub fn exec(name: &str) -> NanaResult<()> {
    let package = load_package()?;
    let script = get_script(name, &package);

    match script {
        Some(cmd) => {
            println!("Script '{}' found, executing:", name);
            println!("{}", style(&cmd).italic());

            run_command(&cmd)?;
        }
        None => return Err(Package(ScriptNotFound(name.into()))),
    }

    Ok(())
}

fn run_command(cmd: &str) -> NanaResult<()> {
    if cmd.starts_with("nana ") {
        Command::new("nana")
            .arg(cmd.strip_prefix("nana ").unwrap_or_default())
            .spawn()?;
    } else if cmd.starts_with("node ") {
        Command::new("node")
            .arg(cmd.strip_prefix("node ").unwrap_or_default())
            .spawn()?;
    } else {
        Command::new("node").arg(cmd).spawn()?;
    }

    Ok(())
}

fn get_script(name: &str, package: &PackageJson) -> Option<String> {
    match &package.scripts {
        Some(scripts) => match scripts.get(name) {
            Some(Some(cmd)) => Some(cmd.clone()),
            _ => None,
        },
        None => None,
    }
}
