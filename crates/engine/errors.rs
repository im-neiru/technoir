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
pub(crate) enum PluginInstallError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Encoding error")]
    EncodingError,
    #[error("Invalid plugin package")]
    InvalidPluginPackage,

    #[error("Target path already exists but is a file, not a directory")]
    PathIsFile,

    #[error("Could not locate the ProgramData directory")]
    ProgramDataNotFound,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(crate) enum StagingBufferError {
    #[error("Bytes exceed max length")]
    BytesExceedMaxLength,
}

impl From<PluginDirectoryError> for PluginInstallError {
    #[inline]
    fn from(value: PluginDirectoryError) -> Self {
        match value {
            PluginDirectoryError::ProgramDataNotFound => Self::ProgramDataNotFound,
            PluginDirectoryError::Io(error) => Self::Io(error),
            PluginDirectoryError::PathIsFile => Self::PathIsFile,
        }
    }
}
