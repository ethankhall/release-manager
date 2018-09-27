#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate manager_lib;
extern crate openssl_probe;

use clap::{App, AppSettings, Arg, ArgGroup};
use std::env;
use std::path::PathBuf;

use manager_lib::commands::artifactory::{artifactory_clap, process_artifactory_command};
use manager_lib::commands::github::{github_clap, process_github_command};
use manager_lib::commands::local::{process_project_command, project_clap};
use manager_lib::config::parse_toml;
use manager_lib::logging::configure_logging;

fn main() {
    let matches = App::new("release-manager")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .multiple(true)
                .help("Enables more verbose output")
                .global(true),
        )
        .arg(
            Arg::with_name("quite")
                .long("quite")
                .short("q")
                .help("Only error output will be displayed")
                .global(true),
        )
        .group(ArgGroup::with_name("logging").args(&["verbose", "quite"]))
        .subcommand(project_clap())
        .subcommand(github_clap())
        .subcommand(artifactory_clap())
        .get_matches();

    configure_logging(
        matches.occurrences_of("verbose") as i32,
        matches.is_present("quite"),
    );

    init_openssl();

    let config_file = match search_up_for_config_files() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("{}", err);
            ::std::process::exit(1);
        }
    };

    let config = parse_toml(&config_file);

    trace!("The parsed configs are: {:?}", config);

    let project_root = match config_file.parent() {
        Some(parent) => parent,
        None => {
            error!("[BUG] Unable to get parent");
            ::std::process::exit(-2);
        }
    };

    let code: i32 = match matches.subcommand() {
        ("local", Some(sub_m)) => process_project_command(sub_m, &config),
        ("github", Some(sub_m)) => process_github_command(sub_m, &config, &project_root),
        ("artifactory", Some(sub_m)) => process_artifactory_command(sub_m, &config),
        _ => {
            error!("No command avaliable");
            -1
        }
    };

    ::std::process::exit(code);
}

#[cfg(target_family = "unix")]
fn init_openssl() {
    openssl_probe::init_ssl_cert_env_vars();
}

#[cfg(target_family = "windows")]
fn init_openssl() {
    openssl_probe::init_ssl_cert_env_vars();
}

fn search_up_for_config_files() -> Result<PathBuf, String> {
    let current_dir: PathBuf = env::current_dir().unwrap();

    let mut path = current_dir.clone();
    let mut at_root = false;

    while !at_root {
        if let Some(config) = config_file(path.clone()) {
            return Ok(config);
        }

        match path.clone().parent() {
            Some(parent_path) => path = parent_path.to_path_buf(),
            None => at_root = true,
        }
    }

    return Err(s!("No Config Found!"));
}

fn config_file(path: PathBuf) -> Option<PathBuf> {
    let config_search = path.join(format!(".release-manager.toml"));
    if config_search.exists() {
        return Some(config_search);
    }

    return None;
}
