use std::{path::Path, slice::Iter};

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

    #[inline]
    pub fn iter(&self) -> LibraryFileIter {
        LibraryFileIter(self.entries.iter())
    }
}

pub struct LibraryFileIter<'a>(Iter<'a, WallpaperMetadata>);

impl<'a> Iterator for LibraryFileIter<'a> {
    type Item = (&'a str, &'a Path);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.0.next()?;

        Some((&entry.name, &entry.path))
    }
}
