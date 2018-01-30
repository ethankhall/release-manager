#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate crom_lib;

use clap::{Arg, ArgGroup, App, AppSettings};

use crom_lib::commands::versions::{process_version_command, version_clap};
use crom_lib::DEFAULT_BASE_URL;
use crom_lib::logging::configure_logging;


fn main() {
    let matches = App::new("Crom")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enables more verbose output"))
        .arg(Arg::with_name("quite")
            .long("quite")
            .short("q")
            .help("Only error output will be displayed"))
        .group(ArgGroup::with_name("logging")
            .args(&["verbose", "quite"]))
        .arg(Arg::with_name("base_url")
            .long("base-url")
            .default_value(DEFAULT_BASE_URL))
        .subcommand(version_clap())
        .get_matches();

    configure_logging(2, false);
    

    let code: i32 = match matches.subcommand() {
        ("version",  Some(sub_m)) => {process_version_command(sub_m)}, // clone was used
        _ => { 
            error!("No command avalibe"); 
            -1
        }
    };

    ::std::process::exit(code);
}