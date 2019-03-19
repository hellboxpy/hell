extern crate clap;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::result::Result;

#[derive(Debug, Copy, Clone)]
struct Environment<'a> {
    manifest_filename: &'a str,
    hellbox_package: &'a str,
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
        .version("1.0")
        .author("Jack Jennings <jack@standard-library.com>")
        .about("Lightweight wrapper around pipenv for running the Hellbox toolchain")
        .subcommand(init)
        .subcommand(install)
        .subcommand(uninstall)
        .subcommand(run)
        .subcommand(inspect);

    let matches = app.get_matches();

    let environment = Environment {
        manifest_filename: "Hellfile.py",
        hellbox_package: "git+git://github.com/hellboxpy/hellbox.git#egg=hellbox",
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
) -> Result<&'a str, &'a str> {
    match subcommand {
        ("init", Some(_)) => handle_init(environment),
        ("install", Some(matches)) => handle_install(environment, matches),
        ("uninstall", Some(matches)) => handle_uninstall(environment, matches),
        ("run", Some(matches)) => handle_run(environment, matches),
        ("inspect", Some(_)) => handle_inspect(environment),
        _ => Err("Subcommand not found"),
    }
}

// Handlers

fn handle_init<'a>(environment: Environment) -> Result<&'a str, &'a str> {
    eprintln!("init will now happen");

    let result = create_pipfile()
        .and_then({ |_| install_hellbox(environment) })
        .and_then({ |_| create_manifest(environment) });

    result
}

fn handle_install<'a>(
    _environment: Environment,
    matches: &ArgMatches<'a>,
) -> Result<&'a str, &'a str> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => install_package(&name),
        None => install_dependencies(),
    }
}

fn handle_uninstall<'a>(
    _environment: Environment,
    matches: &ArgMatches<'a>,
) -> Result<&'a str, &'a str> {
    eprintln!("install will now happen");

    match matches.value_of("package") {
        Some(name) => uninstall_package(&name),
        None => Err("a package name is required"),
    }
}

fn handle_run<'a>(environment: Environment, matches: &ArgMatches<'a>) -> Result<&'a str, &'a str> {
    let name = matches.value_of("package").unwrap_or("default");

    eprintln!("run will now happen: {}", name);

    if !Path::new(environment.manifest_filename).exists() {
        Err("No manifest file exists")
    } else {
        // Maybe init?
        run_task(environment, name)
    }
}

fn handle_inspect<'a>(environment: Environment) -> Result<&'a str, &'a str> {
    eprintln!("inspect will now happen");

    if !Path::new(environment.manifest_filename).exists() {
        Err("No manifest file exists")
    } else {
        // Maybe init?
        run_inspect(environment)
    }
}

// Actions

fn create_pipfile<'a>() -> Result<&'a str, &'a str> {
    let output = run_command("pipenv", vec!["--three"]);

    match output {
        Ok(_) => Ok("done"),
        Err(_) => Err("oh no"),
    }
}

fn create_manifest<'a>(environment: Environment) -> Result<&'a str, &'a str> {
    if !Path::new(&environment.manifest_filename).exists() {
        let mut file = File::create(&environment.manifest_filename).expect("file wasn't created");
        file.write_all(b"from hellbox import Hellbox\n\nHellbox.autoimport()")
            .expect("file wasn't written");
        Ok("done")
    } else {
        Ok("nothing to do")
    }
}

fn install_dependencies<'a>() -> Result<&'a str, &'a str> {
    let output = run_command("pipenv", vec!["install"]);

    match output {
        Ok(_) => Ok("done"),
        Err(_) => Err("oh no"),
    }
}

fn install_package<'a>(name: &str) -> Result<&'a str, &'a str> {
    let output = run_command("pipenv", vec!["install", name]);

    match output {
        Ok(_) => Ok("done"),
        Err(_) => Err("oh no"),
    }
}

fn install_hellbox<'a>(environment: Environment) -> Result<&'a str, &'a str> {
    install_package(&environment.hellbox_package)
}

fn run_command<'a>(command: &str, arguments: Vec<&'a str>) -> Result<String, &'a str> {
    let output = Command::new(command)
        .args(arguments)
        .output()
        .expect("Something went wrong");

    String::from_utf8(output.stdout).map_err(|_| "Something went wrong")
}

fn run_hellbox_commands<'a>(
    environment: Environment,
    commands: Vec<&str>,
) -> Result<&'a str, &'a str> {
    let program = format!(
        "\"execfile(\\\"{}\\\"); import hellbox; {}\"",
        environment.manifest_filename,
        commands.join("; ")
    );

    let output = run_command("pipenv", vec!["run", "python", "-c", &program]);

    match output {
        Ok(_) => Ok("done"),
        Err(_) => Err("oh no"),
    }
}

fn run_inspect<'a>(environment: Environment) -> Result<&'a str, &'a str> {
    run_hellbox_commands(environment, vec!["hellbox.Hellbox.inspect()"])
}

fn run_task<'a>(environment: Environment, name: &str) -> Result<&'a str, &'a str> {
    run_hellbox_commands(
        environment,
        vec![&format!("hellbox.Hellbox.run_task(\\\"{}\\\")", name)],
    )
}

fn uninstall_package<'a>(name: &str) -> Result<&'a str, &'a str> {
    let output = run_command("pipenv", vec!["uninstall", name]);

    match output {
        Ok(_) => Ok("done"),
        Err(_) => Err("oh no"),
    }
}
