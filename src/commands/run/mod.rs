use std::process::Command;

use crate::{
    package::Package,
    result::{NanaError, NanaResult, PackageError},
};

pub struct RunScript {}

impl RunScript {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, name: &str) -> NanaResult<()> {
        let package = Package::from_local_package()?;
        let script = package.script(name);

        match script {
            Some(cmd) => {
                run_command(&cmd)?;
            }
            None => {
                return Err(NanaError::Package(PackageError::ScriptNotFound(
                    name.into(),
                )))
            }
        }

        Ok(())
    }
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
