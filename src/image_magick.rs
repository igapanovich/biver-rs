use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub fn ready() -> bool {
    let status = Command::new("magick").stdout(Stdio::null()).stderr(Stdio::null()).arg("-version").status();
    match status {
        Ok(status) => status.code() == Some(0),
        Err(_) => false,
    }
}

pub fn create_preview(input: &Path, preview: &Path) -> io::Result<()> {
    let mut preview_with_prefix = OsString::from("jpg:");
    preview_with_prefix.push(preview);

    let status = Command::new("magick")
        .arg(input)
        .arg("-flatten")
        .arg("-thumbnail")
        .arg("1024x1024>")
        .arg(preview_with_prefix)
        .status();

    map_imagemagick_status(status)
}

fn map_imagemagick_status(status_result: io::Result<ExitStatus>) -> io::Result<()> {
    status_result.and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "ImageMagick failed."))
        }
    })
}
