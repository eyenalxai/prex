use anyhow::{anyhow, bail, Context, Result};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths::logs_dir;
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

    let (proton_path, compat_data_path) = steam.resolve_proton_paths(appid)?;

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
    pub use_run_verb: bool,
    pub log_output: bool,
}

impl ProtonCommand {
    fn path_to_string(path: &Path, label: &str) -> Result<String> {
        path.to_str()
            .map(|value| value.to_string())
            .ok_or_else(|| anyhow!("{label} contains invalid UTF-8: {}", path.display()))
    }

    fn os_str_to_string(value: &OsStr, label: &str) -> Result<String> {
        value
            .to_str()
            .map(|value| value.to_string())
            .ok_or_else(|| anyhow!("{label} contains invalid UTF-8"))
    }

    fn build_command(&self) -> Result<String> {
        let proton_path = Self::path_to_string(&self.proton_path, "Proton path")?;
        let proton_path =
            shlex::try_quote(proton_path.as_str()).context("Failed to quote proton path")?;
        let exe_path = Self::path_to_string(&self.exe_path, "Executable path")?;
        let exe_path = shlex::try_quote(exe_path.as_str()).context("Failed to quote exe path")?;
        let verb = if self.use_run_verb {
            "run"
        } else {
            "waitforexitandrun"
        };
        let proton_cmd = format!("{proton_path} {verb} {exe_path}");
        let args = self
            .args
            .iter()
            .map(|arg| {
                let arg = Self::os_str_to_string(arg, "Argument")?;
                shlex::try_quote(arg.as_str())
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

    fn build_env(&self) -> Result<Vec<(&'static str, String)>> {
        Ok(vec![
            (
                "STEAM_COMPAT_DATA_PATH",
                Self::path_to_string(&self.compat_data_path, "Compat data path")?,
            ),
            (
                "STEAM_COMPAT_CLIENT_INSTALL_PATH",
                Self::path_to_string(&self.steam_client_path, "Steam client path")?,
            ),
            ("SteamAppId", self.app_id.clone()),
            ("SteamGameId", self.app_id.clone()),
        ])
    }

    pub fn execute(&self, dry_run: bool) -> Result<()> {
        let command_str = self.build_command()?;
        let env_vars = self.build_env()?;

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

        let log_path = if self.log_output {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let log_path = logs_dir()?.join(format!("{}_{}.log", self.app_id, timestamp));
            println!("Logging to: {}", log_path.display());
            Some(log_path)
        } else {
            None
        };

        spawn_and_wait(cmd, log_path.as_deref())
    }
}
