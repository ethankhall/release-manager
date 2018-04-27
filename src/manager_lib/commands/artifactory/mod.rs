use std::path::{MAIN_SEPARATOR, Path};
use std::fs::File;

use tar::Builder;
use glob::glob;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use self::api::{ArtifactoryImpl, Artifactory};
use super::cli_shared;
use super::super::file::write_file_as_bytes;
use super::super::version_manager::build_project;
use super::super::config::{Config, ArtifactoryConfig};
use super::super::errors::*;

mod api;

pub fn artifactory_clap<'a, 'b>() -> App<'a, 'b> {
    let publish = SubCommand::with_name("publish")
        .setting(AppSettings::ArgRequiredElseHelp)
        .about("Uploads a directory into artifactory.")
        .long_about("This command gets pointed at a directory, then wil upload that directory into artifactory.")
        .arg(
            Arg::with_name("REPO")
                .help("The base directory to publish into artifactory. Usually this would be the folder containing 'com' or 'org'")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("version")
                .long("version-override")
                .help("Override the version that release-manager believes should be published")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Writes any intermediate files into the current workind directory.")
        )
        .arg(cli_shared::artifactory_token());

    let distribute =
        SubCommand::with_name("distribute")
            .about("Pushes the current version in Artifactory to Bintray.")
            .arg(cli_shared::artifactory_token());

    return App::new("artifactory")
        .about("Artifactory project operations.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(publish)
        .subcommand(distribute);
}

pub fn process_artifactory_command(args: &ArgMatches, config: &Config) -> i32 {
    let response = match args.subcommand() {
        ("publish", Some(m)) => upload_artifacts(m, config),
        ("distribute", Some(m)) => distribute_artifacts(m),
        _ => Err(CommandError::new(
            ErrorCodes::Unknown,
            format!("No command available. {:?}", args),
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

fn upload_artifacts(args: &ArgMatches, config: &Config) -> Result<(), CommandError> {
    let repo_string = args.value_of("REPO").unwrap();
    let repo_path = Path::new(repo_string);
    if !Path::exists(repo_path) && repo_path.is_dir() {
        trace!("Path `{}` does not exists or is not directory", repo_string);
        return Err(CommandError::new(ErrorCodes::RepoNotValid,
                                     format!("Path `{}` does not exists or is not directory", repo_string)));
    }

    let artifactory_configs = match config.artifactory {
        Some(ref a) => a,
        None => {
            return Err(CommandError::new(ErrorCodes::ArtifactorySectionDoesNotExist,
                                         "Artifactory section of config was missing"));
        }
    };

    let version = match args.value_of("version") {
        Some(x) => x.to_string(),
        None => build_project(None).unwrap().get_version().to_string().clone()
    };


    let tar_bytes = create_tar(repo_string, version, repo_path, artifactory_configs);

    if args.is_present("debug") {
        write_file_as_bytes(tar_bytes.as_slice(), Path::new("upload.tar"));
    }

    let artifactory_api = ArtifactoryImpl::new(
        &artifactory_configs.server,
        &args.value_of(cli_shared::ARTIFACTORY_API_TOKEN).unwrap().to_owned(),
        &artifactory_configs.repo);

    return match artifactory_api.upload_artifacts(tar_bytes) {
        Ok(_) => Ok(()),
        Err(err) => {
            Err(CommandError::new(ErrorCodes::ArtifactoryCommunicationFailed, format!("{:?}", err)))
        }
    };
}

fn create_tar(repo_string: &str, version: String, repo_path: &Path, artifactory_configs: &ArtifactoryConfig) -> Vec<u8> {
    let maven_prefix = artifactory_configs.group.replace(".", &MAIN_SEPARATOR.to_string());
    let mut ar = Builder::new(Vec::new());
    let glob_pattern = format!("{root}/{group}/*/{version}/*", root = repo_string, group = maven_prefix, version = version);

    trace!("Repo Glob is {}", glob_pattern);

    let ignored_files = vec!["maven-metadata-local.xml", "maven-metadata.xml"];

    for path in glob(&glob_pattern).unwrap().filter_map(Result::ok) {
        let relative_path = path.as_path().strip_prefix(repo_path).unwrap();
        let file = &mut File::open(path.clone()).unwrap();
        let path_in_arc = format!("{}", relative_path.display());

        if !ignored_files.contains(&relative_path.file_name().unwrap().to_str().unwrap())  {
            debug!("Adding {:?}=={:?} to archive", path_in_arc, path);
            ar.append_file(path_in_arc, file).expect("file to be added");
        } else {
            info!("Ignoring {:?} as it is metadata that should be managed server side", relative_path)
        }
    }
    return ar.into_inner().unwrap();
}

fn distribute_artifacts(_args: &ArgMatches) -> Result<(), CommandError> {
    return Ok(());
}
