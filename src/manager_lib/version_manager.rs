use std::path::{Path, PathBuf};
use std::boxed::Box;
use std::ffi::OsString;

use semver::Version;
use toml;
use toml_edit::{Document, value};

use file::{read_file, write_file};

pub(crate) fn build_project(path: &Path) -> Option<Box<Project>> {
    let mut path = path;
    let mut at_root = false;

    while !at_root {
        let files = path.read_dir().unwrap();
        for file in files {
            let file_path = file.unwrap();
            if file_path.file_name() == OsString::from("Cargo.toml") {
                let path = file_path.path();
                return Some(Box::new(CargoProject::new(path)));
            }
        }

        match path.clone().parent() {
            Some(parent_path) => path = parent_path,
            None => at_root = true,
        }
    }

    return None;
}

pub(crate) trait Project {
    fn get_version(&self) -> Version;
    fn update_version(&self, Version);
}

struct CargoProject {
    project_root: String
}

impl CargoProject {
    fn new(path: PathBuf) -> CargoProject {
        return CargoProject {
            project_root: s!(path.to_str().unwrap())
        };
    }
}

impl Project for CargoProject {
    fn get_version(&self) -> Version {
        let mut path_buf = PathBuf::from(self.project_root.clone());
        path_buf.push("Cargo.toml");

        let parsed: toml::value::Value = toml::from_str(&read_file(path_buf.as_path())).unwrap();

        let version = match parsed.get("package").and_then(|x| x.get("version")) {
            Some(value) => value.as_str().unwrap(),
            None => panic!("Unable to get version for cargo.toml")
        };

        return Version::parse(version).unwrap();
    }
    
    fn update_version(&self, version: Version) {
        let mut path_buf = PathBuf::from(self.project_root.clone());
        path_buf.push("Cargo.toml");

        let text = read_file(path_buf.as_path());
        let mut doc = text.parse::<Document>().expect("invalid doc");
        doc["package"]["version"] = value(version.to_string());

        write_file(doc.to_string(), path_buf.as_path());
    }
}