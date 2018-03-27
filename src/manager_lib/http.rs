use std::ops::Deref;
use std::boxed::Box;
use std::error::Error;

use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use hyper::{Client, Request, StatusCode};
use hyper::header::{qitem, Accept, Authorization, Headers, UserAgent};
use hyper::mime::Mime;
use futures::{future, Future, Stream};
use hyper::Error as HyperError;

use super::errors::ErrorCodes;

pub(crate) trait HttpRequester {
    fn make_request(&self, request: Request) -> Result<(StatusCode, String), ErrorCodes>;
}

pub(crate) fn set_default_headers(
    headers: &mut Headers,
    accept: Option<&str>,
    token: Option<String>,
) {
    if token.is_some() {
        headers.set(Authorization(format!("token {}", token.unwrap())));
    }

    if accept.is_some() {
        let mime: Mime = accept.unwrap().parse().unwrap();
        headers.set(Accept(vec![qitem(mime)]));
    }

    let user_agent = UserAgent::new(format!("release-manager/{}", env!("CARGO_PKG_VERSION")));
    headers.set(user_agent);
}

pub(crate) struct DefaultHttpRequester {}

impl DefaultHttpRequester {
    pub(crate) fn new() -> Self {
        return DefaultHttpRequester {};
    }

    fn make_external_parts<'a>(&self) -> (Core, Client<HttpsConnector<HttpConnector>>) {
        let core = Core::new().expect("Unable to create HTTP Core");
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

        return (core, client);
    }
}

impl HttpRequester for DefaultHttpRequester {
    fn make_request(&self, request: Request) -> Result<(StatusCode, String), ErrorCodes> {
        trace!("Request to be sent: {:?}", &request);

        let (mut core, client) = self.make_external_parts();
        let work = client.request(request).and_then(|res| {
            let status = Box::new(res.status());

            res.body()
                .fold(Vec::new(), |mut v, chunk| {
                    v.extend(&chunk[..]);
                    future::ok::<_, HyperError>(v)
                })
                .and_then(|chunks| {
                    let bdy = String::from_utf8(chunks).unwrap();
                    future::ok::<_, HyperError>((status, s!(bdy)))
                })
        });

        let (status, body) = match core.run(work) {
            Ok((status, body)) => (status, String::from(body)),
            Err(err) => {
                trace!("Request Error: {:?}", err);
                error!("Unable to make request becasue `{}`", err.description());
                return Err(ErrorCodes::NetworkCallFailed)
            }
        };

        trace!("Body from GitHub API: {}", body);

        return Ok((*status.deref(), body));
    }
}
