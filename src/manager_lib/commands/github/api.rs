use std::vec::Vec;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::io::Error;
use std::boxed::Box;
use std::ops::Deref;

use futures::{Future, Stream, future};
use hyper::{Client, Method, Request, StatusCode};
use hyper::header::{Authorization, Accept, UserAgent, qitem};
use hyper::client::HttpConnector;
use hyper::Error as HyperError;
use hyper::Uri as HyperUri;
use tokio_core::reactor::Core;
use semver::Version;
use url::Url;
use json::parse;
use clap::ArgMatches;
use hyper_tls::HttpsConnector;
use hyper::mime::Mime;

use super::super::cli_shared;

pub(crate) struct GitHubImpl {
    api_token: String,
    github_api: String,
    project_name: String,
    repo_name: String,
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
    fn create_release(
        &self,
        id: String,
        version: Version,
        body: String,
        draft: bool,
    ) -> Result<(), GitHubError>;
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
        let missing_files: Vec<String> = artifacts
            .into_iter()
            .filter(|&(_, value)| !value.exists())
            .map(|(key, _)| key.to_string())
            .collect();

        return if missing_files.is_empty() {
            None
        } else {
            Some(GitHubError::FilesDoesNotExist(missing_files))
        };
    }

    fn make_external_parts<'a>(&self) -> Result<(Core, Client<HttpsConnector<HttpConnector>>), GitHubError> {
        let core = match Core::new() {
            Ok(v) => v,
            Err(error) => {
                return Err(GitHubError::UnableToCreateCore(error));
            }
        };        
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

        return Ok((core, client));
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
        let (mut core, client) = match self.make_external_parts() {
            Ok((core, client)) => (core, client),
            Err(err) => return Err(err)
        };

        let url = self.build_base_url(vec!["releases"]);
        debug!("URL to post to: {}", url);

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

        let mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
        let user_agent = UserAgent::new(format!("release-manager/{}", env!("CARGO_PKG_VERSION")));
        let mut request = Request::new(Method::Post, uri);
        request.set_body(body.dump());
        request.headers_mut().set(Authorization(format!("token {}", self.api_token)));
        request.headers_mut().set(Accept(vec![qitem(mime)]));
        request.headers_mut().set(user_agent);

        trace!("Request to be sent: {:?}", &request);

        let work = client.request(request).and_then(|res| {
            let status = Box::new(res.status());

            res.body().fold(Vec::new(), |mut v, chunk| {
                v.extend(&chunk[..]);
                future::ok::<_, HyperError>(v)
            }).and_then(|chunks| {
                let bdy = String::from_utf8(chunks).unwrap();
                future::ok::<_, HyperError>((status, s!(bdy)))
            })
        });

        let (status, body) = match core.run(work) {
            Ok((status, body)) => (status, String::from(body)),
            Err(err) => {
                trace!("Request Error: {:?}", err);
                return Err(GitHubError::CommunicationError)
            }
        };
        
        trace!("Body from GitHub API: {}", body);

        let status = *status.deref();
        if status != StatusCode::Created {
            debug!("Status code was {}", status);
            return Err(GitHubError::UnableToCreateRelease(status));
        }

        let json_body = parse(&body).unwrap();
        trace!("Reponse from github: {}", json_body.dump());
        return Ok(());
    }

    fn add_artifacts_to_release(
        &self,
        release: String,
        artifacts: BTreeMap<String, PathBuf>,
    ) -> Result<(), GitHubError> {
        match GitHubImpl::validate_files(&artifacts) {
            None => {}
            Some(value) => {
                return Err(value);
            }
        }

        return Ok(());
    }
}
