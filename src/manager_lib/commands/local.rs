use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use semver::Version as SemverVersion;
use semver::Identifier;

use super::super::version_manager::build_project;
use super::super::errors::*;
use super::super::config::Config;

pub fn project_clap<'a, 'b>() -> App<'a, 'b> {
    let create_command = SubCommand::with_name("update-version")
        .about("Bump the version for the project")
        .arg(
            Arg::with_name("at-version")
                .long("at-version")
                .help("Specify the version to create.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bump-major")
                .long("bump-major")
                .help("Bump the major component of the version"),
        )
        .arg(
            Arg::with_name("bump-minor")
                .long("bump-minor")
                .help("Bump the minor component of the version"),
        )
        .arg(
            Arg::with_name("bump-patch")
                .long("bump-patch")
                .help("Bump the patch component of the version"),
        )
        .arg(
            Arg::with_name("snapshot")
                .long("snapshot")
                .help("Update the version, to a snapshot version"),
        )
        .group(
            ArgGroup::with_name("version-options")
                .required(true)
                .args(&[
                    "at-version",
                    "bump-major",
                    "bump-minor",
                    "bump-patch",
                    "snapshot",
                ]),
        );

    let show_version = SubCommand::with_name("show-version").about("Show the current version");

    return App::new("local")
        .about("Local project operations.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_command)
        .subcommand(show_version);
}

pub fn process_project_command(args: &ArgMatches, _config: &Config) -> i32 {
    let response = match args.subcommand() {
        ("update-version", Some(m)) => update_version(m),
        ("show-version", Some(m)) => show_version(m),
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

fn show_version(_args: &ArgMatches) -> Result<(), CommandError> {
    let project = build_project(None).unwrap();
    println!("{}", project.get_version());

    return Ok(());
}

fn update_version(args: &ArgMatches) -> Result<(), CommandError> {
    let project = build_project(None).unwrap();
    let next_version = if let Some(ver) = args.value_of("at-version") {
        SemverVersion::parse(ver).expect("Version provided is not acceptable semver version")
    } else {
        let mut version = project.get_version();

        let (maj, min, pat, snapshot) = (
            args.is_present("bump-major"),
            args.is_present("bump-minor"),
            args.is_present("bump-patch"),
            args.is_present("snapshot"),
        );

        match (maj, min, pat, snapshot) {
            (true, _, _, _) => version.increment_major(),
            (_, true, _, _) => version.increment_minor(),
            (_, _, true, _) => version.increment_patch(),
            (_, _, _, true) => {
                version = SemverVersion {
                    major: version.major,
                    minor: version.minor,
                    patch: version.patch,
                    pre: vec![Identifier::AlphaNumeric(s!("SNAPSHOT"))],
                    build: vec![
                        Identifier::Numeric(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        ),
                    ],
                };
            }
            _ => unreachable!(),
        };

        version
    };

    let next_version_string = next_version.to_string();
    info!(target: "user", "Next version will be {}", next_version_string);

    project.update_version(next_version);
    return Ok(());
}
