use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use self::api::{GitHub, GitHubError, GitHubImpl};
use super::super::config::Config;
use super::super::errors::*;
use super::super::git;
use super::super::version_manager::{build_project, project_from_path};
use super::cli_shared;

mod api;

pub fn github_clap<'a, 'b>() -> App<'a, 'b> {
    let github_command = SubCommand::with_name("artifacts")
        .alias("artifact")
        .about("Add artifacts to github release")
        .arg(cli_shared::github_token())
        .arg(Arg::with_name("file")
            .help("Files to be uploaded. Supports both `path`, and `name=path`. When name is omitted, the filename will be used.")
            .multiple(true)
            .required(true));

    let create_release = SubCommand::with_name("release-and-bump")
        .about("Tag the current branch with the version in the metadata file for the project then bump the patch version.")
        .arg(cli_shared::github_token())
        .arg(Arg::with_name("draft-release")
            .long("draft")
            .help("Release in GitHub will be marked as draft"))
        .arg(cli_shared::message())
        .arg(cli_shared::message_file())
        .group(cli_shared::message_group());

    let release = SubCommand::with_name("release")
        .about("Tag the current branch with the version in the metadata file for the project.")
        .arg(cli_shared::github_token())
        .arg(
            Arg::with_name("draft-release")
                .long("draft")
                .help("Release in GitHub will be marked as draft"),
        )
        .arg(cli_shared::message())
        .arg(cli_shared::message_file())
        .group(cli_shared::message_group());

    let bump = SubCommand::with_name("bump")
        .about("Bump the current version on GitHub.")
        .arg(cli_shared::github_token());

    return App::new("github")
        .about("Upload artifacts to different sources.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(github_command)
        .subcommand(create_release)
        .subcommand(release)
        .subcommand(bump);
}

pub fn process_github_command(args: &ArgMatches, config: &Config, project_root: &Path) -> i32 {
    let response = match args.subcommand() {
        ("artifacts", Some(sub_m)) => upload_github_artifacts(sub_m, config, project_root),
        ("release-and-bump", Some(sub_m)) => match create_release(sub_m, config, project_root) {
            Ok(_) => bump_version(sub_m, config, project_root),
            Err(err) => Err(err),
        },
        ("release", Some(sub_m)) => create_release(sub_m, config, project_root),
        ("bump", Some(sub_m)) => bump_version(sub_m, config, project_root),
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

fn make_github(args: &ArgMatches, config: &Config) -> Result<GitHubImpl, CommandError> {
    return match GitHubImpl::new(args, config) {
        Err(unknown) => {
            return Err(CommandError::new(
                ErrorCodes::Unknown,
                format!("Unknown Error! {:?}", unknown),
            ))
        }
        Ok(v) => Ok(v),
    };
}

fn upload_github_artifacts(args: &ArgMatches, config: &Config, project_root: &Path) -> Result<(), CommandError> {
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

        let mut file_path = project_root.to_path_buf();
        file_path.push(value);

        debug!("File to upload: {} -> {:?}", key, file_path);
        file_map.insert(s!(key), file_path);
    });

    let github = make_github(args, config)?;
    let release = format!("v{}", project.get_version());

    return match github.add_artifacts_to_release(release, file_map) {
        Err(GitHubError::FilesDoesNotExist(files)) => Err(CommandError::new(
            ErrorCodes::FileDoesNotExist,
            format!("File(s) `{}` do not exist", files.join(", ")),
        )),
        _ => Ok(()),
    };
}

fn create_release(args: &ArgMatches, config: &Config, project_root: &Path) -> Result<(), CommandError> {

    let project = build_project(None).unwrap();
    let version = project.get_version();

    let message_contents =
        cli_shared::extract_message(args, format!("Tagging version {}.", version.to_string()));

    let github = make_github(args, config)?;

    let head = match git::find_last_commit(project_root.to_path_buf()) {
        Err(err) => return Err(CommandError::new(err, "Unable to get last commit")),
        Ok(v) => v,
    };

    return match github.create_release(
        head,
        version,
        message_contents,
        args.is_present("draft-release"),
    ) {
        Err(v) => {
            trace!("Unable to create release! {:?}", v);
            Err(CommandError::new(
                ErrorCodes::Unknown,
                s!("Unable to create release"),
            ))
        }
        Ok(_) => Ok(()),
    };
}

fn bump_version(args: &ArgMatches, config: &Config, project_root: &Path) -> Result<(), CommandError> {
   
    let project = match config.clone().github.verion_file {
        Some(file) => project_from_path(file),
        None => build_project(None)
    };

    let project = match project {
        Some(project) => project,
        None => return Err(CommandError::new(ErrorCodes::NoRepoFound, "Could not find project description!"))
    };

    let head = match git::find_last_commit(project_root.to_path_buf()) {
        Err(err) => return Err(CommandError::new(err, "Unable to get last commit")),
        Ok(v) => v,
    };

    let branch_name = match git::find_branch_for_commit(project_root.to_path_buf(), head.clone()) {
        Err(err) => return Err(CommandError::new(err, "Unable to get branch name")),
        Ok(v) => v,
    };

    let mut version = project.get_version();

    let github = make_github(args, config)?;
    version.increment_patch();
    let version_files = project.render_version_files(version);

    return match github.update_files(head, branch_name, version_files) {
        Ok(_) => Ok(()),
        Err(err) => {
            trace!("Unable to bump version: {:?}", err);
            Err(CommandError::new(
                ErrorCodes::UnableToBumpVersion,
                s!("Unable to bump version in GitHub"),
            ))
        }
    };
}
