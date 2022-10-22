use std::path::PathBuf;

use crate::result::NanaResult;

use super::lock::Lock;

const NODE_MODULES_DIR: &str = "node_modules";

#[derive(Debug)]
pub struct NodeModules {
    modules: Vec<(String, PathBuf)>,
}

impl NodeModules {
    pub fn from_local_dir() -> NanaResult<Self> {
        let modules = match std::fs::read_dir(NODE_MODULES_DIR) {
            Ok(dir) => dir
                .into_iter()
                .filter_map(|p| p.ok())
                .filter(|e| e.metadata().unwrap().is_dir())
                .map(|f| (f.file_name().to_str().unwrap().to_string(), f.path()))
                .collect::<Vec<(String, PathBuf)>>(),
            Err(_) => vec![],
        };

        Ok(NodeModules { modules })
    }

    pub fn matches(&self, lock: &Lock) -> bool {
        let dependencies = lock.flat_dependencies();
        let names = dependencies
            .iter()
            .map(|v| &v.name)
            .collect::<Vec<&String>>();

        if self.modules.len() < names.len() {
            return false;
        }

        self.modules
            .iter()
            .map(|(n, _)| n)
            .all(|i| names.contains(&i))
    }
}
