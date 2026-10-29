use std::{env, fs, path::PathBuf};

use crate::errors::PluginDirectoryError;

#[inline]
pub(crate) fn get_plugin_directory() -> Result<PathBuf, PluginDirectoryError> {
    let Some(custom_dir) = env::var_os("TECHNOIR_PLUGINS") else {
        return get_default_plugin_directory();
    };

    let custom_dir = PathBuf::from(custom_dir);

    if custom_dir.is_dir() {
        return Ok(custom_dir);
    }

    get_default_plugin_directory()
}

#[inline]
fn get_default_plugin_directory() -> Result<PathBuf, PluginDirectoryError> {
    let Some(program_data) = super::program_data_path() else {
        return Err(PluginDirectoryError::ProgramDataNotFound);
    };

    let plugin_dir = program_data.join("Technoir/plugins");

    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).map_err(PluginDirectoryError::Io)?;
    }

    if !plugin_dir.is_dir() {
        return Err(PluginDirectoryError::PathIsFile);
    }

    Ok(plugin_dir)
}
