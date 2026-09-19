use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::plugin::package::EntryPoints;

#[derive(Serialize, Deserialize)]
pub(in crate::plugin) struct PipelineIndex<'s> {
    entries: Vec<PipelineEntry>,
    shaders: Vec<ShaderEntry<'s>>,
}

impl<'s> PipelineIndex<'s> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            shaders: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct PipelineEntry {
    pub(in crate::plugin) shader_idx: u32,

    #[serde(default)]
    pub(in crate::plugin) entry_points: Option<EntryPoints>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ShaderEntry<'s> {
    pub(in crate::plugin) blake: Cow<'s, str>,
    pub(in crate::plugin) pos: u64,
    pub(in crate::plugin) len: u64,
}
