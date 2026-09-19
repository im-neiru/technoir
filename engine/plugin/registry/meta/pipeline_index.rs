use serde::{Deserialize, Serialize};

use crate::plugin::package::EntryPoints;

pub(in crate::plugin) struct PipelineIndex {
    entries: PipelineEntry,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct PipelineEntry {
    pub(in crate::plugin) shader_id: u32,

    #[serde(default)]
    pub(in crate::plugin) entry_points: Option<EntryPoints>,
}
