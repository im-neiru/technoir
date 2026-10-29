use std::{borrow::Cow, io::Write};

use rapidhash::RapidHashMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::plugin::package::EntryPoints;
use crate::{
    errors::PluginInstallError,
    plugin::{consts::API_VERSION, package::PluginType},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct IndexFile<'s> {
    pub(in crate::plugin) api_version: Version,
    pub(in crate::plugin) entries: RapidHashMap<Cow<'s, str>, Entry<'s>>,
    pub(in crate::plugin) pipelines: Vec<PipelineEntry<'s>>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct PipelineEntry<'s> {
    pub(in crate::plugin) shader_idx: u32,
    pub(in crate::plugin) entry_points: EntryPoints<'s>,
}

impl<'s> IndexFile<'s> {
    #[inline]
    pub(in crate::plugin) fn new() -> Self {
        Self {
            entries: RapidHashMap::default(),
            api_version: API_VERSION,
            pipelines: Vec::new(),
        }
    }

    pub(in crate::plugin) fn write<S>(&self, s: S) -> Result<(), PluginInstallError>
    where
        S: Write,
    {
        match cbor2::to_writer(&self, s) {
            Err(cbor2::ser::Error::Io(io)) => return Err(PluginInstallError::Io(io)),
            Err(cbor2::ser::Error::Value(_)) => {
                return Err(PluginInstallError::EncodingError);
            }
            Ok(_) => (),
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::plugin) struct Entry<'s> {
    pub(in crate::plugin) name: Cow<'s, str>,
    pub(in crate::plugin) version: Version,

    #[serde(rename = "type")]
    pub(in crate::plugin) plugin_type: PluginType,

    #[serde(default)]
    pub(in crate::plugin) description: Option<Cow<'s, str>>,

    #[serde(default)]
    pub(in crate::plugin) author: Option<Cow<'s, str>>,

    #[serde(default)]
    pub(in crate::plugin) license: Option<Cow<'s, str>>,

    #[serde(default)]
    pub(in crate::plugin) icon: Option<Cow<'s, str>>,

    #[serde(default)]
    pub(in crate::plugin) tags: Vec<Cow<'s, str>>,

    pub(in crate::plugin) code_id: u32,

    pub(in crate::plugin) pipelines: Vec<u32>,
}
