use std::boxed::Box;
use std::time::{SystemTime, UNIX_EPOCH};

use url::Url;
use hyper::{Method, Request, StatusCode};
use hyper::header::ContentType;
use hyper::Uri as HyperUri;

use super::super::super::errors::ErrorCodes;
use super::super::super::http::{self, DefaultHttpRequester, HttpRequester};
use super::builder::ArtifactoryBuild;
use std::collections::HashMap;

#[derive(Debug)]
pub enum ArtifactoryError {
    UnableToMakeURI,
    CommunicationError(ErrorCodes),
    UnableToCreateRelease(StatusCode),
}

pub trait Artifactory {
    fn upload_artifacts(&self, tar_bytes: Vec<u8>) -> Result<(), ArtifactoryError>;
    fn put_json(&self, path: Vec<&str>, body: String) -> Result<(), ArtifactoryError>;
    fn post_json(&self, path: Vec<&str>, body: String) -> Result<(), ArtifactoryError>;
    fn set_properties_on_path(&self, repo_name: &String, build: &ArtifactoryBuild) -> Result<(), ArtifactoryError>;
}

pub(crate) struct ArtifactoryImpl {
    artifactory_base: String,
    api_token: String,
    repo: String,
    requester: Box<HttpRequester>,
}

impl ArtifactoryImpl {
    pub(crate) fn new(artifactory_base: &String, api_token: &String, repo: &String) -> ArtifactoryImpl {
        return ArtifactoryImpl {
            artifactory_base: artifactory_base.clone(),
            api_token: api_token.clone(),
            repo: repo.clone(),
            requester: Box::new(DefaultHttpRequester::new()),
        };
    }

    fn build_base_url<S: AsRef<str>>(&self, args: Vec<S>, properties: Option<HashMap<String, String>>) -> Result<HyperUri, ArtifactoryError> {
        let mut url = Url::parse(&self.artifactory_base).expect("Unable to get base Artifactory Url");
        {
            let mut path = url.path_segments_mut().expect("Cannot get path");
            path.extend(args);
        }

        {
            match properties {
                Some(map) => {
                    let mut query = url.query_pairs_mut();
                    for (key, value) in &map {
                        query.append_pair(key, value);
                    }
                }
                None => {}
            }
        }

        let url = s!(url.as_str());

        return match url.parse::<HyperUri>() {
            Ok(v) => Ok(v),
            Err(_) => {
                return Err(ArtifactoryError::UnableToMakeURI);
            }
        };
    }

    fn exec_send(&self, path: Vec<&str>, body: String, method: Method) -> Result<(), ArtifactoryError> {
        let uri = self.build_base_url(path, None).expect("To get a base url");
        let mut request = Request::new(method, uri);
        let bytes = format!("{}", body);
        request.set_body(bytes);
        http::set_default_headers(
            request.headers_mut(),
            None,
            None,
        );

        {
            let headers = request.headers_mut();
            headers.set_raw("X-JFrog-Art-Api", self.api_token.clone());
            headers.set(ContentType::json());
        }

        return match self.requester.make_request(request) {
            Err(err) => Err(ArtifactoryError::CommunicationError(err)),
            Ok((status, _body)) => match status {
                StatusCode::Ok => Ok(()),
                StatusCode::Created => Ok(()),
                StatusCode::NoContent => Ok(()),
                _ => {
                    debug!("Status code was {}", status);
                    Err(ArtifactoryError::UnableToCreateRelease(status))
                }
            },
        };
    }
}

impl Artifactory for ArtifactoryImpl {
    fn upload_artifacts(&self, tar_bytes: Vec<u8>) -> Result<(), ArtifactoryError> {
        let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let artifact_name = format!("temp-artifact-{}.tar", time_since_epoch);
        let uri = self.build_base_url(vec![&self.repo, &artifact_name], None).expect("To get a base url");
        let mut request = Request::new(Method::Put, uri);
        request.set_body(tar_bytes);
        http::set_default_headers(
            request.headers_mut(),
            None,
            None,
        );

        {
            let headers = request.headers_mut();
            headers.set_raw("X-Explode-Archive", "true");
            headers.set_raw("X-Explode-Archive-Atomic", "true");
            headers.set_raw("X-JFrog-Art-Api", self.api_token.clone());
        }

        return match self.requester.make_request(request) {
            Err(err) => Err(ArtifactoryError::CommunicationError(err)),
            Ok((status, _body)) => match status {
                StatusCode::Ok => Ok(()),
                StatusCode::Created => Ok(()),
                _ => {
                    debug!("Status code was {}", status);
                    Err(ArtifactoryError::UnableToCreateRelease(status))
                }
            },
        };
    }

    fn put_json(&self, path: Vec<&str>, body: String) -> Result<(), ArtifactoryError> {
        return self.exec_send(path, body, Method::Put);
    }

    fn post_json(&self, path: Vec<&str>, body: String) -> Result<(), ArtifactoryError> {
        return self.exec_send(path, body, Method::Post);
    }

    fn set_properties_on_path(&self, repo_name: &String, build: &ArtifactoryBuild) -> Result<(), ArtifactoryError> {
        // /api/storage/libs-release-local/ch/qos/logback/logback-classic/0.9.9?properties=os=win,linux;qa=done&recursive=1

        let mut properties: HashMap<String, String> = HashMap::new();
        let property_value = format!("build.number={number};build.name={name}", number = build.number, name = build.name);
        properties.insert(s!("properties"), property_value);
        properties.insert(s!("recursive"), s!("1"));

        for module in &build.modules {
            let path = vec!["api", "storage", &repo_name, &module.publish_prefix];
            let uri = self.build_base_url(path, Some(properties.clone())).expect("To get a base url");
            let mut request = Request::new(Method::Put, uri);
            {
                let headers = request.headers_mut();
                headers.set_raw("X-JFrog-Art-Api", self.api_token.clone());
            }

            match self.requester.make_request(request) {
                Err(err) => {
                    return Err(ArtifactoryError::CommunicationError(err));
                }
                Ok((status, _body)) => match status.is_success() {
                    true => {}
                    false => {
                        debug!("Status code was {}", status);
                        return Err(ArtifactoryError::UnableToCreateRelease(status));
                    }
                },
            };
        }

        return Ok(());
    }
}