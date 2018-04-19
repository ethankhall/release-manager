use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::fs::File;
use std::vec::Vec;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::md5::Md5;
use tar::Builder;
use glob::glob;
use chrono::prelude::*;
use indicatif::ProgressBar;

use super::super::super::config::ArtifactoryConfig;
use super::super::super::file::read_file_to_bytes;

#[derive(Serialize, Debug)]
pub struct ArtifactoryModule {
    pub(crate) id: String,
    pub(crate) artifacts: Vec<ArtifactoryArtifact>,

    #[serde(skip)]
    pub(crate) publish_prefix: String,
}

#[derive(Serialize, Debug)]
pub struct ArtifactoryArtifact {
    #[serde(rename = "type")]
    pub(crate) artifact_type: String,
    pub(crate) sha1: String,
    pub(crate) md5: String,
    pub(crate) name: String,

    #[serde(skip)]
    file_path: String,
}

#[derive(Serialize, Debug)]
pub struct ArtifactoryBuild {
    pub(crate) version: String,
    pub(crate) name: String,
    pub(crate) number: String,
    pub(crate) started: String,
    pub(crate) modules: Vec<ArtifactoryModule>,
}

const IGNORED_FILES: &'static [&str] = &["maven-metadata-local.xml", "maven-metadata.xml"];

pub(crate) struct ArtifactoryDetails {
    pub(crate) build_details: ArtifactoryBuild,
    pub(crate) tar_bytes: Vec<u8>,
}

pub(crate) fn build_artifactory_details(version: &String,
                                        repo_path: &Path,
                                        artifactory_configs: &ArtifactoryConfig,
                                        name: &String,
                                        build_number: i32) -> ArtifactoryDetails {
    let maven_prefix = artifactory_configs.group.replace(".", &MAIN_SEPARATOR.to_string());

    let mut repo_path_builder = repo_path.to_path_buf();
    repo_path_builder.push(maven_prefix);

    info!("Looking in {:?} for files to be published.", repo_path_builder);
    let pattern = &format!("{}/*", &repo_path_builder.to_str().unwrap());
    let modules_in_package: Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();

    let mut modules: Vec<ArtifactoryModule> = Vec::new();

    let spinner = ProgressBar::new_spinner();

    modules_in_package.into_iter().filter(|x| x.is_dir()).for_each(|module_path| {
        if module_path.is_dir() {
            spinner.tick();

            debug!("Processing {:?} for modules", module_path);
            let mut artifact_path = module_path.to_path_buf();
            artifact_path.push(version);

            let artifacts = create_artifacts_for_path(&artifact_path.as_path(), &spinner);
            let module_name = module_path.file_name().unwrap().to_str().unwrap();

            let artifactory_module = ArtifactoryModule {
                id: format!("{}:{}:{}", artifactory_configs.group, module_name, version),
                artifacts,
                publish_prefix: format!("{}/{}/{}", artifactory_configs.group.replace(".", "/"), module_name, version)
            };

            modules.push(artifactory_module);
        } else {
            debug!("Ignoring {:?} as it is a file, not a directory", module_path);
        }
    });

    let now: DateTime<Utc> = Utc::now();

    let build = ArtifactoryBuild {
        version: s!("1.0.1"),
        name: name.to_string(),
        number: format!("{}", build_number),
        started: format!("{}", now.format("%Y-%m-%dT%H:%M:%S%.3f%z")),
        modules,
    };

    let tar_bytes = create_tar(&build);

    return ArtifactoryDetails {
        build_details: build,
        tar_bytes,
    };
}

fn create_artifacts_for_path(path_for_artifacts: &Path, spinner: &ProgressBar) -> Vec<ArtifactoryArtifact> {
    let mut artifacts: Vec<ArtifactoryArtifact> = Vec::new();

    debug!("Processing {:?} for artifacts", path_for_artifacts);

    let pattern = &format!("{}/*", &path_for_artifacts.to_str().unwrap());

    let files_for_module: Vec<PathBuf> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    for file in files_for_module {
        if !IGNORED_FILES.contains(&file.file_name().unwrap().to_str().unwrap()) {
            let file_bytes = read_file_to_bytes(&file);

            spinner.tick();
            spinner.set_message(&format!("Processing {:?}", file));

            let mut md5 = Md5::new();
            md5.input(file_bytes.as_slice());

            let mut sha1 = Sha1::new();
            sha1.input(file_bytes.as_slice());

            let artifact = ArtifactoryArtifact {
                artifact_type: file.extension().unwrap().to_str().unwrap().to_owned(),
                sha1: sha1.result_str(),
                md5: md5.result_str(),
                name: file.file_name().unwrap().to_str().unwrap().to_owned(),
                file_path: format!("{}", file.display()),
            };

            debug!("Adding artifact {:?}", artifact);
            artifacts.push(artifact);
        } else {
            debug!("Ignoring {:?} as it's an ignored file", file);
        }
    }

    return artifacts;
}

fn create_tar(artifactory_build: &ArtifactoryBuild) -> Vec<u8> {
    let mut ar = Builder::new(Vec::new());
    let modules = &artifactory_build.modules;

    modules.into_iter().for_each(|module: &ArtifactoryModule| {
        let artifacts = &module.artifacts;

        artifacts.into_iter().for_each(|artifact| {
            let file = &mut File::open(&artifact.file_path).unwrap();
            let mut buf: PathBuf = module.publish_prefix.clone().split("/").collect();
            buf.push(&artifact.name);

            let path_in_arc = format!("{}", buf.display());

            debug!("Adding {} to tar with location {}", &artifact.file_path, path_in_arc);

            ar.append_file(path_in_arc, file).expect("file to be added");
        });
    });
    return ar.into_inner().unwrap();
}