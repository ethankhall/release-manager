use std::env;
use std::path::{Path, PathBuf};
use std::vec::Vec;
use std::convert::Into;

use serialize::hex::ToHex;
use json::{JsonValue, Null};

use git2::Repository as GitRepository;
use git2::{Commit, ObjectType, Oid};
use super::errors::*;

pub(crate) fn find_last_commit(root_path: PathBuf) -> Result<String, ErrorCodes> {
    let repo = match GitRepository::discover(root_path) {
        Ok(repo) => repo,
        Err(err) => {
            error!("Unable to find git project: {}", err.message());
            return Err(ErrorCodes::NoRepoFound);
        }
    };

    let obj = match repo.head()
        .and_then(|x| x.resolve())
        .and_then(|x| x.peel(ObjectType::Commit))
    {
        Ok(obj) => obj,
        Err(err) => {
            error!("Unable to get HEAD from repo");
            return Err(ErrorCodes::UnableToGetHeadSha);
        }
    };

    return obj.into_commit()
        .map_err(|_| ErrorCodes::UnableToGetHeadSha)
        .map(|x| {
            let strs: Vec<String> = x.id()
                .as_bytes()
                .to_vec()
                .iter()
                .map(|x| format!("{:02x}", x))
                .collect();
            strs.join("")
        });
}
