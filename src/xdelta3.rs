use std::process::Command;

pub fn ready() -> bool {
    let output = Command::new("xdelta3").arg("-V").output();
    match output {
        Ok(output) => output.status.code() == Some(0),
        Err(_) => false,
    }
}
