use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::ffi::OsString;
use std::env;
use std::collections::HashMap;

use semver::Version;
use toml;
use toml_edit::{value, Document};

use file::{read_file, write_file};

pub(crate) fn build_project(path: Option<PathBuf>) -> Option<Arc<Project>> {
    let path = match path {
        None => env::current_dir().unwrap(),
        Some(x) => x,
    };

    trace!("Starting search from {:?}", path);

    loop {
        let files = path.read_dir().unwrap();
        for file in files {
            let file_path = file.unwrap();
            if file_path.file_name() == OsString::from("Cargo.toml") {
                let path = file_path.path();
                return Some(Arc::new(CargoProject::new(path.parent().unwrap())));
            }
        }

        if path.parent().is_none() {
            break;
        }
    }

    return None;
}

pub(crate) trait Project {
    fn project_root(&self) -> PathBuf;
    fn get_version(&self) -> Version;
    fn update_version(&self, Version);
    fn render_version_files(&self, Version) -> HashMap<String, String>;
    fn get_version_files(&self) -> Vec<PathBuf>;
}

struct CargoProject {
    project_root: String,
}

impl CargoProject {
    fn new(path: &Path) -> CargoProject {
        debug!("Project path: {:?}", path);
        return CargoProject {
            project_root: s!(path.to_str().unwrap()),
        };
    }

    fn get_cargo_file(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(self.project_root.clone());
        path_buf.push("Cargo.toml");

        trace!("Parsing Cargo.toml located at {:?}", path_buf);

        return path_buf;
    }
}

impl Project for CargoProject {
    fn project_root(&self) -> PathBuf {
        return PathBuf::from(self.project_root.clone());
    }

    fn get_version_files(&self) -> Vec<PathBuf> {
        return vec![self.get_cargo_file()];
    }

    fn get_version(&self) -> Version {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let parsed: toml::value::Value = toml::from_str(&read_file(cargo_path)).unwrap();

        let version = match parsed.get("package").and_then(|x| x.get("version")) {
            Some(value) => value.as_str().unwrap(),
            None => panic!("Unable to get version for cargo.toml"),
        };

        debug!("Current project version: {}", version);

        return Version::parse(version).unwrap();
    }

    fn update_version(&self, version: Version) {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let text = read_file(cargo_path);
        let mut doc = text.parse::<Document>().expect("invalid doc");
        doc["package"]["version"] = value(version.to_string());

        write_file(doc.to_string(), cargo_path);
    }

    fn render_version_files(&self, version: Version) -> HashMap<String, String> {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let text = read_file(cargo_path);
        let mut doc = text.parse::<Document>().expect("invalid doc");
        doc["package"]["version"] = value(version.to_string());

        let mut map = HashMap::new();
        map.insert(s!("Cargo.toml"), doc.to_string());

        return map;
    }
}
