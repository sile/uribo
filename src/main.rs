use orfail::OrFail;
use std::path::PathBuf;

fn main() -> orfail::Result<()> {
    let path = find_dot_uribo_file_path().or_fail()?;
    let command = std::fs::read_to_string(path).or_fail()?;

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .or_fail()?;

    let code = status.code().unwrap_or(0);
    std::process::exit(code);
}

fn find_dot_uribo_file_path() -> orfail::Result<PathBuf> {
    let mut dir = std::env::current_dir().or_fail()?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            return Ok(path);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(orfail::Failure::new(
        ".uribo file is not found in any directory between the current and the root.",
    ))
}
