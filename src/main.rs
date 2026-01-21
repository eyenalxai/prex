mod cli;
mod commands;
mod completers;
mod process;
mod proton;
mod steam;
mod wineserver;

use anyhow::Result;
use clap::{CommandFactory as _, Parser};
use clap_complete::CompleteEnv;
use cli::{Cli, CommandKind};

fn main() -> Result<()> {
    CompleteEnv::with_factory(Cli::command).complete();

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
            args,
        } => commands::run(dry_run, steam_dir, &appid, &exe, args),
        CommandKind::Cmd {
            dry_run,
            steam_dir,
            appid,
            args,
        } => commands::cmd(dry_run, steam_dir, &appid, args),
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
            args,
        } => commands::launch(dry_run, user_id, steam_dir, &appid, &exe, args),
        CommandKind::Path { steam_dir, appid } => commands::path(steam_dir, &appid),
        CommandKind::Completions { shell } => commands::completions(shell),
    }
}
