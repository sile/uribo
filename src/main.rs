use clap::Parser;
use orfail::OrFail;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const ENV_DEFAULT_CONFIG_PATH: &str = "URIBO_DEFAULT_CONFIG_PATH";

#[derive(Parser)]
#[clap(version)]
enum Args {
    /// Run a command defined in a .uribo file between the current directory and the root directory
    Run {
        /// Name of the command to run.
        name: String,

        /// Extra arguments to pass to the command.
        extra_args: Vec<String>,
    },

    /// Find a command defined in a .uribo file between the current directory and the root directory
    Find {
        /// Name of the command to find.
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

    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Clone, clap::Args, serde::Serialize, serde::Deserialize)]
struct Command {
    /// Command to run.
    command: String,

    /// Arguments to pass to the command.
    args: Vec<String>,

    /// Working directory to run the command.
    ///
    /// Relative paths are resolved against the directory containing the .uribo file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[clap(long, short)]
    working_dir: Option<PathBuf>,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    match args {
        Args::Run {
            name,
            extra_args: args,
        } => run(&name, args).or_fail(),
        Args::Find { name } => find(&name).or_fail(),
        Args::Put { name, command } => put(&name, command).or_fail(),
        Args::Delete { name } => delete(&name).or_fail(),
        Args::External(mut args) => {
            (!args.is_empty()).or_fail_with(|_| "No subcommand given".to_owned())?;
            let name = args.remove(0);
            run(&name, args).or_fail()
        }
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

fn run(name: &str, extra_args: Vec<String>) -> orfail::Result<()> {
    let Some((mut command, config_path)) = find_command(name).or_fail()? else {
        eprintln!("{name:?} command is not defined");
        std::process::exit(1);
    };
    let uribo_dir = config_path.parent().or_fail()?;

    let mut cmd = std::process::Command::new(&command.command);
    if let Some(dir) = command.working_dir {
        cmd.current_dir(uribo_dir.join(dir));
    }
    command.args.extend(extra_args);
    let status = cmd.args(&command.args).status().or_fail()?;

    let code = status.code().unwrap_or(0);
    std::process::exit(code);
}

fn find(name: &str) -> orfail::Result<()> {
    let Some((command, config_path)) = find_command(name).or_fail()? else {
        eprintln!("{name:?} command is not defined");
        std::process::exit(1);
    };

    #[derive(serde::Serialize)]
    struct Output {
        command: Command,
        config_path: PathBuf,
    }

    let output = Output {
        command,
        config_path,
    };
    serde_json::to_writer_pretty(std::io::stdout(), &output).or_fail()?;
    println!();
    Ok(())
}

fn find_command(name: &str) -> orfail::Result<Option<(Command, PathBuf)>> {
    let mut dir = std::env::current_dir().or_fail()?;
    loop {
        let path = dir.join(".uribo");
        if path.exists() {
            if let Some(command) = find_command_from_path(name, &path).or_fail()? {
                return Ok(Some((command, path)));
            }
        }
        if !dir.pop() {
            break;
        }
    }

    if let Ok(path) = std::env::var(ENV_DEFAULT_CONFIG_PATH) {
        let path: &Path = path.as_ref();
        if let Some(command) = find_command_from_path(name, &path).or_fail()? {
            return Ok(Some((command, path.to_owned())));
        }
    }

    Ok(None)
}

fn find_command_from_path<P: AsRef<Path>>(
    command_name: &str,
    path: P,
) -> orfail::Result<Option<Command>> {
    let path = path.as_ref();
    if path.exists() {
        let file = std::fs::File::open(&path).or_fail()?;
        let mut command_map: BTreeMap<String, Command> = serde_json::from_reader(file)
            .or_fail_with(|e| format!("failed to parse {}: {e}", path.display()))?;
        if let Some(command) = command_map.remove(command_name) {
            return Ok(Some(command));
        }
    }
    Ok(None)
}
