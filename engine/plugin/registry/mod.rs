use crate::utils::ReadMappedFile;

mod installer;
mod meta;

use meta::{Codes, IndexFile};

pub(crate) struct PluginRegistry {
    index: ReadMappedFile,
    programs: Codes,
}

impl PluginRegistry {
    pub(super) fn index_file(&self) -> IndexFile<'static> {
        let index: IndexFile = cbor2::from_reader(self.index.stream()).unwrap();

        index
    }
}
