mod cli;
mod commands;
mod process;
mod proton;
mod steam;
mod wineserver;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, CommandKind};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CommandKind::Ls { steam_dir } => commands::ls(steam_dir),
        CommandKind::Ps => commands::ps(),
        CommandKind::Users { steam_dir } => commands::users(steam_dir),
        CommandKind::Run {
            dry_run,
            steam_dir,
            appid,
            exe,
        } => commands::run(dry_run, steam_dir, &appid, &exe),
        CommandKind::Attach {
            dry_run,
            bypass_gamescope,
            appid,
            exe,
            args,
        } => commands::attach(dry_run, &appid, exe, args, bypass_gamescope),
        CommandKind::Launch {
            dry_run,
            user_id,
            steam_dir,
            appid,
            exe,
        } => commands::launch(dry_run, user_id, steam_dir, &appid, &exe),
    }
}
