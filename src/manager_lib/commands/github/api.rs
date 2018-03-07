use std::vec::Vec;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::boxed::Box;

use hyper::{Request, Method, StatusCode};
use hyper::Uri as HyperUri;
use semver::Version;
use url::Url;
use json::{self, parse,JsonValue};
use clap::ArgMatches;
use super::super::super::http::{self, HttpRequester, DefaultHttpRequester};

use super::super::cli_shared;

pub(crate) struct GitHubImpl {
    api_token: String,
    github_api: String,
    project_name: String,
    repo_name: String,
    requester: Box<HttpRequester>,
}

#[derive(Debug)]
pub enum GitHubError {
    FilesDoesNotExist(Vec<String>),
    UnableToCreateRelease(StatusCode),
    UnableToCreateTree,
    CommunicationError,
    UnableToParseResponse,
    UnableToMakeURI,
    UnableToUpdateReference,
}

pub trait GitHub {
    fn create_release(
        &self,
        id: String,
        version: Version,
        body: String,
        draft: bool,
    ) -> Result<(), GitHubError>;
    fn update_files(&self, head: String, branch_name: String, files: HashMap<String, String>) -> Result<(), GitHubError>;
    fn add_artifacts_to_release(
        &self,
        release: String,
        artifacts: BTreeMap<String, PathBuf>,
    ) -> Result<(), GitHubError>;
}

impl GitHubImpl {
    pub(crate) fn new(args: &ArgMatches) -> Result<GitHubImpl, GitHubError> {
        let path: Vec<&str> = args.values_of(cli_shared::GITHUB_PATH)
            .expect("GitHub path to be set")
            .collect();
        let (project, repo) = (path[0], path[1]);

        let github = GitHubImpl {
            api_token: args.value_of(cli_shared::GITHUB_API_TOKEN)
                .expect("GitHub API Token not provided")
                .into(),
            github_api: s!("https://api.github.com"),
            project_name: project.into(),
            repo_name: repo.into(),
            requester: Box::new(DefaultHttpRequester::new())
        };

        return Ok(github);
    }

    fn build_base_url<S: AsRef<str>>(&self, args: Vec<S>) -> Result<HyperUri, GitHubError> {
        let mut url = Url::parse(&self.github_api).expect("Unable to get base GitHub Url");
        {
            let mut path = url.path_segments_mut().expect("Cannot get path");
            path.extend(&["repos", &self.project_name, &self.repo_name]);
            path.extend(args);
        }

        let url = s!(url.as_str());

         return match url.parse::<HyperUri>() {
            Ok(v) => Ok(v),
            Err(_) => {
                return Err(GitHubError::UnableToMakeURI);
            }
        };
    }

    fn handle_network_request(&self, uri: HyperUri, method: Method, body: JsonValue) -> Result<JsonValue, GitHubError> {
        trace!("Body to send {:?} => {:?}", uri, json::stringify(body.clone()));

        let mut request = Request::new(method, uri);
        request.set_body(body.dump());
        http::set_default_headers(request.headers_mut(), Some("application/vnd.github.v3+json"), Some(self.api_token.clone()));
        return match self.requester.make_request(request) {
            Err(_) => Err(GitHubError::CommunicationError),
            Ok((status, body)) => {
                match status {
                    StatusCode::Ok => parse(&body).map_err(|_| GitHubError::UnableToParseResponse),
                    StatusCode::Created => parse(&body).map_err(|_| GitHubError::UnableToParseResponse),
                    _ => {
                        debug!("Status code was {}", status);
                        Err(GitHubError::UnableToCreateRelease(status))
                    }
                }
            }
        };
    }

    fn validate_files(artifacts: &BTreeMap<String, PathBuf>) -> Result<(), GitHubError> {
        let missing_files: Vec<String> = artifacts
            .into_iter()
            .filter(|&(_, value)| !value.exists())
            .map(|(key, _)| key.to_string())
            .collect();

        return if missing_files.is_empty() {
            Ok(())
        } else {
            Err(GitHubError::FilesDoesNotExist(missing_files))
        };
    }
}

impl GitHub for GitHubImpl {
    fn create_release(
        &self,
        id: String,
        version: Version,
        body: String,
        draft: bool,
    ) -> Result<(), GitHubError> {
        let uri = self.build_base_url(vec!["releases"])?;
        debug!("URL to post to: {}", uri);

        let release_name = format!("v{}", version.to_string());
        let body = object!{
            "tag_name" => release_name.clone(),
            "target_commitish" => id,
            "name" => release_name,
            "body" => body,
            "draft" => draft,
            "prerelease" => false
        };

        return match self.handle_network_request(uri, Method::Post, body) {
            Ok(_) => Ok(()),
            Err(x) => Err(x)
        };
    }

    fn update_files(&self, head: String, branch_name: String, files: HashMap<String, String>) -> Result<(), GitHubError> {
        let mut tree_entries: Vec<JsonValue> = vec![];

        for (name, entry) in files {
            let tree_entry = object!{
                "path" => name,
                "mode" => "100644",
                "type" => "blob",
                "content" => entry
            };

            tree_entries.push(tree_entry);
        }

        let body = object!{
            "base_tree" => head.clone(),
            "tree" => tree_entries,
        };

        let response = self.handle_network_request(self.build_base_url(vec!["git", "trees"])?, Method::Post, body)?;
        let tree_id = match response {
            JsonValue::Object(obj) => s!(obj.get("sha").unwrap().as_str().unwrap()),
            _ => return Err(GitHubError::UnableToCreateTree)
        };
        trace!("New Tree ID: {:?}", tree_id);

        let body = object! {
            "message" => "Updating the to the next version.\n[skip ci]",
            "tree" => tree_id,
            "parents" => vec![head],
            "committer" => object!{
                "name" => "Release Manager CI",
                "email" => "ci@release-manager.com"
            }
        };

        let response = self.handle_network_request(self.build_base_url(vec!["git", "commits"])?, Method::Post, body)?;

        let new_commit_id = match response {
            JsonValue::Object(obj) => s!(obj.get("sha").unwrap().as_str().unwrap()),
            _ => return Err(GitHubError::UnableToCreateTree)
        };
        trace!("New Tree ID: {:?}", new_commit_id);

        let body = object!{
            "sha" => new_commit_id
        };

        let uri = self.build_base_url(vec!["git", "refs", "heads", &branch_name])?;
        return match self.handle_network_request(uri, Method::Patch, body) {
            Ok(_) => Ok(()),
            Err(e) => {
                debug!("Unable to update Reference: {:?}", e);
                Err(GitHubError::UnableToUpdateReference)
            }
        };
    }

    fn add_artifacts_to_release(
        &self,
        _release: String,
        artifacts: BTreeMap<String, PathBuf>,
    ) -> Result<(), GitHubError> {
        match GitHubImpl::validate_files(&artifacts) {
            Ok(_) => {}
            Err(value) => {
                return Err(value);
            }
        }

        return Ok(());
    }
}
