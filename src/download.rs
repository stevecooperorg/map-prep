use crate::http::fetch_bytes;
use anyhow::{Context, Result};
use http::Uri;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub struct CachedDownloader {
    directory: PathBuf,
    ext: String,
}

impl CachedDownloader {
    pub fn new(directory: PathBuf, ext: &str) -> Self {
        Self {
            directory,
            ext: ext.to_string(),
        }
    }

    pub async fn download_if_missing(&self, id: &str, uri: &Uri) -> Result<PathBuf> {
        let entry_path = self.entry_path(uri, id);
        if !entry_path.exists() {
            let bytes = fetch_bytes(uri)
                .await
                .context("attempting to donwload static google map")?;
            fs::write(&entry_path, bytes).context("writing downloaded map bytes to disk")?;
        }
        Ok(entry_path)
    }

    fn entry_path(&self, uri: &Uri, id: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        uri.hash(&mut hasher);
        let hash = hasher.finish();
        let entry_file_root = base64::encode(hash.to_string());
        let entry_file_name = format!("{}-{}.{}", id, entry_file_root, self.ext);
        self.directory.join(entry_file_name)
    }
}
