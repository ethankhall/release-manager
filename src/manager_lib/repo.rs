use std::env;
use std::path::{Path, PathBuf};
use std::vec::Vec;
use std::convert::Into;

use serialize::hex::ToHex;
use json::{JsonValue, Null};

use git2::Repository as GitRepository;
use git2::{Commit, ObjectType, Oid};
use git2::Error as GitError;

use super::errors::*;

pub(crate) fn get_repo() -> Result<(DefaultRepo, PathBuf), CommandError> {
    let pwd = env::current_dir().unwrap();
    let path = pwd.as_path();
    return match DefaultRepo::new(path) {
        Some(v) => Ok((v, path.to_path_buf())),
        None => {
            return Err(CommandError::new(
                ErrorCodes::NoRepoFound,
                format!("Unable to find repo at {:?}", path),
            ))
        }
    };
}

pub(crate) struct Version {
    pub(crate) id: String,
    pub(crate) message: Option<String>,
    pub(crate) name: String,
}

impl Into<JsonValue> for Version {
    fn into(self) -> JsonValue {
        let message = match self.message {
            Some(value) => JsonValue::String(value.trim().to_string()),
            _ => Null,
        };
        return object! {
            "id" => self.id,
            "message" => message,
            "name" => self.name
        };
    }
}

pub(crate) trait Repo {
    fn find_versions(&self) -> Vec<Version>;
    fn get_head(&self) -> String;
    fn commit_files(&self, paths: Vec<PathBuf>, message: String);
}

pub(crate) struct DefaultRepo {
    repo: GitRepository,
    root_path: PathBuf,
}

impl DefaultRepo {
    pub(crate) fn new(root_path: &Path) -> Option<DefaultRepo> {
        let repo = match GitRepository::discover(root_path) {
            Ok(repo) => repo,
            Err(err) => {
                error!("Unable to find git project: {}", err.message());
                return None;
            }
        };

        return Some(DefaultRepo {
            repo: repo,
            root_path: root_path.to_path_buf(),
        });
    }

    fn find_last_commit(&self) -> Result<Commit, GitError> {
        let obj = self.repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        return obj.into_commit()
            .map_err(|_| GitError::from_str("Couldn't find commit"));
    }

    fn add_files_to_index(&self, paths: &Vec<PathBuf>) -> Result<Oid, GitError> {
        let mut index = self.repo
            .index()
            .expect("Unable to create Index for commit");
        for path_buf in paths {
            let path = path_buf.strip_prefix(&self.root_path).unwrap();
            trace!("Adding file {:?} to repo", path);
            index.add_path(path).expect("Unable to add file to index");
        }
        index.write().expect("Unable to write git index to disk.");
        return index.write_tree();
    }
}

impl Repo for DefaultRepo {
    fn find_versions(&self) -> Vec<Version> {
        let tag_names = self.repo.tag_names(None).unwrap();

        let tag_and_refs = tag_names.iter().flat_map(|name| name).flat_map(|name| {
            let full_tag = format!("refs/tags/{}", name);
            self.repo
                .find_reference(&full_tag)
                .map(|reference| (name, reference))
        });

        let mut result: Vec<Version> = Vec::new();

        for (name, reference) in tag_and_refs {
            let hash = reference.target().unwrap().as_bytes().to_hex();

            let message = match self.repo.find_tag(reference.target().unwrap()) {
                Ok(val) => {
                    let v = val.message().map(|x| format!("{}", x)).unwrap();
                    Some(v)
                }
                Err(_) => None,
            };

            let version = Version {
                id: hash,
                message: message,
                name: s!(name),
            };

            result.push(version);
        }

        return result;
    }

    fn get_head(&self) -> String {
        return self.repo
            .head()
            .and_then(|x| x.peel_to_commit())
            .and_then(|x| Ok(x.id()))
            .expect("Unable to get current commit")
            .as_bytes()
            .to_hex();
    }

    fn commit_files(&self, paths: Vec<PathBuf>, message: String) {
        let sig = &self.repo.signature().unwrap();
        let tree = self.add_files_to_index(&paths)
            .expect("Unable to build tree");

        let parent_id = self.find_last_commit()
            .expect("Unable to find latest commit");
        let tree_id = self.repo.find_tree(tree).unwrap();
        self.repo
            .commit(Some("HEAD"), sig, sig, &message, &tree_id, &[&parent_id])
            .expect("Unable to create commit for version bump.");
    }
}
