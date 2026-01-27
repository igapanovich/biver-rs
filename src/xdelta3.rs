use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::{fs, io};

pub trait XDelta3Env {
    fn xdelta3_path(&self) -> Option<&Path>;
}

pub fn ready(env: &impl XDelta3Env) -> bool {
    let status = xdelta3_command(env).arg("-V").status();
    match status {
        Ok(status) => status.code() == Some(0),
        Err(_) => false,
    }
}

pub fn create_patch(env: &impl XDelta3Env, old: &Path, new: &Path, patch: &Path) -> io::Result<()> {
    let status = xdelta3_command(env)
        .arg("-e") // compress
        .arg("-s") // source
        .arg(old)
        .arg(new)
        .arg(patch)
        .status();

    map_xdelta3_status(status)
}

pub fn apply_patch(env: &impl XDelta3Env, old: &Path, patch: &Path, new: &Path) -> io::Result<()> {
    fs::remove_file(new)?;

    let status = xdelta3_command(env)
        .arg("-d") // decompress
        .arg("-s") // source
        .arg(old)
        .arg(patch)
        .arg(new)
        .status();

    map_xdelta3_status(status)
}

fn map_xdelta3_status(status_result: io::Result<ExitStatus>) -> io::Result<()> {
    status_result.and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "xdelta3 failed."))
        }
    })
}

fn xdelta3_command(env: &impl XDelta3Env) -> Command {
    let mut xdelta3_path = env.xdelta3_path();
    let xdelta3_path = xdelta3_path.get_or_insert_with(|| Path::new("xdelta3"));

    let mut command = Command::new(xdelta3_path);
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
}
