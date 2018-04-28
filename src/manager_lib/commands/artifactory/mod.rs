use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use serde_json;

use self::api::{ArtifactoryImpl, Artifactory};
use self::builder::build_artifactory_details;
use super::cli_shared;
use super::super::file::write_file_as_bytes;
use super::super::version_manager::build_project;
use super::super::config::Config;
use super::super::errors::*;

mod api;
mod builder;

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
            Arg::with_name("build-number")
                .long("build-number")
                .help("Build number in artifactory. This needs to be unique, but can be as simple as MS since epoch")
                .takes_value(true)
                .required(true)
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
            .arg(cli_shared::artifactory_token())
            .arg(
                Arg::with_name("no-publish")
                    .long("no-publish")
                    .help("When set, version will not auto publish in Bintray")
            )
            .arg(
                Arg::with_name("debug")
                    .long("debug")
                    .short("d")
                    .help("Writes any intermediate files into the current workind directory.")
            )
            .arg(
                Arg::with_name("build-number")
                    .long("build-number")
                    .help("Build number in artifactory. This needs to be unique, but can be as simple as MS since epoch")
                    .takes_value(true)
                    .required(true)
            );


    return App::new("artifactory")
        .about("Artifactory project operations.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(publish)
        .subcommand(distribute);
}

pub fn process_artifactory_command(args: &ArgMatches, config: &Config) -> i32 {
    let response = match args.subcommand() {
        ("publish", Some(m)) => upload_artifacts(m, config),
        ("distribute", Some(m)) => distribute_artifacts(m, config),
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
    let repo_path = Path::new(args.value_of("REPO").unwrap());
    if !Path::exists(repo_path) && repo_path.is_dir() {
        trace!("Path `{:?}` does not exists or is not directory", repo_path);
        return Err(CommandError::new(ErrorCodes::RepoNotValid,
                                     format!("Path `{:?}` does not exists or is not directory", repo_path)));
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

    let build_number: i32 = args.value_of("build-number").unwrap().parse::<i32>().unwrap();

    let artifactory_details = build_artifactory_details(&version, repo_path, artifactory_configs, &config.github.repo, build_number);


    if args.is_present("debug") {
        write_file_as_bytes(artifactory_details.tar_bytes.as_slice(), Path::new("upload.tar"));
    }

    let artifactory_api = ArtifactoryImpl::new(
        &artifactory_configs.server,
        &args.value_of(cli_shared::ARTIFACTORY_API_TOKEN).unwrap().to_owned(),
        &artifactory_configs.repo);

    match artifactory_api.upload_artifacts(artifactory_details.tar_bytes) {
        Ok(_) => {}
        Err(err) => {
            return Err(CommandError::new(ErrorCodes::ArtifactoryCommunicationFailed, format!("{:?}", err)));
        }
    };

    match artifactory_api.set_properties_on_path(&artifactory_configs.repo, &artifactory_details.build_details) {
        Ok(_) => {}
        Err(err) => {
            return Err(CommandError::new(ErrorCodes::ArtifactoryCommunicationFailed, format!("{:?}", err)));
        }
    };

    let json_as_string = serde_json::to_string(&artifactory_details.build_details).unwrap();

    if args.is_present("debug") {
        debug!("JSON body to be sent to artifactory: `{}`", json_as_string);
    }

    return match artifactory_api.put_json(vec!["api", "build"], json_as_string) {
        Ok(_) => {
            warn!("Released Build {}", build_number);
            Ok(())
        }
        Err(err) => {
            Err(CommandError::new(ErrorCodes::ArtifactoryCommunicationFailed, format!("{:?}", err)))
        }
    };
}

fn distribute_artifacts(args: &ArgMatches, config: &Config) -> Result<(), CommandError> {
    let build_number: i32 = args.value_of("build-number").unwrap().parse::<i32>().unwrap();
    let publish = match !args.is_present("no-publish") {
        true => "true",
        false => "false"
    };

    let artifactory_configs = match config.artifactory {
        Some(ref a) => a,
        None => {
            return Err(CommandError::new(ErrorCodes::ArtifactorySectionDoesNotExist,
                                         "Artifactory section of config was missing"));
        }
    };

    let remote_repo = match artifactory_configs.bintray_repo.clone() {
        Some(x) => x,
        None => {
            return Err(CommandError::new(ErrorCodes::ArtifactorySectionDoesNotContainBintray,
                                         "Artifactory section was missing 'bintray-repo'"));
        }
    };

    let promote_json = json!({
        "publish": publish,
        "overrideExistingFiles": "false",
        "async": "false",
        "targetRepo": remote_repo,
        "sourceRepos": vec![artifactory_configs.repo.clone()],
        "dryRun": "false"
    });

    let artifactory_api = ArtifactoryImpl::new(
        &artifactory_configs.server,
        &args.value_of(cli_shared::ARTIFACTORY_API_TOKEN).unwrap().to_owned(),
        &artifactory_configs.repo);

    let build_number_string = format!("{}", build_number);
    let path = vec!["api", "build", "distribute", &config.github.repo, &build_number_string];

    let json_as_string = serde_json::to_string(&promote_json).unwrap();

    if args.is_present("debug") {
        debug!("JSON body to be sent to artifactory: `{}`", json_as_string);
    }

    return match artifactory_api.post_json(path, json_as_string) {
        Ok(_) => {
            warn!("Released Build {}", build_number);
            Ok(())
        }
        Err(err) => {
            Err(CommandError::new(ErrorCodes::ArtifactoryCommunicationFailed, format!("{:?}", err)))
        }
    };
}
