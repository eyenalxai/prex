use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::process::spawn_and_wait;
use crate::steam::Steam;

pub struct LaunchContext {
    pub exe_full_path: PathBuf,
    pub compat_data_path: PathBuf,
    pub proton_path: PathBuf,
}

pub fn resolve_launch_context(
    steam: &Steam,
    appid: &str,
    exe: &Path,
    resolve_in_game_dir: bool,
) -> Result<LaunchContext> {
    let library_path = steam.find_library_for_app(appid)?;
    let install_dir = steam.get_install_dir(&library_path, appid)?;
    let exe_full_path = if resolve_in_game_dir {
        install_dir.join(exe)
    } else if exe.is_absolute() {
        exe.to_path_buf()
    } else {
        std::env::current_dir()?.join(exe)
    };

    if !exe_full_path.exists() {
        if resolve_in_game_dir {
            bail!(
                "Executable not found: {}\nLooking in: {}",
                exe.display(),
                install_dir.display()
            );
        }

        bail!("Executable not found: {}", exe_full_path.display());
    }

    let compat_tool = steam.get_compat_tool(appid)?;
    let compat_tool_name = compat_tool
        .as_ref()
        .and_then(|t| t.name.as_ref())
        .map_or("proton_experimental", String::as_str);

    let proton_path = steam.get_proton_path(&library_path, compat_tool_name)?;
    let compat_data_path = steam.get_compat_data_path(&library_path, appid);

    Ok(LaunchContext {
        exe_full_path,
        compat_data_path,
        proton_path,
    })
}

pub struct ProtonCommand {
    pub proton_path: PathBuf,
    pub exe_path: PathBuf,
    pub compat_data_path: PathBuf,
    pub steam_client_path: PathBuf,
    pub app_id: String,
    pub launch_options: Option<String>,
    pub args: Vec<OsString>,
}

impl ProtonCommand {
    fn build_command(&self) -> Result<String> {
        let proton_path_lossy = self.proton_path.to_string_lossy();
        let proton_path =
            shlex::try_quote(proton_path_lossy.as_ref()).context("Failed to quote proton path")?;
        let exe_path_lossy = self.exe_path.to_string_lossy();
        let exe_path =
            shlex::try_quote(exe_path_lossy.as_ref()).context("Failed to quote exe path")?;
        let proton_cmd = format!("{proton_path} waitforexitandrun {exe_path}");
        let args = self
            .args
            .iter()
            .map(|arg| {
                let arg_lossy = arg.to_string_lossy().into_owned();
                shlex::try_quote(arg_lossy.as_str())
                    .map(|value| value.into_owned())
                    .context("Failed to quote argument")
            })
            .collect::<Result<Vec<_>>>()?;
        let proton_cmd = if args.is_empty() {
            proton_cmd
        } else {
            format!("{proton_cmd} {}", args.join(" "))
        };

        match &self.launch_options {
            Some(opts) if opts.contains("%command%") => Ok(opts.replace("%command%", &proton_cmd)),
            Some(opts) => Ok(format!("{opts} {proton_cmd}")),
            None => Ok(proton_cmd),
        }
    }

    fn build_env(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "STEAM_COMPAT_DATA_PATH",
                self.compat_data_path.to_string_lossy().to_string(),
            ),
            (
                "STEAM_COMPAT_CLIENT_INSTALL_PATH",
                self.steam_client_path.to_string_lossy().to_string(),
            ),
            ("SteamAppId", self.app_id.clone()),
            ("SteamGameId", self.app_id.clone()),
        ]
    }

    pub fn execute(&self, dry_run: bool) -> Result<()> {
        let command_str = self.build_command()?;
        let env_vars = self.build_env();

        if dry_run {
            println!("Environment:");
            for (key, value) in &env_vars {
                println!("  {}={}", key, value);
            }
            println!("\nCommand:");
            println!("  {}", command_str);
            return Ok(());
        }

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&command_str);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        if let Some(parent) = self.exe_path.parent() {
            cmd.current_dir(parent);
        }

        spawn_and_wait(cmd)
    }
}
