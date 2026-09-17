use semver::Version;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub plugin: Plugin,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    /// Human-readable plugin name.
    pub name: String,

    /// Globally unique reverse-domain-style plugin identifier.
    pub id: String,

    /// Plugin's own semantic version.
    pub version: Version,

    /// Technoir engine API version targeted by this plugin.
    pub api_version: Version,

    /// Plugin category.
    #[serde(rename = "type")]
    pub plugin_type: PluginType,

    /// Main Luau entry point relative to the plugin root.
    pub entry_point: String,

    /// Short plugin description.
    #[serde(default)]
    pub description: Option<String>,

    /// Author name or email.
    #[serde(default)]
    pub author: Option<String>,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: Option<String>,

    /// Pipeline configuration path relative to the plugin root.
    #[serde(default = "default_pipelines")]
    pub pipelines: String,

    /// Preview thumbnail path relative to the plugin root.
    #[serde(default)]
    pub icon: Option<String>,

    /// Search/filter tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum PluginType {
    #[serde(rename = "wallpaper")]
    Wallpaper,
}

#[inline(always)]
fn default_pipelines() -> String {
    String::from("pipelines.toml")
}
