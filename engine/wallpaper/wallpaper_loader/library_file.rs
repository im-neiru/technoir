use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraryFile {
    entries: Box<[WallpaperMetadata]>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WallpaperMetadata {
    name: Box<str>,
    path: Box<Path>,
}

impl LibraryFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let library_file: Self = toml::de::from_slice(bytes.as_slice())?;
        Ok(library_file)
    }
}
