use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::plugin::package::EntryPoints;

pub(in crate::plugin) struct PipelineIndex<'s> {
    entries: Vec<PipelineEntry>,
    shaders: Vec<ShaderEntry<'s>>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct PipelineEntry {
    pub(in crate::plugin) shader_idx: u32,

    #[serde(default)]
    pub(in crate::plugin) entry_points: Option<EntryPoints>,
}

pub(crate) struct ShaderEntry<'s> {
    blake: Cow<'s, str>,
    pos: u64,
    len: u64,
}
