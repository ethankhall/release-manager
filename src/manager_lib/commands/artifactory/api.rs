use std::boxed::Box;
use std::time::{SystemTime, UNIX_EPOCH};

use url::Url;
use hyper::{Method, Request, StatusCode};
use hyper::Uri as HyperUri;

use super::super::super::errors::ErrorCodes;
use super::super::super::http::{self, DefaultHttpRequester, HttpRequester};

#[derive(Debug)]
pub enum ArtifactoryError {
    UnableToMakeURI,
    CommunicationError(ErrorCodes),
    UnableToCreateRelease(StatusCode)
}

pub trait Artifactory {
    fn upload_artifacts(&self, tar_bytes: Vec<u8>) -> Result<(), ArtifactoryError>;
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
            requester: Box::new(DefaultHttpRequester::new())
        };
    }

    fn build_base_url<S: AsRef<str>>(&self, args: Vec<S>) -> Result<HyperUri, ArtifactoryError> {
        let mut url = Url::parse(&self.artifactory_base).expect("Unable to get base Artifactory Url");
        {
            let mut path = url.path_segments_mut().expect("Cannot get path");
            path.extend(args);
        }

        let url = s!(url.as_str());

        return match url.parse::<HyperUri>() {
            Ok(v) => Ok(v),
            Err(_) => {
                return Err(ArtifactoryError::UnableToMakeURI);
            }
        };
    }
}

impl Artifactory for ArtifactoryImpl {
    fn upload_artifacts(&self, tar_bytes: Vec<u8>) -> Result<(), ArtifactoryError> {
        let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let artifact_name = format!("temp-artifact-{}.tar", time_since_epoch);
        let uri = self.build_base_url(vec![&self.repo, &artifact_name]).expect("To get a base url");
        let mut request = Request::new(Method::Put, uri);
        request.set_body(tar_bytes);
        http::set_default_headers(
            request.headers_mut(),
            None,
            None
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
}