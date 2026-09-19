use std::io;

#[derive(Debug, thiserror::Error)]
pub(crate) enum PluginDirectoryError {
    #[error("Could not locate the ProgramData directory")]
    ProgramDataNotFound,

    #[error("Failed to create the plugin directory: {0}")]
    Io(#[from] io::Error),

    #[error("Target path already exists but is a file, not a directory")]
    PathIsFile,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PluginRegistryEncodeError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Encoding error")]
    EncodingError,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(crate) enum StagingBufferError {
    #[error("Bytes exceed max length")]
    BytesExceedMaxLength,
}
