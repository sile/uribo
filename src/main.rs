use clap::Parser;
use orfail::OrFail;
use std::collections::BTreeMap;

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// Name of the command to run.
    #[clap(default_value = "default")]
    name: String,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    let command = find_command(&args).or_fail()?;
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .or_fail()?;

    let code = status.code().unwrap_or(0);
    std::process::exit(code);
}

fn find_command(args: &Args) -> orfail::Result<String> {
    let mut dir = std::env::current_dir().or_fail()?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            let file = std::fs::File::open(&path).or_fail()?;
            let mut command_map: BTreeMap<String, String> = serde_json::from_reader(file)
                .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?;
            if let Some(command) = command_map.remove(&args.name) {
                return Ok(command);
            }
        }
        dir.pop().or_fail_with(|_| {
            format!(".uribo file defining {:?} command is not found", args.name)
        })?;
    }
}
