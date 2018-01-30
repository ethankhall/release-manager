use futures::{Future, Stream};
use hyper::{Client};
use tokio_core::reactor::Core;

use super::{ToCromPath};

#[derive(Debug, Clone)]
pub struct CromAuth {
    pub token: String
}

#[derive(Debug, Clone)]
pub struct CromServer {
    pub base_path: String
}

#[derive(Debug, Clone)]
pub struct CromApi {
    pub server: CromServer,
    pub auth: Option<CromAuth>
}

#[derive(Debug, Clone)]
pub enum CromRequestResponse {
    InternalError(String),
    UserError(String),
    Ok(String)
}

impl CromApi {

    pub fn new(base_path: String, auth_token: Option<String>) -> CromApi {
        return CromApi { server: CromServer { base_path: base_path }, auth: auth_token.and_then(|x| Some(CromAuth { token: x})) };
    }

    pub fn do_get(&self, path: &ToCromPath, append: Vec<&str>) -> CromRequestResponse {
        let path_url = path.to_crom_path();
        let mut url = format!("{basePath}{path}", basePath=self.server.base_path, path=path_url);
        if !append.is_empty() {
            let append_string = append.join("/");
            url = format!("{}/{}", url, append_string);
        }
        
        debug!("Making request for {}", url);

        let mut core = match Core::new() {
            Err(err) => return CromRequestResponse::InternalError(format!("Error: {:?}", err)),
            Ok(ok) => ok
        };

        let client = Client::new(&core.handle());

        let uri = match url.parse() {
            Ok(ok) => ok,
            Err(err) => return CromRequestResponse::InternalError(format!("Error: {:?}", err))
        };

        let work = client.get(uri).map_err(|_err| ()).and_then(|resp| {
            resp.body().concat2().map_err(|_err| ()).map(|chunk| {
                let v = chunk.to_vec();
                String::from_utf8_lossy(&v).to_string()
            })
        });

        return match core.run(work) {
            Ok(ok) => CromRequestResponse::Ok(ok),
            Err(err) => CromRequestResponse::InternalError(format!("Error Running Execution against {}: {:?}", path.friendly_name(), err)),
        };


        // let work = client.get(uri).and_then(|res| {
        //     let status = res.status();
        //     if status == StatusCode::NotFound {
        //         return CromRequestResponse::UserError(format!("Unable to find {}", path.friendly_name()));
        //     } else if status.is_client_error() {
        //         return CromRequestResponse::UserError(format!("User error when trying to access {}", path.friendly_name()));
        //     } else if status.is_server_error() {
        //         return CromRequestResponse::UserError(format!("Server error when trying to access {}", path.friendly_name()));
        //     };

        //     debug!("Response: {}", res.status());

        //     let mut body: Vec<u8 > = Vec::new();

        //     // res.body().for_each(|mut chunk| {
        //     //     body.extend(&chunk[..]);
        //     // });
        //     println!("{:?}", res.body());

        //     return CromRequestResponse::Ok(String::from_utf8(body).unwrap());
        // });

        // return match core.run(work) {
        //     Ok(ok) => ok,
        //     Err(err) => CromRequestResponse::InternalError(format!("Error Running Execution against {}: {:?}", path.friendly_name(), err)),
        // };
    }
}