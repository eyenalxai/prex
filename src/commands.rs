use anyhow::{Result, bail};
use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::env::{Bash, Elvish, EnvCompleter, Fish, Powershell, Zsh};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::process::{format_command, spawn_and_wait_wine};
use crate::proton::{ProtonCommand, resolve_launch_context};
use crate::steam::{Steam, get_game_name};
use crate::wineserver::{WineserverInfo, scan_running_games};

pub fn ls(steam_dir: Option<String>) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    for (app_id, name, compat_tool) in steam.list_proton_games()? {
        println!("{}\t{}\t{}", app_id, name, compat_tool);
    }
    Ok(())
}

pub fn ps() -> Result<()> {
    for info in scan_running_games() {
        let name = get_game_name(&info.compatdata, &info.appid);
        println!("{}\t{}", info.appid, name);
    }
    Ok(())
}

pub fn users(steam_dir: Option<String>) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    for (user_id, persona_name) in steam.list_users()? {
        println!("{}\t{}", user_id, persona_name.unwrap_or_default());
    }
    Ok(())
}

pub fn run(
    dry_run: bool,
    steam_dir: Option<String>,
    appid: &str,
    exe: &Path,
    args: Vec<OsString>,
) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let context = resolve_launch_context(&steam, appid, exe, false)?;
    let cmd = ProtonCommand {
        proton_path: context.proton_path,
        exe_path: context.exe_full_path,
        compat_data_path: context.compat_data_path,
        steam_client_path: steam.root_path().to_path_buf(),
        app_id: appid.to_string(),
        launch_options: None,
        args,
    };

    cmd.execute(dry_run)
}

pub fn cmd(
    dry_run: bool,
    steam_dir: Option<String>,
    appid: &str,
    args: Vec<OsString>,
) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let library_path = steam.find_library_for_app(appid)?;
    let compat_tool = steam.get_compat_tool(appid)?;
    let compat_tool_name = compat_tool
        .as_ref()
        .and_then(|t| t.name.as_ref())
        .map_or("proton_experimental", String::as_str);
    let proton_path = steam.get_proton_path(&library_path, compat_tool_name)?;
    let compat_data_path = steam.get_compat_data_path(&library_path, appid);
    let exe_path = compat_data_path.join("prex/drive_c/windows/system32/cmd.exe");

    if !exe_path.exists() {
        bail!("Executable not found: {}", exe_path.display());
    }

    let cmd = ProtonCommand {
        proton_path,
        exe_path,
        compat_data_path,
        steam_client_path: steam.root_path().to_path_buf(),
        app_id: appid.to_string(),
        launch_options: None,
        args,
    };

    cmd.execute(dry_run)
}

pub fn attach(
    dry_run: bool,
    appid: &str,
    exe: PathBuf,
    args: Vec<OsString>,
    bypass_gamescope: Option<String>,
) -> Result<()> {
    let info = WineserverInfo::find_by_appid(appid)?;
    let cmd = info.wine_command(exe.as_os_str(), &args, bypass_gamescope.as_deref())?;
    let exe_name = exe.file_name().and_then(|n| n.to_str());

    if dry_run {
        println!("{}", format_command(&cmd));
        return Ok(());
    }

    spawn_and_wait_wine(
        cmd,
        Some(&info.wine64),
        exe_name,
        bypass_gamescope.is_some(),
    )
}

pub fn launch(
    dry_run: bool,
    user_id: Option<String>,
    steam_dir: Option<String>,
    appid: &str,
    exe: &Path,
    args: Vec<OsString>,
) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let context = resolve_launch_context(&steam, appid, exe, true)?;
    let launch_options = steam.get_launch_options(user_id.as_deref(), appid)?;
    let cmd = ProtonCommand {
        proton_path: context.proton_path,
        exe_path: context.exe_full_path,
        compat_data_path: context.compat_data_path,
        steam_client_path: steam.root_path().to_path_buf(),
        app_id: appid.to_string(),
        launch_options,
        args,
    };

    cmd.execute(dry_run)
}

pub fn path(steam_dir: Option<String>, appid: &str) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let library_path = steam.find_library_for_app(appid)?;
    let compat_data_path = steam.get_compat_data_path(&library_path, appid);
    let prefix_path = compat_data_path.join("pfx");

    if !prefix_path.exists() {
        bail!("Prefix not found: {}", prefix_path.display());
    }

    println!("{}", prefix_path.display());
    Ok(())
}

pub fn completions(shell: Shell) -> Result<()> {
    let cmd = crate::cli::Cli::command();
    let name = cmd.get_name().to_string();
    let bin = cmd.get_bin_name().unwrap_or(cmd.get_name()).to_string();
    let completer = std::env::current_exe()
        .ok()
        .map_or_else(|| bin.clone(), |path| path.to_string_lossy().to_string());
    let mut stdout = std::io::stdout();
    let env_completer: &dyn EnvCompleter = match shell {
        Shell::Bash => &Bash,
        Shell::Zsh => &Zsh,
        Shell::Fish => &Fish,
        Shell::Elvish => &Elvish,
        Shell::PowerShell => &Powershell,
        _ => bail!("Unsupported shell for dynamic completions"),
    };
    env_completer.write_registration("COMPLETE", &name, &bin, &completer, &mut stdout)?;
    Ok(())
}
