mod cli;
mod commands;
mod completers;
mod db;
mod paths;
mod process;
mod proton;
mod steam;
mod wineserver;

use anyhow::Result;
use clap::{CommandFactory as _, Parser};
use clap_complete::CompleteEnv;
use cli::{Cli, CommandKind, MmAction};

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
            single_instance,
            appid,
            exe,
            args,
        } => commands::run(dry_run, steam_dir, &appid, &exe, args, single_instance),
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
        CommandKind::Mm { action } => match action {
            MmAction::Add {
                steam_dir,
                appid,
                exe,
            } => commands::mm_add(steam_dir, &appid, &exe),
            MmAction::Remove { appid } => commands::mm_remove(&appid),
            MmAction::Ls => commands::mm_list(),
            MmAction::SetActive { appid } => commands::mm_set_active(&appid),
        },
        CommandKind::Nxm { url } => commands::nxm(&url),
        CommandKind::Completions { shell } => commands::completions(shell),
    }
}
