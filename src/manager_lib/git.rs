use std::path::{PathBuf};
use std::vec::Vec;

use git2::Repository as GitRepository;
use git2::{ObjectType};
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

pub(crate) fn find_branch_for_commit(root_path: PathBuf, sha: String) -> Result<String, ErrorCodes> {
    let repo = find_git_repo(root_path)?;

    let branches = repo.branches(None).expect("To get a Branches");
    for branch_result in branches {
        let (branch, _) = branch_result.expect("To get Branch");
        let branch_parts: Vec<String> = branch
            .get()
            .target()
            .expect("To get a Oid from a reference")
            .as_bytes()
            .to_vec()
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect();
        let branch_id = branch_parts.join("");

        let name = {
            s!(branch.name().expect("Branch to have a name.").expect("Branch to have a name."))
        };

        if branch_id == sha {
            return Ok(name)
        }
    }

    return Err(ErrorCodes::UnableToFindBranchNameForSha);
}