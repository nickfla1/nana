use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use std::io::Cursor;

use bytes::BytesMut;
use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tar::Archive;
use tokio::sync::Mutex;

use crate::{package::metadata::MetadataVersion, result::NanaResult};

pub async fn dowload_metadata_list(list: &HashMap<String, MetadataVersion>) -> NanaResult<()> {
    let mut tasks = vec![];

    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?.progress_chars("#>-"));
    pb.set_message("Downloading dependencies");

    let pb = Arc::new(Mutex::new(pb));

    for meta_version in list.values() {
        let pb = pb.clone();
        tasks.push(async move { download_dist(meta_version, pb).await });
    }

    for res in futures::future::join_all(tasks).await {
        res?
    }

    pb.lock_owned().await.finish_and_clear();

    Ok(())
}

async fn download_dist(
    meta_version: &MetadataVersion,
    pb: Arc<Mutex<ProgressBar>>,
) -> NanaResult<()> {
    let res = reqwest::get(&meta_version.dist.tarball).await?;
    let file_size = res.content_length().unwrap();

    let mut stream = res.bytes_stream();

    pb.lock().await.inc_length(file_size);

    let mut bytes = BytesMut::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        bytes.extend_from_slice(&chunk);
        pb.lock().await.inc(chunk.len() as u64);
    }

    let mut content = Cursor::new(bytes);

    let decoder = GzDecoder::new(&mut content);
    let mut archive = Archive::new(decoder);

    for mut entry in archive.entries()?.filter_map(|e| e.ok()) {
        let package_path: PathBuf = format!("node_modules/{}/", meta_version.name).into();
        let file_path = entry.path()?.strip_prefix("package")?.to_owned();
        let path = package_path.join(file_path);
        std::fs::create_dir_all(path.parent().unwrap())?;
        entry.unpack(&path)?;
    }

    Ok(())
}
