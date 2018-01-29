#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate simplelog;
extern crate crom_lib;

use simplelog::*;
use clap::{Arg, App, AppSettings};

use crom_lib::commands::versions::{process_version_command, version_clap};
use crom_lib::DEFAULT_BASE_URL;


fn main() {
    let matches = App::new("Crom")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("base_url")
            .long("base-url")
            .default_value(DEFAULT_BASE_URL))
        .subcommand(version_clap())
        .get_matches();

    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Debug, Config::default()).unwrap(),
        ]
    ).unwrap();

    let code: i32 = match matches.subcommand() {
        ("version",  Some(sub_m)) => {process_version_command(sub_m)}, // clone was used
        _ => { 
            error!("No command avalibe"); 
            -1
        }
    };

    ::std::process::exit(code);
}