use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::slice::SliceConcatExt;
use std::ops::Deref;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use super::super::version_manager::build_project;
use super::super::errors::*;
use super::super::git;
use super::cli_shared;
use self::api::{GitHub, GitHubError, GitHubImpl};

mod api;

pub fn github_clap<'a, 'b>() -> App<'a, 'b> {
    let github_command = SubCommand::with_name("artifacts")
        .about("Add artifacts to github release")
        // .arg(cli_shared::github_user())
        .arg(cli_shared::github_token())
        .arg(cli_shared::github_path())
        .arg(Arg::with_name("file")
            .help("Files to be uploaded. Supports both `path`, and `name=path`. When name is omitted, the filename will be used.")
            .multiple(true)
            .required(true));

    let create_release = SubCommand::with_name("release-and-bump")
        .about("Tag the current branch with the version in the metadata file for the project.")
        .aliases(&["release", "rb"])
        //  .arg(cli_shared::github_user())
        .arg(cli_shared::github_token())
        .arg(cli_shared::github_path())
        .arg(Arg::with_name("draft-release")
            .long("draft")
            .help("Release in GitHub will be marked as draft"))
        .arg(cli_shared::message())
        .arg(cli_shared::message_file())
        .group(cli_shared::message_group());

    return App::new("github")
        .about("Upload artifacts to different sources.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(github_command)
        .subcommand(create_release);
}

pub fn process_github_command(args: &ArgMatches) -> i32 {
    let response = match args.subcommand() {
        ("artifacts", Some(sub_m)) => upload_github_artifacts(sub_m),
        ("release-and-bump", Some(sub_m)) => create_release(sub_m),
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

fn make_github(args: &ArgMatches) -> Result<GitHubImpl, CommandError> {
    return match GitHubImpl::new(args) {
        Err(unknown) => {
            return Err(CommandError::new(
                ErrorCodes::Unknown,
                format!("Unknown Error! {:?}", unknown),
            ))
        }
        Ok(v) => Ok(v),
    };
}

fn upload_github_artifacts(args: &ArgMatches) -> Result<(), CommandError> {
    let project = build_project(None).unwrap();

    let mut file_map: BTreeMap<String, PathBuf> = BTreeMap::new();
    let files = args.values_of("file");
    files.unwrap().for_each(|f| {
        let (key, value) = if f.contains("=") {
            let split: Vec<&str> = f.split("=").collect();
            (split[0], split[1])
        } else {
            let pathbuf = Path::new(f);
            (pathbuf.file_name().and_then(|x| x.to_str()).unwrap(), f)
        };

        let mut file_path = project.deref().project_root();
        file_path.push(value);

        debug!("File to upload: {} -> {:?}", key, file_path);
        file_map.insert(s!(key), file_path);
    });

    let github = make_github(args)?;

    let version = project.get_version();

    return match github.add_artifacts_to_release(version.to_string(), file_map) {
        Err(GitHubError::FilesDoesNotExist(files)) => Err(CommandError::new(
            ErrorCodes::FileDoesNotExist,
            format!("File(s) `{}` do not exist", files.join(", ")),
        )),
        _ => Ok(()),
    };
}

fn create_release(args: &ArgMatches) -> Result<(), CommandError> {
    let project = build_project(None).unwrap();
    let mut version = project.get_version();

    let message_contents =
        cli_shared::extract_message(args, format!("Tagging version {}.", version.to_string()));

    let github = make_github(args)?;

    let head = match git::find_last_commit(project.deref().project_root()) {
        Err(err) => return Err(CommandError::new(err, "Unable to get last commit")),
        Ok(v) => v
    };

    match github.create_release(
        head.clone(),
        version.clone(),
        message_contents,
        args.is_present("draft-release"),
    ) {
        Err(v) => {
            trace!("Unable to create release! {:?}", v);
            return Err(CommandError::new(
                ErrorCodes::Unknown,
                s!("Unable to create release"),
            ));
        }
        Ok(_) => {}
    };

    version.increment_patch();
    let version_files = project.render_version_files(version);
    github.update_files(head, version_files);

    return Ok(());
}
