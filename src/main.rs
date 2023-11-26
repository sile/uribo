use clap::Parser;
use orfail::OrFail;
use std::collections::BTreeMap;

#[derive(Parser)]
#[clap(version)]
enum Args {
    /// Run a command defined in a .uribo file between the current directory and the root directory
    Run {
        /// Name of the command to run.
        #[clap(default_value = "default")]
        name: String,

        /// Shell to run the command.
        #[clap(long, default_value = "sh", env = "URIBO_SHELL")]
        shell: String,
    },

    /// Put a command definition to `$PWD/.uribo` file
    Put {
        /// Command name.
        name: String,

        /// Command to run.
        command: String,

        /// Arguments to pass to the command.
        args: Vec<String>,
    },

    /// Delete a command definition from `$PWD/.uribo` file
    Delete {
        /// Command name.
        name: String,
    },
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    match args {
        Args::Run { name, shell } => run(&name, &shell).or_fail(),
        Args::Put {
            name,
            command,
            args,
        } => put(&name, &command, &args).or_fail(),
        Args::Delete { name } => delete(&name).or_fail(),
    }
}

fn put(name: &str, command: &str, args: &[String]) -> orfail::Result<()> {
    let path = std::env::current_dir().or_fail()?.join(".uribo");
    let mut command_map = if path.exists() {
        let file = std::fs::File::open(&path).or_fail()?;
        serde_json::from_reader(file)
            .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?
    } else {
        BTreeMap::new()
    };

    let command = if args.is_empty() {
        command.to_owned()
    } else {
        format!("{} {}", command, args.join(" "))
    };
    command_map.insert(name.to_owned(), command);

    let mut json = serde_json::to_string_pretty(&command_map).or_fail()?;
    json.push('\n');
    std::fs::write(&path, &json).or_fail()?;

    Ok(())
}

fn delete(name: &str) -> orfail::Result<()> {
    let path = std::env::current_dir().or_fail()?.join(".uribo");
    if !path.exists() {
        eprintln!(".uribo file does not exist in the current directory");
        std::process::exit(1);
    }

    let file = std::fs::File::open(&path).or_fail()?;
    let mut command_map: BTreeMap<String, String> = serde_json::from_reader(file)
        .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?;
    if command_map.remove(name).is_none() {
        eprintln!("{name:?} command is not defined");
        std::process::exit(1);
    }

    let mut json = serde_json::to_string_pretty(&command_map).or_fail()?;
    json.push('\n');
    std::fs::write(&path, &json).or_fail()?;

    Ok(())
}

fn run(name: &str, shell: &str) -> orfail::Result<()> {
    let Some(command) = find_command(name).or_fail()? else {
        eprintln!("{name:?} command is not defined");
        std::process::exit(1);
    };
    let status = std::process::Command::new(shell)
        .arg("-c")
        .arg(command)
        .status()
        .or_fail()?;

    let code = status.code().unwrap_or(0);
    std::process::exit(code);
}

fn find_command(name: &str) -> orfail::Result<Option<String>> {
    let mut dir = std::env::current_dir().or_fail()?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            let file = std::fs::File::open(&path).or_fail()?;
            let mut command_map: BTreeMap<String, String> = serde_json::from_reader(file)
                .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?;
            if let Some(command) = command_map.remove(name) {
                return Ok(Some(command));
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(None)
}
