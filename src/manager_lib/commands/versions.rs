use std::env;
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, AppSettings, SubCommand, Arg, ArgGroup};

use semver::Version as SemverVersion;

use super::super::version_manager::build_project;
use super::super::repo::{DefaultRepo, Repo};
use super::super::file::read_file;
use super::CommandError;

enum ErrorCodes {
    Unknown,
    UnableToPush,
    NoRepoFound
}

pub fn version_clap<'a,'b>() -> App<'a, 'b> {
    let list_command = SubCommand::with_name("list")
            .about("List the versions avaliable")
            .arg(Arg::with_name("output-format")
                .long("output-format")
                .help("Output the data in different formats for further processing")
                .possible_values(&["json", "plain"])
                .default_value("plain"));
    
    let tag_release = SubCommand::with_name("tag-release")
        .about("Tag the current branch with the version in the metadata file for the project.")
        .arg(Arg::with_name("message")
            .long("message")
            .short("m")
            .help("Give a message for the tag being created")
            .takes_value(true)
            .max_values(1)
            .min_values(0))
        .arg(Arg::with_name("message-file")
            .long("message-file")
            .help("Read message from file for the tag being created")
            .takes_value(true)
            .number_of_values(1))
        .group(ArgGroup::with_name("messages")
            .args(&["message", "message-file"]))
        .arg(Arg::with_name("push")
            .long("push")
            .help("Push to remote SCM"));

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
        .arg(Arg::with_name("message")
            .long("message")
            .short("m")
            .help("Give a message for the tag being created")
            .takes_value(true)
            .max_values(1)
            .min_values(0)
            .requires("commit"))
        .arg(Arg::with_name("message-file")
            .long("message-file")
            .help("Read message from file for the tag being created")
            .takes_value(true)
            .number_of_values(1)
            .requires("commit"))
        .group(ArgGroup::with_name("messages")
            .args(&["message", "message-file"]))
        .arg(Arg::with_name("push")
            .long("push")
            .help("Push to remote SCM")
            .requires("commit"));
    
    return App::new("version")
        .about("Operates on versions of projects.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(list_command)
        .subcommand(tag_release)
        .subcommand(create_command);
}

pub fn process_version_command(args: &ArgMatches) -> i32 {
    let response = match args.subcommand() {
        ("list",  Some(sub_m)) => { list(sub_m) },
        ("tag-release",  Some(sub_m)) => { tag_release(sub_m) },
        ("bump-version", Some(m)) => { bump_version(m) },
        _ => Err(CommandError::new(ErrorCodes::Unknown as i32, format!("No command avaliable. {:?}", args)))
    };

    return match response {
        Ok(_) => 0,
        Err(value) => {
            error!("{}", value.message);
            value.error_code
        }
    };
}

fn extract_message(args: &ArgMatches, inline: &str, file_name: &str, default: String) -> String {
    let (message, message_file) = (args.value_of(inline), args.value_of(file_name));
    let message_contents = if message_file.is_some() {
        let path = Path::new(message_file.unwrap());
        read_file(&path)
    } else if message.is_some() {
        s!(message.unwrap())
    } else {
        default
    };

    debug!("Message is {}", message_contents);

    return message_contents;
}

fn get_repo() -> Result<(DefaultRepo, PathBuf), CommandError> {
    let pwd = env::current_dir().unwrap();
    let path = pwd.as_path();
    return match DefaultRepo::new(path) {
        Some(v) => Ok((v, path.to_path_buf())),
        None => return Err(CommandError::new(ErrorCodes::NoRepoFound as i32, format!("Unable to find repo at {:?}", path)))
    };
}

fn tag_release(args: &ArgMatches) -> Result<(), CommandError> {
    let (repo, path) = get_repo()?;

    let project = build_project(path).unwrap();
    let version = project.get_version();

    let message_contents = extract_message(args, "message", "message-file", format!("Tagging version {}.", version.to_string()));

    repo.tag_version(version, message_contents);

    return Ok(());
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
        let message_contents = extract_message(args, "message", "message-file", format!("Incrementing version to {}.", next_version_string));

        repo.commit_files(project.get_version_files(), message_contents);
    }

    if args.is_present("push") {
        if !repo.update_remote() {
            return Err(CommandError::new(ErrorCodes::UnableToPush as i32, s!("Unable to push to repo")));
        }
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