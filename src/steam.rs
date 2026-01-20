pub mod vdf;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::steam::vdf::{
    AppLaunchConfig, AppState, CompatToolEntry, InstallConfigStore, LibraryFolders,
    UserLocalConfigStore,
};

pub struct Steam {
    root: PathBuf,
}

impl Steam {
    pub fn new(steam_dir: Option<String>) -> Result<Self> {
        let root = match steam_dir {
            Some(dir) => PathBuf::from(dir),
            None => {
                let home = std::env::var("HOME").context("HOME environment variable not set")?;
                PathBuf::from(home).join(".local/share/Steam")
            }
        };

        if !root.exists() {
            bail!("Steam directory not found at {}", root.display());
        }

        Ok(Self { root })
    }

    pub fn find_library_for_app(&self, app_id: &str) -> Result<PathBuf> {
        let library_folders_path = self.root.join("config/libraryfolders.vdf");
        let content = fs::read_to_string(&library_folders_path)
            .context("Failed to read libraryfolders.vdf")?;

        let folders: LibraryFolders =
            keyvalues_serde::from_str(&content).context("Failed to parse libraryfolders.vdf")?;

        for entry in folders.libraries.values() {
            if entry.apps.contains_key(app_id) {
                return Ok(PathBuf::from(&entry.path));
            }
        }

        bail!("App {} not found in any Steam library", app_id);
    }

    pub fn get_install_dir(&self, library_path: &Path, app_id: &str) -> Result<PathBuf> {
        let manifest_path = library_path
            .join("steamapps")
            .join(format!("appmanifest_{}.acf", app_id));

        let content = fs::read_to_string(&manifest_path).context("Failed to read app manifest")?;

        let manifest: AppState =
            keyvalues_serde::from_str(&content).context("Failed to parse app manifest")?;

        Ok(library_path
            .join("steamapps")
            .join("common")
            .join(&manifest.installdir))
    }

    pub fn get_compat_tool(&self, app_id: &str) -> Result<Option<CompatToolEntry>> {
        let config_path = self.root.join("config/config.vdf");
        let content = fs::read_to_string(&config_path).context("Failed to read config.vdf")?;

        let config: InstallConfigStore =
            keyvalues_serde::from_str(&content).context("Failed to parse config.vdf")?;

        let mapping = &config.software.valve.steam.compat_tool_mapping;

        Ok(mapping.get(app_id).cloned())
    }

    /// List all installed games that have a Proton compatdata prefix.
    /// Returns Vec<(app_id, game_name, compat_tool_name)>
    pub fn list_proton_games(&self) -> Result<Vec<(String, String, String)>> {
        let config_path = self.root.join("config/config.vdf");
        let content = fs::read_to_string(&config_path).context("Failed to read config.vdf")?;

        let config: InstallConfigStore =
            keyvalues_serde::from_str(&content).context("Failed to parse config.vdf")?;
        let mapping = &config.software.valve.steam.compat_tool_mapping;

        let library_folders_path = self.root.join("config/libraryfolders.vdf");
        let content = fs::read_to_string(&library_folders_path)
            .context("Failed to read libraryfolders.vdf")?;

        let folders: LibraryFolders =
            keyvalues_serde::from_str(&content).context("Failed to parse libraryfolders.vdf")?;

        let mut games = Vec::new();
        let mut seen = HashSet::new();

        for entry in folders.libraries.values() {
            let library_path = PathBuf::from(&entry.path);
            let compatdata_path = library_path.join("steamapps").join("compatdata");
            let compat_entries = match fs::read_dir(&compatdata_path) {
                Ok(entries) => entries,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => return Err(err).context("Failed to read compatdata directory"),
            };

            for entry in compat_entries {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let app_id = match path.file_name().and_then(|name| name.to_str()) {
                    Some(name) if name.chars().all(|c| c.is_ascii_digit()) => name.to_string(),
                    _ => continue,
                };

                if !seen.insert(app_id.clone()) {
                    continue;
                }

                let manifest_path = library_path
                    .join("steamapps")
                    .join(format!("appmanifest_{}.acf", app_id));
                let manifest_content = match fs::read_to_string(&manifest_path) {
                    Ok(content) => content,
                    Err(_) => continue,
                };

                let manifest: AppState = match keyvalues_serde::from_str(&manifest_content) {
                    Ok(manifest) => manifest,
                    Err(_) => continue,
                };

                if Self::is_runtime_app(&manifest) {
                    continue;
                }

                let game_name = manifest.name.unwrap_or_else(|| "(unknown)".to_string());
                let compat_tool_name = mapping
                    .get(&app_id)
                    .and_then(|entry| entry.name.as_ref())
                    .filter(|name| !name.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());

                games.push((app_id, game_name, compat_tool_name));
            }
        }

        // Sort by app_id for consistent output
        games.sort_by(|a, b| {
            a.0.parse::<u64>()
                .unwrap_or(0)
                .cmp(&b.0.parse::<u64>().unwrap_or(0))
        });

        Ok(games)
    }

    fn is_runtime_app(manifest: &AppState) -> bool {
        let install_dir = manifest.installdir.as_str();
        if install_dir.starts_with("Proton") || install_dir.starts_with("SteamLinuxRuntime") {
            return true;
        }

        match manifest.name.as_deref() {
            Some("Steamworks Common Redistributables") => true,
            Some(name) => name.starts_with("Proton") || name.starts_with("Steam Linux Runtime"),
            None => false,
        }
    }

    pub fn get_launch_options(
        &self,
        user_id: Option<&str>,
        app_id: &str,
    ) -> Result<Option<String>> {
        let user_id = match user_id {
            Some(id) => id.to_string(),
            None => self.detect_user_id()?,
        };

        let local_config_path = self
            .root
            .join("userdata")
            .join(&user_id)
            .join("config/localconfig.vdf");

        let content =
            fs::read_to_string(&local_config_path).context("Failed to read localconfig.vdf")?;

        let config: UserLocalConfigStore =
            keyvalues_serde::from_str(&content).context("Failed to parse localconfig.vdf")?;

        let apps = config.software.valve.steam.apps;

        let launch_options = apps
            .as_ref()
            .and_then(|apps| apps.get(app_id))
            .and_then(|app: &AppLaunchConfig| app.launch_options.clone());

        Ok(launch_options)
    }

    fn detect_user_id(&self) -> Result<String> {
        let users = self.list_users()?;

        match users.as_slice() {
            [] => bail!("No Steam users found in userdata directory"),
            [(user_id, _)] => Ok(user_id.clone()),
            _ => bail!(
                "Multiple Steam users found: {}. Use --user-id to specify one",
                users
                    .iter()
                    .map(|(id, _)| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    pub fn get_proton_path(&self, library_path: &Path, compat_tool_name: &str) -> Result<PathBuf> {
        let proton_dir = if compat_tool_name == "proton_experimental" {
            "Proton - Experimental"
        } else {
            compat_tool_name
        };

        let proton_path = library_path
            .join("steamapps")
            .join("common")
            .join(proton_dir)
            .join("proton");

        if proton_path.exists() {
            return Ok(proton_path);
        }

        let steam_proton_path = self
            .root
            .join("steamapps")
            .join("common")
            .join(proton_dir)
            .join("proton");

        if steam_proton_path.exists() {
            return Ok(steam_proton_path);
        }

        bail!(
            "Proton not found at {} or {}",
            proton_path.display(),
            steam_proton_path.display()
        )
    }

    pub fn get_compat_data_path(&self, library_path: &Path, app_id: &str) -> PathBuf {
        library_path
            .join("steamapps")
            .join("compatdata")
            .join(app_id)
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// List all Steam users found in userdata directory.
    /// Returns Vec<(user_id, persona_name)>
    pub fn list_users(&self) -> Result<Vec<(String, Option<String>)>> {
        let userdata_path = self.root.join("userdata");
        let mut users: Vec<(String, Option<String>)> = Vec::new();

        for entry in fs::read_dir(&userdata_path).context("Failed to read userdata directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name()
            {
                let name_str = name.to_string_lossy();
                if name_str.chars().all(|c| c.is_ascii_digit()) {
                    let user_id = name_str.to_string();

                    // Try to get persona name from localconfig.vdf
                    let persona_name = self.get_persona_name(&user_id);

                    users.push((user_id, persona_name));
                }
            }
        }

        // Sort by user_id for consistent output
        users.sort_by(|a, b| {
            a.0.parse::<u64>()
                .unwrap_or(0)
                .cmp(&b.0.parse::<u64>().unwrap_or(0))
        });

        Ok(users)
    }

    fn get_persona_name(&self, user_id: &str) -> Option<String> {
        let local_config_path = self
            .root
            .join("userdata")
            .join(user_id)
            .join("config/localconfig.vdf");

        let content = fs::read_to_string(&local_config_path).ok()?;
        let config: UserLocalConfigStore = keyvalues_serde::from_str(&content).ok()?;

        config.friends.and_then(|f| f.persona_name)
    }
}

#[must_use]
pub fn get_game_name(compatdata: &Path, app_id: &str) -> String {
    let steamapps = match compatdata.parent().and_then(|p| p.parent()) {
        Some(p) => p,
        None => return "(unknown)".to_string(),
    };

    let manifest_path = steamapps.join(format!("appmanifest_{}.acf", app_id));
    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return "(unknown)".to_string(),
    };

    let manifest: AppState = match keyvalues_serde::from_str(&content) {
        Ok(m) => m,
        Err(_) => return "(unknown)".to_string(),
    };

    manifest.name.unwrap_or_else(|| "(unknown)".to_string())
}
