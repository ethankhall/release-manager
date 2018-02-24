#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate manager_lib;

use clap::{Arg, ArgGroup, App, AppSettings};

use manager_lib::commands::project::{process_project_command, project_clap};
use manager_lib::commands::github::{process_github_command, github_clap};
use manager_lib::logging::configure_logging;


fn main() {
    let matches = App::new("release-manager")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enables more verbose output")
            .global(true))
        .arg(Arg::with_name("quite")
            .long("quite")
            .short("q")
            .help("Only error output will be displayed")
            .global(true))
        .group(ArgGroup::with_name("logging")
            .args(&["verbose", "quite"]))
        .subcommand(project_clap())
        .subcommand(github_clap())
        .get_matches();

    configure_logging(matches.occurrences_of("verbose") as i32, matches.is_present("quite"));
    

    let code: i32 = match matches.subcommand() {
        ("project",  Some(sub_m)) => process_project_command(sub_m),
        ("github", Some(sub_m)) => process_github_command(sub_m),
        _ => { 
            error!("No command avalibe"); 
            -1
        }
    };

    ::std::process::exit(code);
}