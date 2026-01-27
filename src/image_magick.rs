use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub trait ImageMagickEnv {
    fn image_magick_path(&self) -> Option<&Path>;
}

pub fn ready(env: &impl ImageMagickEnv) -> bool {
    let status = image_magick_command(env).arg("-version").status();
    match status {
        Ok(status) => status.code() == Some(0),
        Err(_) => false,
    }
}

pub fn create_preview(env: &impl ImageMagickEnv, input: &Path, preview: &Path) -> io::Result<()> {
    let mut preview_with_prefix = OsString::from("jpg:");
    preview_with_prefix.push(preview);

    let status = image_magick_command(env)
        .arg(input)
        .arg("-flatten")
        .arg("-thumbnail")
        .arg("1024x1024>")
        .arg(preview_with_prefix)
        .status();

    map_image_magick_status(status)
}

fn map_image_magick_status(status_result: io::Result<ExitStatus>) -> io::Result<()> {
    status_result.and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "ImageMagick failed."))
        }
    })
}

fn image_magick_command(env: &impl ImageMagickEnv) -> Command {
    let mut image_magick_path = env.image_magick_path();
    let image_magick_path = image_magick_path.get_or_insert_with(|| Path::new("magick"));

    let mut command = Command::new(image_magick_path);
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
}
