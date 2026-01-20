use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LibraryFolders {
    #[serde(flatten)]
    pub libraries: HashMap<String, LibraryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryEntry {
    pub path: String,
    #[serde(default)]
    pub apps: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AppState {
    pub installdir: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InstallConfigStore {
    #[serde(rename = "Software")]
    pub software: SoftwareConfig,
}

#[derive(Debug, Deserialize)]
pub struct SoftwareConfig {
    #[serde(rename = "Valve")]
    pub valve: ValveConfig,
}

#[derive(Debug, Deserialize)]
pub struct ValveConfig {
    #[serde(rename = "Steam")]
    pub steam: SteamSettings,
}

#[derive(Debug, Deserialize)]
pub struct SteamSettings {
    #[serde(rename = "CompatToolMapping", default)]
    pub compat_tool_mapping: HashMap<String, CompatToolEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompatToolEntry {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserLocalConfigStore {
    #[serde(rename = "Software")]
    pub software: LocalSoftwareConfig,
    #[serde(default)]
    pub friends: Option<FriendsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FriendsConfig {
    #[serde(rename = "PersonaName")]
    pub persona_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocalSoftwareConfig {
    #[serde(rename = "Valve")]
    pub valve: LocalValveConfig,
}

#[derive(Debug, Deserialize)]
pub struct LocalValveConfig {
    #[serde(rename = "Steam")]
    pub steam: LocalSteamSettings,
}

#[derive(Debug, Deserialize)]
pub struct LocalSteamSettings {
    pub apps: Option<HashMap<String, AppLaunchConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct AppLaunchConfig {
    #[serde(rename = "LaunchOptions")]
    pub launch_options: Option<String>,
}
