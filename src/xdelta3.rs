use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::{fs, io};

pub fn ready() -> bool {
    let status = Command::new("xdelta3").stdout(Stdio::null()).stderr(Stdio::null()).arg("-V").status();
    match status {
        Ok(status) => status.code() == Some(0),
        Err(_) => false,
    }
}

pub fn create_patch(old: &Path, new: &Path, patch: &Path) -> io::Result<()> {
    let status = Command::new("xdelta3")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-e") // compress
        .arg("-s") // source
        .arg(old)
        .arg(new)
        .arg(patch)
        .status();

    map_xdelta3_status(status)
}

pub fn apply_patch(old: &Path, patch: &Path, new: &Path) -> io::Result<()> {
    fs::remove_file(new)?;

    let status = Command::new("xdelta3")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
