mod download;
mod fetch;
mod lock;
mod modules;
mod state;

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
        let mut package = Package::from_local_package()?;

        // 3.   Check if `nana.lock.yml` is present
        // 3.1  Load `nana.lock.yml`
        let mut lock = match Lock::from_local_lock() {
            Ok(lock) => lock,
            Err(_) => Lock::default(),
        };

        // 3.2  Calculate `package.json` integrity
        // 3.3  Check if `nana.lock.yml` integrity matches `package.json`'s
        let dependencies = if lock.matches(&mut package)? {
            // 3.4  Load and use `nana.lock.yml` dependencies map
            lock.flat_dependencies()
        } else {
            // 4.   Calculate and load dependencies from `package.json`
            let deps = self.resolve_dependencies(&package).await?;
            lock.set_dependencies(&deps);

            deps.iter()
                .map(|(_, v)| v.clone())
                .collect::<Vec<MetadataVersion>>()
        };

        dependencies.iter().for_each(|v| {
            println!("> {}@{}", v.name, v.version);
        });

        // 5.   Load `node_modules`
        let node_modules = NodeModules::from_local_dir()?;

        // 5.1  Check if `node_modules` already contains required dependencies
        if !node_modules.matches(&lock) {
            // 6.   Download modules
            node_modules.download(&lock).await?;
        }

        lock.save_if_dirty()?;

        Ok(())
    }

    async fn resolve_dependencies(
        &mut self,
        package: &Package,
    ) -> NanaResult<Vec<(String, MetadataVersion)>> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<InstallCommand>();

        let handler_tx = tx.clone();
        let state = self.state();
        let handler = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                let tx = handler_tx.clone();
                match cmd {
                    InstallCommand::FetchPackage(name, version_range) => {
                        state
                            .shared
                            .lock()
                            .unwrap()
                            .dependencies_in_progress
                            .insert(format!("{}@{}", name, version_range));

                        fetch_metadata(&name, &version_range, tx).await.unwrap();
                    }
                    InstallCommand::AddPackage(name, version_range, version) => {
                        let key = version.key();
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            state.shared.lock().unwrap().dependencies.entry(key)
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
                            .unwrap()
                            .dependencies_in_progress
                            .remove(&format!("{}@{}", name, version_range));

                        if state
                            .shared
                            .lock()
                            .unwrap()
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
                            .unwrap()
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
            .lock()?
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(String, MetadataVersion)>>())
    }
}

// pub async fn exec() -> NanaResult<()> {
//     let package = load_package()?;

//     if let Some(name) = &package.name {
//         println!(
//             "Installing dependencies for package {}.",
//             style(name).italic()
//         );
//     }

//     let dependency_tree = {
//         match Path::new("nana.lock.yml").exists() {
//             true => {
//                 let data = std::fs::read_to_string("nana.lock.yml")?;
//                 let result: HashMap<String, Dependency> = serde_yaml::from_str(&data)?;

//                 std::sync::Arc::new(std::sync::Mutex::new(result))
//             }
//             false => {
//                 let pb = ProgressBar::new(0);
//                 pb.set_message("Downloading metadata");
//                 pb.set_style(
//                     ProgressStyle::default_bar()
//                         .template(
//                             "{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.green/darkgreen}] {pos}/{len}",
//                         )?
//                         .progress_chars("█░"),
//                 );

//                 let result = load_dependencies_metadata(&package, Box::new(pb)).await?;

//                 let lock_data = serde_yaml::to_string(&result)?;
//                 std::fs::write("nana.lock.yml", lock_data)?;

//                 result
//             }
//         }
//     };

//     {
//         let pb = ProgressBar::new(0);
//         pb.set_message("Downloading dependencies");
//         pb.set_style(
//             ProgressStyle::default_bar()
//                 .template(
//                     "{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.green/darkgreen}] {bytes}/{total_bytes} ({bytes_per_sec} - {eta})",
//                 )?
//                 .progress_chars("█░"),
//         );

//         dowload_metadata_list(dependency_tree.clone(), Box::new(pb)).await?;
//     }

//     println!("{}", style("Done.").green());

//     Ok(())
// }
