use clap::Parser;
use orfail::OrFail;
use std::collections::BTreeMap;

#[derive(Parser)]
#[clap(version)]
enum Args {
    /// Run a command defined in a .uribo file between the current directory and the root directory
    Run {
        /// Name of the command to run.
        name: String,
    },

    /// Put a command definition to `$PWD/.uribo` file
    Put {
        /// Command name.
        name: String,

        #[clap(flatten)]
        command: Command,
    },

    /// Delete a command definition from `$PWD/.uribo` file
    Delete {
        /// Command name.
        name: String,
    },
}

#[derive(Clone, clap::Args, serde::Serialize, serde::Deserialize)]
struct Command {
    /// Command to run.
    command: String,

    /// Arguments to pass to the command.
    args: Vec<String>,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    match args {
        Args::Run { name } => run(&name).or_fail(),
        Args::Put { name, command } => put(&name, command).or_fail(),
        Args::Delete { name } => delete(&name).or_fail(),
    }
}

fn put(name: &str, command: Command) -> orfail::Result<()> {
    let path = std::env::current_dir().or_fail()?.join(".uribo");
    let mut command_map = if path.exists() {
        let file = std::fs::File::open(&path).or_fail()?;
        serde_json::from_reader(file)
            .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?
    } else {
        BTreeMap::new()
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
    let mut command_map: BTreeMap<String, Command> = serde_json::from_reader(file)
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

fn run(name: &str) -> orfail::Result<()> {
    let Some(command) = find_command(name).or_fail()? else {
        eprintln!("{name:?} command is not defined");
        std::process::exit(1);
    };
    let status = std::process::Command::new(&command.command)
        .args(&command.args)
        .status()
        .or_fail()?;

    let code = status.code().unwrap_or(0);
    std::process::exit(code);
}

fn find_command(name: &str) -> orfail::Result<Option<Command>> {
    let mut dir = std::env::current_dir().or_fail()?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            let file = std::fs::File::open(&path).or_fail()?;
            let mut command_map: BTreeMap<String, Command> = serde_json::from_reader(file)
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
