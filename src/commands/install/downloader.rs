use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use std::io::Cursor;

use bytes::BytesMut;
use flate2::read::GzDecoder;
use futures::StreamExt;
use tar::Archive;
use tokio::sync::Mutex;

use crate::{package::metadata::MetadataVersion, progress::ProgressHandler, result::NanaResult};

use super::metadata::Dependency;

pub async fn dowload_metadata_list(
    dependency_tree: Arc<Mutex<HashMap<String, Dependency>>>,
    progress_handler: Box<dyn ProgressHandler>,
) -> NanaResult<()> {
    let mut tasks = vec![];

    let pb = Arc::new(Mutex::new(progress_handler));

    for dependency in dependency_tree.lock().await.values() {
        for version in dependency.versions.values() {
            let pb = pb.clone();
            let version = version.clone();
            tasks.push(async move { download_dist(&version, pb).await });
        }
    }

    for res in futures::future::join_all(tasks).await {
        res?
    }

    pb.lock_owned().await.progress_done();

    Ok(())
}

async fn download_dist(
    meta_version: &MetadataVersion,
    pb: Arc<Mutex<Box<dyn ProgressHandler>>>,
) -> NanaResult<()> {
    let res = reqwest::get(&meta_version.dist.tarball).await?;
    let file_size = res.content_length().unwrap();

    let mut stream = res.bytes_stream();

    pb.lock().await.progress_increment_length(file_size);

    let mut bytes = BytesMut::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        bytes.extend_from_slice(&chunk);
        pb.lock().await.progress_increment(chunk.len() as u64);
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
