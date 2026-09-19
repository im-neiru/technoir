use std::borrow::Cow;

use rapidhash::RapidHashMap;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest<'s> {
    pub plugin: Plugin<'s>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin<'s> {
    /// Human-readable plugin name.
    pub name: Cow<'s, str>,

    /// Globally unique reverse-domain-style plugin identifier.
    pub id: Cow<'s, str>,

    /// Plugin's own semantic version.
    pub version: Version,

    /// Technoir engine API version targeted by this plugin.
    pub api_version: Version,

    /// Plugin category.
    #[serde(rename = "type")]
    pub plugin_type: PluginType,

    /// Main Luau entry point relative to the plugin root.
    pub entry_point: Cow<'s, str>,

    /// Short plugin description.
    #[serde(default)]
    pub description: Option<Cow<'s, str>>,

    /// Author name or email.
    #[serde(default)]
    pub author: Option<Cow<'s, str>>,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: Option<Cow<'s, str>>,

    /// Pipeline configuration path relative to the plugin root.
    #[serde(default = "default_pipelines")]
    pub pipelines: Cow<'s, str>,

    /// Preview thumbnail path relative to the plugin root.
    #[serde(default)]
    pub icon: Option<Cow<'s, str>>,

    /// Search/filter tags.
    #[serde(default)]
    pub tags: Vec<Cow<'s, str>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PluginType {
    #[serde(rename = "wallpaper")]
    Wallpaper,
}

#[inline(always)]
fn default_pipelines<'s>() -> Cow<'s, str> {
    Cow::Borrowed("pipelines.toml")
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Pipelines<'s>(pub RapidHashMap<String, Pipeline<'s>>);

impl<'s> Pipelines<'s> {
    pub fn get(&self, name: &str) -> Option<&Pipeline> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Pipeline)> {
        self.0.iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline<'s> {
    /// Shader path relative to the plugin's `shaders/` directory.
    pub shader: Cow<'s, str>,

    /// Explicit entry points.
    ///
    /// `None` means the engine asks Naga to select the unique entry point.
    pub entry_points: EntryPoints<'s>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryPoints<'s> {
    #[serde(default)]
    pub vertex: Cow<'s, str>,

    #[serde(default)]
    pub fragment: Cow<'s, str>,
}
