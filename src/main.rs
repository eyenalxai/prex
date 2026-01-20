mod process;
mod steam;
mod wineserver;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::process::{spawn_and_wait, spawn_and_wait_wine};
use crate::steam::{Steam, get_game_name};
use crate::wineserver::{WineserverInfo, scan_running_games};

#[derive(Parser)]
#[command(name = "parton")]
#[command(about = "Run Windows executables in Proton contexts")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Subcommand)]
enum CommandKind {
    #[command(about = "List installed games configured to use Proton")]
    Ls {
        #[arg(short = 's', long, help = "Path to Steam installation")]
        steam_dir: Option<String>,
    },
    #[command(about = "List currently running Proton games")]
    Ps,
    #[command(about = "Run an executable in a game's Proton prefix")]
    Run {
        #[arg(short = 'n', long, help = "Print the command without executing")]
        dry_run: bool,
        #[arg(short = 's', long, help = "Path to Steam installation")]
        steam_dir: Option<String>,
        #[arg(help = "Steam application ID (e.g. 123456)")]
        appid: String,
        #[arg(help = "Path to executable, relative to game install directory")]
        exe: PathBuf,
    },
    #[command(about = "Run an executable in an already-running game's Proton session")]
    Attach {
        #[arg(short = 'n', long, help = "Print the command without executing")]
        dry_run: bool,
        #[arg(
            long,
            value_name = "WxH",
            default_missing_value = "1280x720",
            num_args = 0..=1,
            require_equals = true,
            help = "Escape gamescope using a virtual desktop"
        )]
        bypass_gamescope: Option<String>,
        #[arg(help = "Steam application ID (e.g. 123456)")]
        appid: String,
        #[arg(help = "Path to Windows executable")]
        exe: PathBuf,
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            help = "Arguments to pass to the executable"
        )]
        args: Vec<OsString>,
    },
    #[command(about = "List Steam users")]
    Users {
        #[arg(short = 's', long, help = "Path to Steam installation")]
        steam_dir: Option<String>,
    },
    #[command(about = "Run an executable with the game's Steam launch options applied")]
    Launch {
        #[arg(short = 'n', long, help = "Print the command without executing")]
        dry_run: bool,
        #[arg(
            short = 'u',
            long,
            help = "Steam user ID (auto-detected if only one user)"
        )]
        user_id: Option<String>,
        #[arg(short = 's', long, help = "Path to Steam installation")]
        steam_dir: Option<String>,
        #[arg(help = "Steam application ID (e.g. 123456)")]
        appid: String,
        #[arg(help = "Path to executable, relative to game install directory")]
        exe: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CommandKind::Ls { steam_dir } => ls(steam_dir),
        CommandKind::Ps => ps(),
        CommandKind::Users { steam_dir } => users(steam_dir),
        CommandKind::Run {
            dry_run,
            steam_dir,
            appid,
            exe,
        } => run(dry_run, steam_dir, &appid, &exe),
        CommandKind::Attach {
            dry_run,
            bypass_gamescope,
            appid,
            exe,
            args,
        } => attach(dry_run, &appid, exe, args, bypass_gamescope),
        CommandKind::Launch {
            dry_run,
            user_id,
            steam_dir,
            appid,
            exe,
        } => launch(dry_run, user_id, steam_dir, &appid, &exe),
    }
}

fn ls(steam_dir: Option<String>) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    for (app_id, name, compat_tool) in steam.list_proton_games()? {
        println!("{}\t{}\t{}", app_id, name, compat_tool);
    }
    Ok(())
}

fn ps() -> Result<()> {
    for info in scan_running_games() {
        let name = get_game_name(&info.compatdata, &info.appid);
        println!("{}\t{}", info.appid, name);
    }
    Ok(())
}

fn users(steam_dir: Option<String>) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    for (user_id, persona_name) in steam.list_users()? {
        println!("{}\t{}", user_id, persona_name.unwrap_or_default());
    }
    Ok(())
}

fn run(dry_run: bool, steam_dir: Option<String>, appid: &str, exe: &Path) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let context = resolve_launch_context(&steam, appid, exe)?;
    let cmd = ProtonCommand {
        proton_path: context.proton_path,
        exe_path: context.exe_full_path,
        compat_data_path: context.compat_data_path,
        steam_client_path: steam.root_path().to_path_buf(),
        app_id: appid.to_string(),
        launch_options: None,
    };

    cmd.execute(dry_run)
}

fn attach(
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

fn launch(
    dry_run: bool,
    user_id: Option<String>,
    steam_dir: Option<String>,
    appid: &str,
    exe: &Path,
) -> Result<()> {
    let steam = Steam::new(steam_dir)?;
    let context = resolve_launch_context(&steam, appid, exe)?;
    let launch_options = steam.get_launch_options(user_id.as_deref(), appid)?;
    let cmd = ProtonCommand {
        proton_path: context.proton_path,
        exe_path: context.exe_full_path,
        compat_data_path: context.compat_data_path,
        steam_client_path: steam.root_path().to_path_buf(),
        app_id: appid.to_string(),
        launch_options,
    };

    cmd.execute(dry_run)
}

struct LaunchContext {
    exe_full_path: PathBuf,
    compat_data_path: PathBuf,
    proton_path: PathBuf,
}

fn resolve_launch_context(steam: &Steam, appid: &str, exe: &Path) -> Result<LaunchContext> {
    let library_path = steam.find_library_for_app(appid)?;
    let install_dir = steam.get_install_dir(&library_path, appid)?;
    let exe_full_path = install_dir.join(exe);

    if !exe_full_path.exists() {
        bail!(
            "Executable not found: {}\nLooking in: {}",
            exe.display(),
            install_dir.display()
        );
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

struct ProtonCommand {
    proton_path: PathBuf,
    exe_path: PathBuf,
    compat_data_path: PathBuf,
    steam_client_path: PathBuf,
    app_id: String,
    launch_options: Option<String>,
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

    fn execute(&self, dry_run: bool) -> Result<()> {
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

fn format_command(cmd: &Command) -> String {
    std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}
