use std::path::PathBuf;

fn main() {
    let Ok(path) = find_dot_uribo_file_path().map_err(|e| eprintln!("{e}")) else {
        std::process::exit(1);
    };

    let Ok(command) = std::fs::read_to_string(path).map_err(|e| eprintln!("{e}")) else {
        std::process::exit(1);
    };

    let Ok(status) = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|e| eprintln!("{e}"))
    else {
        std::process::exit(1);
    };
    if let Some(code) = status.code() {
        std::process::exit(code);
    }
}

fn find_dot_uribo_file_path() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            return Ok(path);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(".uribo file is not found in any directory between the current and the root.".to_owned())
}
