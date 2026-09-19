use std::{borrow::Cow, fs, path::Path};

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{errors::PluginRegistryEncodeError, plugin::package::PluginType};

#[derive(Serialize, Deserialize)]
pub(crate) struct IndexFile<'s> {
    api_version: Version,
    entries: Vec<Entry<'s>>,
}

impl<'s> IndexFile<'s> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            api_version: Version::new(0, 1, 0),
        }
    }

    pub(crate) fn write(&self, path: impl AsRef<Path>) -> Result<(), PluginRegistryEncodeError> {
        let file = fs::File::open(path)?;

        match cbor2::to_writer(&self, file) {
            Err(cbor2::ser::Error::Io(io)) => return Err(PluginRegistryEncodeError::Io(io)),
            Err(cbor2::ser::Error::Value(_)) => {
                return Err(PluginRegistryEncodeError::EncodingError);
            }
            Ok(_) => (),
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Entry<'s> {
    name: Cow<'s, str>,
    id: Cow<'s, str>,
    version: Version,

    #[serde(rename = "type")]
    plugin_type: PluginType,

    #[serde(default)]
    description: Option<String>,

    #[serde(default)]
    author: Option<String>,

    #[serde(default)]
    license: Option<String>,

    #[serde(default)]
    icon: Option<String>,

    #[serde(default)]
    tags: Vec<String>,

    code_id: u32,
}
