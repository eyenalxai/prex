use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prex")]
#[command(about = "Run Windows executables in Proton contexts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: CommandKind,
}

#[derive(Subcommand)]
pub enum CommandKind {
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
        #[arg(help = "Path to Windows executable")]
        exe: PathBuf,
    },
    #[command(about = "Open cmd.exe in a game's Proton prefix")]
    Cmd {
        #[arg(short = 'n', long, help = "Print the command without executing")]
        dry_run: bool,
        #[arg(short = 's', long, help = "Path to Steam installation")]
        steam_dir: Option<String>,
        #[arg(help = "Steam application ID (e.g. 123456)")]
        appid: String,
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
        #[arg(
            help = "Path to executable, relative to the game install root (example: Game/ersc_launcher.exe)"
        )]
        exe: PathBuf,
    },
}
