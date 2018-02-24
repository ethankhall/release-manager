use std::vec::Vec;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::io::Error;

use futures::{Future, Stream};
use hyper::{Client, Request, Method, StatusCode};
use hyper::client::HttpConnector;
use hyper::Uri as HyperUri;
use tokio_core::reactor::Core;
use semver::Version;
use url::Url;
use json::parse;
use clap::ArgMatches;

use super::super::cli_shared;

pub(crate) struct GitHubImpl {
    api_token: String,
    github_api: String,
    project_name: String,
    repo_name: String,
    core: Core,
}

#[derive(Debug)]
pub enum GitHubError {
    FilesDoesNotExist(Vec<String>),
    UnableToCreateRelease(StatusCode),
    UnableToCreateCore(Error),
    CommunicationError,
    UnableToMakeURI,
}

pub trait GitHub {
    fn create_release(&self, id: String, version: Version, body: String, draft: bool) -> Result<(), GitHubError>;
    fn add_artifacts_to_release(&self, release: String, artifacts: BTreeMap<String, PathBuf>) -> Result<(), GitHubError>;
}

impl GitHubImpl {
    pub(crate) fn new(args: &ArgMatches) -> Result<GitHubImpl, GitHubError> {
        let path: Vec<&str> = args.values_of(cli_shared::GITHUB_PATH).expect("GitHub path to be set").collect();;
        let (project, repo) = (path[0], path[1]);

        let core = match Core::new() {
            Ok(v) => v,
            Err(error) => {
                return Err(GitHubError::UnableToCreateCore(error));
            }
        };

        let github = GitHubImpl { 
            api_token: args.value_of(cli_shared::GITHUB_API_TOKEN).expect("GitHub API Token not provided").into(), 
            github_api: s!("https://api.github.com"),
            project_name: project.into(),
            repo_name: repo.into(),
            core: core,
        };

        return Ok(github);
    }

    fn build_base_url<S: AsRef<str>>(&self, args: Vec<S>) -> String {
        let mut url = Url::parse(&self.github_api).expect("Unable to get base GitHub Url");
        {
            let mut path = url.path_segments_mut().expect("Cannot get path");
            path.extend(&["repos", &self.project_name, &self.repo_name]);
            path.extend(args);
        }

        return s!(url.as_str());
    }

    fn validate_files(artifacts: &BTreeMap<String, PathBuf>) -> Option<GitHubError> {
        let missing_files:Vec<String> = artifacts.into_iter()
            .filter(|&(_, value)| !value.exists())
            .map(|(key, _)| key.to_string())
            .collect();

        return if missing_files.is_empty() {
            None
        } else {
            Some(GitHubError::FilesDoesNotExist(missing_files))
        };
    }

    fn make_client(&self) -> Client<HttpConnector> {
        return Client::new(&self.core.handle());
    }
}

impl GitHub for GitHubImpl {

    fn create_release(&self, id: String, version: Version, body: String, draft: bool) -> Result<(), GitHubError> {
        let client = self.make_client();

        let url = self.build_base_url(vec!["releases"]);
        debug!("URL to pose to: {}", url);

        let uri = match url.parse::<HyperUri>() {
            Ok(v) => v,
            Err(_) => {
                return Err(GitHubError::UnableToMakeURI);
            }
        };

        let release_name = format!("v{}", version.to_string());
        let body = object!{
            "tag_name" => release_name.clone(),
            "target_commitish" => id,
            "name" => release_name,
            "body" => body,
            "draft" => draft,
            "prerelease" => false
        };

        let mut request = Request::new(Method::Post, uri);
        request.set_body(body.dump());
        request.headers_mut().set_raw("Authorization", format!("token {}", self.api_token));

        let response = match client.request(request).wait() {
            Err(err) => return Err(GitHubError::CommunicationError),
            Ok(response) => response
        };

        if response.status() != StatusCode::Created {
            error!("Unable to create release!");
            debug!("Status code was {}", response.status());
            return Err(GitHubError::UnableToCreateRelease(response.status()));
        }

        let body = response.body().concat2().wait().map(|chunk| {
            let v = chunk.to_vec();
            String::from_utf8_lossy(&v).to_string()
        }).unwrap();

        trace!("Body from GitHub API: {}", body);

        let json_body = parse(&body).unwrap();
        trace!("Reponse from github: {}", json_body.dump());
        return Ok(());
    }

    fn add_artifacts_to_release(&self, release: String, artifacts: BTreeMap<String, PathBuf>) -> Result<(), GitHubError> {
        match GitHubImpl::validate_files(&artifacts) {
            None => {},
            Some(value) => {
                return Err(value);
            }
        }

        return Ok(());
    }
}