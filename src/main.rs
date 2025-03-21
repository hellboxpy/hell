#[macro_use]
extern crate clap;

use ansi_term::Colour::{Green, Red, Yellow};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::result::Result;

struct Environment {
    manifest_filename: String,
    hellbox_package: String,
}

fn main() {
    let app = create_application();
    let matches = app.get_matches();
    let environment = Environment {
        manifest_filename: "Hellfile.py".to_owned(),
        hellbox_package: "git+git://github.com/hellboxpy/hellbox.git#egg=hellbox".to_owned(),
    };

    let result = dispatch(environment, matches.subcommand());

    match result {
        Ok(_) => {}
        Err(m) => eprintln!("{}", m),
    }
}

fn create_application<'a, 'b>() -> App<'a, 'b> {
    let init = SubCommand::with_name("init").about(
        "Creates an isolated environment in for installing plugins and dependencies \
         and creates a blank Hellfile.py file within which to define tasks.",
    );

    let install =
        SubCommand::with_name("install").about("Installs all dependencies from the pyproject.toml file.");

    let add = SubCommand::with_name("add")
        .about("Installs a package and freezes dependencies.")
        .arg(Arg::with_name("package"));

    let remove = SubCommand::with_name("remove")
        .about("Uninstalls a package and freezes dependencies.")
        .arg(Arg::with_name("package"));

    let run = SubCommand::with_name("run")
        .about("Runs a task defined in Hellbox.py.")
        .arg(Arg::with_name("task"));

    let inspect =
        SubCommand::with_name("inspect").about("View the defined tasks and their processes.");

    let environment = SubCommand::with_name("environment")
        .about("Prints information about the local environment.");

    let postinstall = SubCommand::with_name("_postinstall").setting(AppSettings::Hidden);

    App::new("hell")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(init)
        .subcommand(install)
        .subcommand(add)
        .subcommand(remove)
        .subcommand(run)
        .subcommand(inspect)
        .subcommand(environment)
        .subcommand(postinstall)
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
}

fn dispatch<'a>(
    environment: Environment,
    subcommand: (&str, Option<&ArgMatches<'a>>),
) -> Result<i32, String> {
    match subcommand {
        ("init", Some(_)) => handle_init(environment),
        ("install", Some(_)) => handle_install(environment),
        ("add", Some(matches)) => handle_add(environment, matches),
        ("remove", Some(matches)) => handle_remove(environment, matches),
        ("run", Some(matches)) => handle_run(environment, matches),
        ("inspect", Some(_)) => handle_inspect(environment),
        ("environment", Some(_)) => handle_environment(environment),
        ("_postinstall", Some(_)) => handle_postinstall(environment),
        (_, _) => Err("Subcommand not provided".to_owned()),
    }
}

// Handlers

fn handle_init<'a>(environment: Environment) -> Result<i32, String> {
    eprintln!("init will now happen");

    create_project()
        .and_then(|_| install_package(&environment.hellbox_package))
        .and_then(|_| create_manifest(&environment.manifest_filename))
}

fn handle_install<'a>(_environment: Environment) -> Result<i32, String> {
    eprintln!("install will now happen");

    install_dependencies()
}

fn handle_add<'a>(_environment: Environment, matches: &ArgMatches<'a>) -> Result<i32, String> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => install_package(&name),
        None => install_dependencies(),
    }
}

fn handle_remove<'a>(_environment: Environment, matches: &ArgMatches<'a>) -> Result<i32, String> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => uninstall_package(&name),
        None => Err("a package name is required".to_owned()),
    }
}

fn handle_run<'a>(environment: Environment, matches: &ArgMatches<'a>) -> Result<i32, String> {
    let name = matches.value_of("task").unwrap_or("default");

    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        // Maybe init?
        run_task(&environment.manifest_filename, name)
    }
}

fn handle_inspect<'a>(environment: Environment) -> Result<i32, String> {
    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        // Maybe init?
        run_inspect(&environment.manifest_filename)
    }
}

fn handle_environment(_environment: Environment) -> Result<i32, String> {
    println!("hell {}", crate_version!());
    check_package_version("hellbox").map(|o| println!("{}", o));
    check_version("python").map(|o| println!("{}", o));
    check_version("uv").map(|o| println!("{}", o));

    Ok(0)
}

fn handle_postinstall(_environment: Environment) -> Result<i32, String> {
    let mut missing = 0;

    eprintln!(
        "{}\nChecking for expected tools...",
        Green.paint("hell was installed!")
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
                Red.bold().paint("uv: not found")
            )
        }
    }

    if missing == 0 {
        eprintln!("{}", Green.paint("OK!"))
    }

    Ok(0)
}

// Actions

fn check_command<'a>(
    command: &str,
    arguments: Vec<&'a str>,
) -> Result<std::process::ExitStatus, String> {
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
            let stdout = String::from_utf8(o.stdout).unwrap();
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
            let version = String::from_utf8(o.stdout).unwrap();
            Some(version.trim_end().to_owned())
        }
        Err(_) => None,
    }
}

fn create_project<'a>() -> Result<i32, String> {
    run_command("uv", vec!["init"])
}

fn create_manifest<'a>(filepath: &str) -> Result<i32, String> {
    if !Path::new(filepath).exists() {
        let mut file = File::create(filepath).expect("file wasn't created");
        file.write_all(b"from hellbox import Hellbox\n\nHellbox.autoimport()")
            .map(|_| 0)
            .map_err(|_| "Hellbox.py failed to write".to_owned())
    } else {
        Ok(0)
    }
}

fn install_dependencies<'a>() -> Result<i32, String> {
    run_command("uv", vec!["sync"])
}

fn install_package<'a>(name: &str) -> Result<i32, String> {
    run_command("uv", vec!["add", name])
}

fn run_command<'a>(command: &str, arguments: Vec<&'a str>) -> Result<i32, String> {
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

fn run_hellbox_commands<'a>(filepath: &str, commands: Vec<&str>) -> Result<i32, String> {
    let program = format!(
        "exec(open(\"{}\").read()); import hellbox; {}",
        filepath,
        commands.join("; ")
    );

    run_command("uv", vec!["run", "python", "-c", &program])
}

fn run_inspect<'a>(filepath: &str) -> Result<i32, String> {
    run_hellbox_commands(filepath, vec!["hellbox.Hellbox.inspect()"])
}

fn run_task<'a>(filepath: &str, name: &str) -> Result<i32, String> {
    run_hellbox_commands(
        filepath,
        vec![&format!("hellbox.Hellbox.run_task(\"{}\")", name)],
    )
}

fn uninstall_package<'a>(name: &str) -> Result<i32, String> {
    run_command("uv", vec!["remove", name])
}
