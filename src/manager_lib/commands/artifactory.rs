use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use super::super::config::Config;
use super::super::errors::*;

pub fn artifacory_clap<'a, 'b>() -> App<'a, 'b> {
    let upload = SubCommand::with_name("upload")
        .about("Uploads a directory into artifactory.")
        .help("This command gets pointed at a directory, then wil upload that directory into artifactory.")
        .arg(
            Arg::with_name("REPO")
                .help("The base directory to publish into artifactory. Usually this would be the folder containing 'com' or 'org'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("build-number")
                .long("build-number")
                .help("The build number in artifactory.")
                .takes_value(true)
                .required(true)
        );

    let create_release =
        SubCommand::with_name("create-release").about("Creates a release in artifactory.");

    return App::new("artifactory")
        .about("Artifactory project operations.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(upload)
        .subcommand(create_release);
}

pub fn process_artifactory_command(args: &ArgMatches, _config: &Config) -> i32 {
    let response = match args.subcommand() {
        ("upload", Some(m)) => upload_artifacts(m),
        ("create-release", Some(m)) => create_release(m),
        _ => Err(CommandError::new(
            ErrorCodes::Unknown,
            format!("No command avaliable. {:?}", args),
        )),
    };

    return match response {
        Ok(_) => 0,
        Err(value) => {
            error!("{}", value.message);
            value.error_code as i32
        }
    };
}

fn upload_artifacts(_args: &ArgMatches) -> Result<(), CommandError> {
    return Ok(());
}

fn create_release(_args: &ArgMatches) -> Result<(), CommandError> {
    return Ok(());
}
