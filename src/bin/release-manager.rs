#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate manager_lib;

use clap::{Arg, ArgGroup, App, AppSettings};

use manager_lib::commands::versions::{process_version_command, version_clap};
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
        .subcommand(version_clap())
        .get_matches();

    configure_logging(matches.occurrences_of("verbose") as i32, matches.is_present("quite"));
    

    let code: i32 = match matches.subcommand() {
        ("version",  Some(sub_m)) => {process_version_command(sub_m)}, // clone was used
        _ => { 
            error!("No command avalibe"); 
            -1
        }
    };

    ::std::process::exit(code);
}