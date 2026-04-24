use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::result::Result;

struct Environment {
    manifest_filename: String,
    hellbox_package: String,
}

#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates an isolated environment for installing plugins and dependencies
    /// and creates a blank Hellfile.py file within which to define tasks.
    Init,
    /// Installs all dependencies from the pyproject.toml file.
    Install,
    /// Installs a package and freezes dependencies.
    Add { package: Option<String> },
    /// Uninstalls a package and freezes dependencies.
    Remove { package: Option<String> },
    /// Runs a task defined in Hellbox.py.
    Run { task: Option<String> },
    /// View the defined tasks and their processes.
    Inspect,
    /// Prints information about the local environment.
    Environment,
    #[command(hide = true)]
    Postinstall,
}

fn main() {
    let cli = Cli::parse();
    let environment = Environment {
        manifest_filename: "Hellfile.py".to_owned(),
        hellbox_package: "git+git://github.com/hellboxpy/hellbox.git#egg=hellbox".to_owned(),
    };

    let result = dispatch(environment, cli.command);

    match result {
        Ok(_) => {}
        Err(m) => eprintln!("{}", m),
    }
}

fn dispatch(environment: Environment, command: Commands) -> Result<i32, String> {
    match command {
        Commands::Init => handle_init(environment),
        Commands::Install => handle_install(environment),
        Commands::Add { package } => handle_add(environment, package),
        Commands::Remove { package } => handle_remove(environment, package),
        Commands::Run { task } => handle_run(environment, task),
        Commands::Inspect => handle_inspect(environment),
        Commands::Environment => handle_environment(environment),
        Commands::Postinstall => handle_postinstall(environment),
    }
}

// Handlers

fn handle_init(environment: Environment) -> Result<i32, String> {
    eprintln!("init will now happen");

    create_project()
        .and_then(|_| install_package(&environment.hellbox_package))
        .and_then(|_| create_manifest(&environment.manifest_filename))
}

fn handle_install(_environment: Environment) -> Result<i32, String> {
    eprintln!("install will now happen");

    install_dependencies()
}

fn handle_add(_environment: Environment, package: Option<String>) -> Result<i32, String> {
    eprintln!("install will now happen");

    match package {
        Some(name) => install_package(&name),
        None => install_dependencies(),
    }
}

fn handle_remove(_environment: Environment, package: Option<String>) -> Result<i32, String> {
    eprintln!("install will now happen");

    match package {
        Some(name) => uninstall_package(&name),
        None => Err("a package name is required".to_owned()),
    }
}

fn handle_run(environment: Environment, task: Option<String>) -> Result<i32, String> {
    let name = task.as_deref().unwrap_or("default");

    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        run_task(&environment.manifest_filename, name)
    }
}

fn handle_inspect(environment: Environment) -> Result<i32, String> {
    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        run_inspect(&environment.manifest_filename)
    }
}

fn handle_environment(_environment: Environment) -> Result<i32, String> {
    println!("hell {}", env!("CARGO_PKG_VERSION"));
    check_package_version("hellbox").map(|o| println!("{}", o));
    check_version("python").map(|o| println!("{}", o));
    check_version("uv").map(|o| println!("{}", o));

    Ok(0)
}

fn handle_postinstall(_environment: Environment) -> Result<i32, String> {
    let mut missing = 0;

    eprintln!(
        "{}\nChecking for expected tools...",
        "hell was installed!".green()
    );

    match check_command("uv", vec!["--version"]) {
        Ok(_) => {}
        Err(_) => {
            missing += 1;
            eprintln!(
                "{}\
                 \n\nThe uv tool creates and runs the Python virtual environment \
                 used by hell, and is a required dependency. It can be installed with pip.\
                 \n\n  pip install uv\
                 \n",
                "uv: not found".red().bold()
            )
        }
    }

    if missing == 0 {
        eprintln!("{}", "OK!".green())
    }

    Ok(0)
}

// Actions

fn check_command(command: &str, arguments: Vec<&str>) -> Result<std::process::ExitStatus, String> {
    Command::new(command)
        .args(arguments)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| format!("{} failed to run.", command))
}

fn check_package_version(name: &str) -> Option<String> {
    let output = Command::new("uv")
        .args(vec!["run", "pip", "list"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            let mut packages = stdout.lines();

            packages.find(|l| l.starts_with(name)).map(|v| {
                v.trim_end()
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
            })
        }
        Err(_) => None,
    }
}

fn check_version(command: &str) -> Option<String> {
    let output = Command::new(command).args(vec!["--version"]).output();

    match output {
        Ok(o) => {
            let version = String::from_utf8_lossy(&o.stdout).into_owned();
            Some(version.trim_end().to_owned())
        }
        Err(_) => None,
    }
}

fn create_project() -> Result<i32, String> {
    run_command("uv", vec!["init"])
}

fn create_manifest(filepath: &str) -> Result<i32, String> {
    if !Path::new(filepath).exists() {
        let mut file = File::create(filepath).expect("file wasn't created");
        file.write_all(b"from hellbox import Hellbox\n\nHellbox.autoimport()")
            .map(|_| 0)
            .map_err(|_| "Hellbox.py failed to write".to_owned())
    } else {
        Ok(0)
    }
}

fn install_dependencies() -> Result<i32, String> {
    run_command("uv", vec!["sync"])
}

fn install_package(name: &str) -> Result<i32, String> {
    run_command("uv", vec!["add", name])
}

fn run_command(command: &str, arguments: Vec<&str>) -> Result<i32, String> {
    let mut child = Command::new(command)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Command failed to run");

    let status = child.wait().expect("Command wasn't running");

    match status.code() {
        None => Err(format!("{} failed with no status code", command)),
        Some(code) => Ok(code),
    }
}

fn run_hellbox_commands(filepath: &str, commands: Vec<&str>) -> Result<i32, String> {
    let program = format!(
        "exec(open(\"{}\").read()); import hellbox; {}",
        filepath,
        commands.join("; ")
    );

    run_command("uv", vec!["run", "python", "-c", &program])
}

fn run_inspect(filepath: &str) -> Result<i32, String> {
    run_hellbox_commands(filepath, vec!["hellbox.Hellbox.inspect()"])
}

fn run_task(filepath: &str, name: &str) -> Result<i32, String> {
    run_hellbox_commands(
        filepath,
        vec![&format!("hellbox.Hellbox.run_task(\"{}\")", name)],
    )
}

fn uninstall_package(name: &str) -> Result<i32, String> {
    run_command("uv", vec!["remove", name])
}
