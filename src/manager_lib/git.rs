use std::path::PathBuf;
use std::vec::Vec;

use git2::Repository as GitRepository;
use git2::{ObjectType, Oid};
use super::errors::*;

fn find_git_repo(root_path: PathBuf) -> Result<GitRepository, ErrorCodes> {
    return match GitRepository::discover(root_path) {
        Ok(repo) => Ok(repo),
        Err(err) => {
            error!("Unable to find git project: {}", err.message());
            return Err(ErrorCodes::NoRepoFound);
        }
    };
}
pub(crate) fn find_last_commit(root_path: PathBuf) -> Result<String, ErrorCodes> {
    let repo = find_git_repo(root_path)?;

    let obj = match repo.head()
        .and_then(|x| x.resolve())
        .and_then(|x| x.peel(ObjectType::Commit))
    {
        Ok(obj) => obj,
        Err(err) => {
            trace!("Error getting last commit: {:?}", err);
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

pub(crate) fn find_branch_for_commit(
    root_path: PathBuf,
    sha: String,
) -> Result<String, ErrorCodes> {
    let repo = find_git_repo(root_path)?;
    let oid = Oid::from_str(&sha).unwrap();
    trace!("SHA: {}", oid);

    let branch = repo.branches(None)
        .expect("To be able to get branches")
        .find(|result| match result {
            &Ok((ref branch, _)) => branch.get().target() == Some(oid),
            _ => false,
        });

    return match branch {
        Some(Ok((ref branch, _))) => match branch.name() {
            Ok(name) => {
                let name = strip_remote(name.unwrap());
                Ok(s!(name))
            }
            Err(_) => Err(ErrorCodes::UnableToFindBranchNameForSha),
        },
        None | Some(Err(_)) => Err(ErrorCodes::UnableToFindBranchNameForSha),
    };
}

fn strip_remote(branch_name: &str) -> String {
    return s!(branch_name.split("/").last().unwrap());
}

#[test]
fn will_remove_origin() {
    assert_eq!(strip_remote("origin/master"), s!("master"));
}

#[test]
fn will_keep_raw_branch_name() {
    assert_eq!(strip_remote("master"), s!("master"));
}