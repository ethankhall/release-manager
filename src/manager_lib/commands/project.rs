use clap::{App, ArgMatches, AppSettings, SubCommand, Arg, ArgGroup};

use semver::Version as SemverVersion;

use super::cli_shared;
use super::super::version_manager::build_project;
use super::super::repo::{get_repo, Repo};
use super::super::errors::*;

pub fn project_clap<'a,'b>() -> App<'a, 'b> {
    let list_command = SubCommand::with_name("list")
        .about("List the versions avaliable")
        .arg(Arg::with_name("output-format")
            .long("output-format")
            .help("Output the data in different formats for further processing")
            .possible_values(&["json", "plain"])
            .default_value("plain"));

    let create_command = SubCommand::with_name("bump-version")
        .about("Bump the version for the project")
        .arg(Arg::with_name("set-version")
            .long("set-version")
            .help("Specify the version to create.")
            .takes_value(true))
        .arg(Arg::with_name("bump-major")
            .long("bump-major")
            .help("Bump the major component of the version"))
        .arg(Arg::with_name("bump-minor")
            .long("bump-minor")
            .help("Bump the minor component of the version"))
        .arg(Arg::with_name("bump-patch")
            .long("bump-patch")
            .help("Bump the patch component of the version"))
        .group(ArgGroup::with_name("version-options")
            .required(true)
            .args(&["set-version", "bump-major", "bump-minor", "bump-patch"]))
        .arg(Arg::with_name("commit")
            .long("commit")
            .short("c")
            .help("Commit the version file change"))
        .arg(cli_shared::message())
        .arg(cli_shared::message_file())
        .group(cli_shared::message_group());
    
    return App::new("project")
        .about("Operations on a project.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(list_command)
        .subcommand(create_command);
}

pub fn process_project_command(args: &ArgMatches) -> i32 {
    let response = match args.subcommand() {
        ("list",  Some(sub_m)) => { list(sub_m) },
        ("bump-version", Some(m)) => { bump_version(m) },
        _ => Err(CommandError::new(ErrorCodes::Unknown, format!("No command avaliable. {:?}", args)))
    };

    return match response {
        Ok(_) => 0,
        Err(value) => {
            error!("{}", value.message);
            value.error_code as i32
        }
    };
}

fn bump_version(args: &ArgMatches) -> Result<(), CommandError> {
    let (repo, path) = get_repo()?;

    let project = build_project(path).unwrap();
    let next_version = if let Some(ver) = args.value_of("set-version") {
        SemverVersion::parse(ver).expect("Version provided is not acceptable semver version")
    } else {
        let mut version = project.get_version();

        let (maj, min, pat) = (args.is_present("bump-major"),
                                args.is_present("bump-minor"),
                                args.is_present("bump-patch"));

        match (maj, min, pat) {
            (true, _, _) => version.increment_major(),
            (_, true, _) => version.increment_minor(),
            (_, _, true) => version.increment_patch(),
            _            => unreachable!(),
        };

        version
    };

    let next_version_string = next_version.to_string();
    info!(target: "user", "Next version will be {}", next_version_string);

    project.update_version(next_version);

    if args.is_present("commit") {
        let message_contents = cli_shared::extract_message(args, format!("Incrementing version to {}.", next_version_string));

        repo.commit_files(project.get_version_files(), message_contents);
    }

    return Ok(());
}

fn list(args: &ArgMatches) -> Result<(), CommandError> {
    let (repo, _) = get_repo()?;

    let versions = repo.find_versions();

    let format = match args.value_of("output-format") {
        Some(value) => value,
        _ => "plain"
    };

    match format {
        "json" => {
            let result = object! {
                "results" => versions
            };
            println!("{}", result.dump());
        },
        "plain" | _ => {
            versions.into_iter().for_each(|x| { 
                if x.message.is_some() {
                    let mut header = format!("{}", x.message.unwrap());
                    header.truncate(25);
                    println!("{} - {} - {}", x.name, x.id, header.trim());
                } else {
                    println!("{} - {}", x.name, x.id);
                }
            });
        }
    }

    return Ok(());
}