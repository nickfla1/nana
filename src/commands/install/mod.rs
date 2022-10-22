mod download;
mod fetch;
mod lock;
mod modules;
mod state;

use console::style;

use crate::{
    commands::install::fetch::fetch_metadata,
    package::{metadata::MetadataVersion, Package},
    result::NanaResult,
};

use self::{lock::Lock, modules::NodeModules, state::State};

#[derive(Debug)]
pub enum InstallCommand {
    FetchPackage(String, String),
    AddPackage(String, String, MetadataVersion),
    Finish,
}

pub struct Install {
    state: State,
}

impl Install {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}

impl Install {
    pub async fn run(&mut self) -> NanaResult<()> {
        // 1.   Check if `package.json` is present
        // 2.   Load and validate `package.json`
        let package = Package::from_local_package()?;

        // 3.   Check if `nana.lock.yml` is present
        // 3.1  Load `nana.lock.yml`
        let mut lock = match Lock::from_local_lock() {
            Ok(lock) => lock,
            Err(_) => Lock::default(),
        };

        // 3.2  Calculate `package.json` integrity
        // 3.3  Check if `nana.lock.yml` integrity matches `package.json`'s
        if lock.matches(&package)? {
            // 3.4  Load and use `nana.lock.yml` dependencies map
            lock.flat_dependencies();
        } else {
            // 4.   Calculate and load dependencies from `package.json`
            let deps = self.resolve_dependencies(&package).await?;
            lock.set_dependencies(&deps);
        }

        self.state().shared.lock().await.progress.finish_and_clear();
        println!("Resolving dependencies: {}", style("OK").green());

        // 5.   Load `node_modules`
        let node_modules = NodeModules::from_local_dir()?;

        // 5.1  Check if `node_modules` already contains required dependencies
        if !node_modules.matches(&lock) {
            // 6.   Download modules
            self.download(&lock).await?;
        }

        self.state().shared.lock().await.progress.finish_and_clear();
        println!("Downloading dependencies: {}", style("OK").green());

        lock.save_if_dirty()?;

        Ok(())
    }

    async fn resolve_dependencies(
        &mut self,
        package: &Package,
    ) -> NanaResult<Vec<(String, MetadataVersion)>> {
        self.state()
            .shared
            .lock()
            .await
            .progress
            .set_message("Resolving dependencies");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<InstallCommand>();

        let handler_tx = tx.clone();
        let state = self.state();
        let handler = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                // println!("> {:?}", cmd);
                let tx = handler_tx.clone();
                match cmd {
                    InstallCommand::FetchPackage(name, version_range) => {
                        state
                            .shared
                            .lock()
                            .await
                            .dependencies_in_progress
                            .insert(format!("{}@{}", name, version_range));

                        state.shared.lock().await.progress.inc_length(1);

                        fetch_metadata(&name, &version_range, tx).await.unwrap();
                    }
                    InstallCommand::AddPackage(name, version_range, version) => {
                        let key = version.key();
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            state.shared.lock().await.dependencies.entry(key)
                        {
                            e.insert(version.clone());

                            if let Some(deps) = version.dependencies {
                                for (name, version_range) in deps {
                                    tx.send(InstallCommand::FetchPackage(name, version_range))
                                        .unwrap();
                                }
                            }
                        }

                        state
                            .shared
                            .lock()
                            .await
                            .dependencies_in_progress
                            .remove(&format!("{}@{}", name, version_range));

                        state.shared.lock().await.progress.inc(1);

                        if state
                            .shared
                            .lock()
                            .await
                            .dependencies_in_progress
                            .is_empty()
                        {
                            tx.send(InstallCommand::Finish).unwrap();
                        }
                    }
                    InstallCommand::Finish => {
                        if state
                            .shared
                            .lock()
                            .await
                            .dependencies_in_progress
                            .is_empty()
                        {
                            break;
                        }
                    }
                }
            }
        });

        for (name, version_range) in package.dependencies() {
            tx.send(InstallCommand::FetchPackage(name, version_range))?;
        }

        handler.await?;

        Ok(self
            .state()
            .shared
            .lock()
            .await
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(String, MetadataVersion)>>())
    }
}
