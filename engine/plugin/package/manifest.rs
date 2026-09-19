use rapidhash::RapidHashMap;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub plugin: Plugin,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub enum PluginType {
    #[serde(rename = "wallpaper")]
    Wallpaper,
}

#[inline(always)]
fn default_pipelines() -> String {
    String::from("pipelines.toml")
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Pipelines(pub RapidHashMap<String, Pipeline>);

impl Pipelines {
    pub fn get(&self, name: &str) -> Option<&Pipeline> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Pipeline)> {
        self.0.iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    /// Shader path relative to the plugin's `shaders/` directory.
    pub shader: String,

    /// Explicit entry points.
    ///
    /// `None` means the engine asks Naga to select the unique entry point.
    #[serde(default)]
    pub entry_points: Option<EntryPoints>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryPoints {
    #[serde(default)]
    pub vertex: Option<String>,

    #[serde(default)]
    pub fragment: Option<String>,
}
