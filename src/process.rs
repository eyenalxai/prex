use anyhow::{Context, Result};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub fn spawn_and_wait(cmd: Command, log_file: Option<&Path>) -> Result<()> {
    spawn_and_wait_wine(cmd, None, None, false, log_file)
}

fn interrupted_flag() -> Result<&'static Arc<AtomicBool>> {
    static INTERRUPTED: OnceLock<Arc<AtomicBool>> = OnceLock::new();
    if let Some(flag) = INTERRUPTED.get() {
        return Ok(flag);
    }

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);
    ctrlc::set_handler(move || {
        flag_clone.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    let _ = INTERRUPTED.set(flag);
    INTERRUPTED
        .get()
        .context("Interrupted flag was not initialized")
}

pub fn spawn_and_wait_wine(
    mut cmd: Command,
    wine64: Option<&Path>,
    exe_to_kill: Option<&str>,
    kill_explorer: bool,
    log_file: Option<&Path>,
) -> Result<()> {
    if let Some(path) = log_file {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating log directory at {}", parent.display()))?;
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("Opening log file at {}", path.display()))?;
        let file_err = file.try_clone().context("Cloning log file handle")?;
        cmd.stdout(file);
        cmd.stderr(file_err);
    }

    unsafe {
        cmd.pre_exec(|| {
            nix::unistd::setpgid(Pid::from_raw(0), Pid::from_raw(0))?;
            Ok(())
        });
    }

    let mut child = cmd.spawn()?;
    let pid = Pid::from_raw(i32::try_from(child.id()).context("PID out of range")?);

    let interrupted = interrupted_flag()?;
    interrupted.store(false, Ordering::SeqCst);

    loop {
        match child.try_wait()? {
            Some(status) => {
                if status.success() {
                    return Ok(());
                }
                let code = status
                    .code()
                    .map_or_else(|| "unknown".to_string(), |c| c.to_string());
                anyhow::bail!("Command exited with status: {}", code);
            }
            None if interrupted.load(Ordering::SeqCst) => {
                if let (Some(wine), Some(exe)) = (wine64, exe_to_kill) {
                    let _ = Command::new(wine)
                        .args(["taskkill", "/F", "/IM", exe])
                        .output();
                }
                if kill_explorer && let Some(wine) = wine64 {
                    let _ = Command::new(wine)
                        .args(["taskkill", "/F", "/IM", "explorer.exe"])
                        .output();
                    std::thread::sleep(Duration::from_millis(200));
                }
                let _ = kill(Pid::from_raw(-pid.as_raw()), Signal::SIGTERM);
                std::thread::sleep(Duration::from_millis(500));
                let _ = kill(Pid::from_raw(-pid.as_raw()), Signal::SIGKILL);
                let _ = child.wait();
                return Ok(());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

pub fn format_command(cmd: &Command) -> String {
    std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|part| {
            let part_lossy = part.to_string_lossy();
            let part_owned = part_lossy.into_owned();
            match shlex::try_quote(part_owned.as_str()) {
                Ok(value) => value.into_owned(),
                Err(_) => part_owned,
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
