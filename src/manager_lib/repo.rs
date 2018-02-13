use std::path::Path;
use std::vec::Vec;
use std::convert::Into;

use serialize::hex::ToHex;
use json::{JsonValue, Null};

use git2::Repository as GitRepository;

pub(crate) struct Version {
    pub(crate) id: String,
    pub(crate) message: Option<String>,
    pub(crate) name: String
}

impl Into<JsonValue> for Version {
    fn into(self) -> JsonValue {
        let message = match self.message {
            Some(value) => JsonValue::String(value.trim().to_string()),
            _ => Null
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
}

pub(crate) struct DefaultRepo {
    repo: GitRepository
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

        return Some(DefaultRepo { repo: repo });
    }
}

impl Repo for DefaultRepo {
    fn find_versions(&self) -> Vec<Version> {
        let tag_names = self.repo.tag_names(None).unwrap();

        let tag_and_refs = tag_names
            .iter()
            .flat_map(|name| name)
            .flat_map(|name| {
                let full_tag = format!("refs/tags/{}", name);
                self.repo.find_reference(&full_tag).map(|reference| (name, reference))
            });

        let mut result: Vec<Version> = Vec::new();

        for (name, reference) in tag_and_refs {
            let hash = reference.target().unwrap().as_bytes().to_hex();

            let message = match self.repo.find_tag(reference.target().unwrap()) {
                Ok(val) => {
                    let v = val.message().map(|x| format!("{}", x)).unwrap();
                    Some(v)
                },
                Err(_) => None
            };

            let version = Version {
                id: hash,
                message: message,
                name: s!(name)
            };

            result.push(version);
        }

        return result;
    }
}