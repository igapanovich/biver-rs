use std::process::{Command, Stdio};

pub fn ready() -> bool {
    let status = Command::new("magick").stdout(Stdio::null()).stderr(Stdio::null()).arg("-version").status();
    match status {
        Ok(status) => status.code() == Some(0),
        Err(_) => false,
    }
}
