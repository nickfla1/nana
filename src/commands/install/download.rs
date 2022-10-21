use std::path::PathBuf;

use std::io::Cursor;

use bytes::BytesMut;
use flate2::read::GzDecoder;
use futures::StreamExt;
use tar::Archive;

use crate::{package::metadata::MetadataVersion, result::NanaResult};

use super::{lock::Lock, modules::NodeModules};

impl NodeModules {
    pub async fn download(&self, lock: &Lock) -> NanaResult<()> {
        let mut tasks = vec![];

        let dependencies = lock.flat_dependencies();
        for meta_version in dependencies {
            tasks.push(async move { download_dist(&meta_version).await });
        }

        futures::future::join_all(tasks).await;

        Ok(())
    }
}

async fn download_dist(meta_version: &MetadataVersion) -> NanaResult<()> {
    println!(">> {}", meta_version.name);

    let res = reqwest::get(&meta_version.dist.tarball).await?;

    let mut stream = res.bytes_stream();

    let mut bytes = BytesMut::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        bytes.extend_from_slice(&chunk);
    }

    let mut content = Cursor::new(bytes);

    let decoder = GzDecoder::new(&mut content);
    let mut archive = Archive::new(decoder);

    for mut entry in archive.entries()?.filter_map(|e| e.ok()) {
        let package_path: PathBuf = format!("node_modules/{}/", meta_version.name).into();
        let entry_path = entry.path()?;
        let file_path: PathBuf = if entry_path.starts_with("package") {
            entry_path.strip_prefix("package")?.to_owned()
        } else {
            entry_path.into()
        };

        let path = package_path.join(file_path);
        std::fs::create_dir_all(path.parent().unwrap())?;
        entry.unpack(&path)?;
    }

    Ok(())
}
