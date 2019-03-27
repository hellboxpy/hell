#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::result::Result;

struct Environment {
    manifest_filename: String,
    hellbox_package: String,
}

fn main() {
    let init = SubCommand::with_name("init").about(
        "Creates a isolated environment in .hellbox for installing \
         plugins and dependencies and creates a blank Hellfile to define \
         tasks within.",
    );

    let install = SubCommand::with_name("install")
        .about(
            "Installs a package and freezes dependencies, or installs all \
             dependencies from requirements.txt if no package specified",
        )
        .arg(Arg::with_name("package"));

    let uninstall = SubCommand::with_name("uninstall")
        .about("Uninstalls a package and freezes dependencies")
        .arg(Arg::with_name("package"));

    let run = SubCommand::with_name("run")
        .about("Runs a task defined in %s")
        .arg(Arg::with_name("task"));

    let inspect =
        SubCommand::with_name("inspect").about("View the defined tasks and their processes");

    let app = App::new("hell")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(init)
        .subcommand(install)
        .subcommand(uninstall)
        .subcommand(run)
        .subcommand(inspect)
        .setting(AppSettings::ArgRequiredElseHelp);

    let matches = app.get_matches();

    let environment = Environment {
        manifest_filename: "Hellfile.py".to_owned(),
        hellbox_package: "git+git://github.com/hellboxpy/hellbox.git#egg=hellbox".to_owned(),
    };

    let result = dispatch(environment, matches.subcommand());

    match result {
        Ok(m) => eprintln!("{}", m),
        Err(m) => eprintln!("{}", m),
    }
}

fn dispatch<'a>(
    environment: Environment,
    subcommand: (&str, Option<&ArgMatches<'a>>),
) -> Result<String, String> {
    match subcommand {
        ("init", Some(_)) => handle_init(environment),
        ("install", Some(matches)) => handle_install(environment, matches),
        ("uninstall", Some(matches)) => handle_uninstall(environment, matches),
        ("run", Some(matches)) => handle_run(environment, matches),
        ("inspect", Some(_)) => handle_inspect(environment),
        (_, _) => Err("Subcommand not provided".to_owned()),
    }
}

// Handlers

fn handle_init<'a>(environment: Environment) -> Result<String, String> {
    eprintln!("init will now happen");

    create_pipfile()
        .and_then({ |_| install_package(&environment.hellbox_package) })
        .and_then({ |_| create_manifest(&environment.manifest_filename) })
}

fn handle_install<'a>(
    _environment: Environment,
    matches: &ArgMatches<'a>,
) -> Result<String, String> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => install_package(&name),
        None => install_dependencies(),
    }
}

fn handle_uninstall<'a>(
    _environment: Environment,
    matches: &ArgMatches<'a>,
) -> Result<String, String> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => uninstall_package(&name),
        None => Err("a package name is required".to_owned()),
    }
}

fn handle_run<'a>(environment: Environment, matches: &ArgMatches<'a>) -> Result<String, String> {
    let name = matches.value_of("task").unwrap_or("default");

    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        // Maybe init?
        run_task(&environment.manifest_filename, name)
    }
}

fn handle_inspect<'a>(environment: Environment) -> Result<String, String> {
    if !Path::new(&environment.manifest_filename).exists() {
        Err("No manifest file exists".to_owned())
    } else {
        // Maybe init?
        run_inspect(&environment.manifest_filename)
    }
}

// Actions

fn create_pipfile<'a>() -> Result<String, String> {
    let output = run_command("pipenv", vec!["--three"]);

    match output {
        Ok(_) => Ok("done".to_owned()),
        Err(_) => Err("oh no".to_owned()),
    }
}

fn create_manifest<'a>(filepath: &str) -> Result<String, String> {
    if !Path::new(filepath).exists() {
        let mut file = File::create(filepath).expect("file wasn't created");
        file.write_all(b"from hellbox import Hellbox\n\nHellbox.autoimport()")
            .expect("file wasn't written");
        Ok("done".to_owned())
    } else {
        Ok("nothing to do".to_owned())
    }
}

fn install_dependencies<'a>() -> Result<String, String> {
    let output = run_command("pipenv", vec!["install"]);

    match output {
        Ok(_) => Ok("done".to_owned()),
        Err(_) => Err("oh no".to_owned()),
    }
}

fn install_package<'a>(name: &str) -> Result<String, String> {
    let output = run_command("pipenv", vec!["install", name]);

    match output {
        Ok(_) => Ok("done".to_owned()),
        Err(_) => Err("oh no".to_owned()),
    }
}

fn run_command<'a>(command: &str, arguments: Vec<&'a str>) -> Result<String, String> {
    let output = Command::new(command)
        .args(arguments)
        .output()
        .expect("Command failed to run");

    String::from_utf8(output.stdout).map_err(|_| "Output failed to be encoded".to_owned())
}

fn run_hellbox_commands<'a>(filepath: &str, commands: Vec<&str>) -> Result<String, String> {
    let program = format!(
        "exec(open(\"{}\").read()); import hellbox; {}",
        filepath,
        commands.join("; ")
    );

    let output = run_command("pipenv", vec!["run", "python", "-c", &program]);

    match output {
        Ok(m) => Ok(m),
        Err(_) => Err("oh no".to_owned()),
    }
}

fn run_inspect<'a>(filepath: &str) -> Result<String, String> {
    run_hellbox_commands(filepath, vec!["hellbox.Hellbox.inspect()"])
}

fn run_task<'a>(filepath: &str, name: &str) -> Result<String, String> {
    run_hellbox_commands(
        filepath,
        vec![&format!("hellbox.Hellbox.run_task(\"{}\")", name)],
    )
}

fn uninstall_package<'a>(name: &str) -> Result<String, String> {
    let output = run_command("pipenv", vec!["uninstall", name]);

    match output {
        Ok(_) => Ok("done".to_owned()),
        Err(_) => Err("oh no".to_owned()),
    }
}
