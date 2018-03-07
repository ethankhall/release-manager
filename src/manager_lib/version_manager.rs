use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::ffi::OsString;
use std::env;
use std::collections::HashMap;
use std::vec::Vec;

use semver::Version;
use toml;
use toml_edit::{value, Document};
use ini::Ini;

use file::{read_file, write_file};

const VERSION_PROPERTIES_NAME: &'static str = "version.properties";

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
            let file_name = file_path.file_name();
            if file_name == OsString::from("Cargo.toml") {
                let path = file_path.path();
                return Some(Arc::new(CargoProject::new(path.parent().unwrap())));
            } else if file_name == OsString::from(VERSION_PROPERTIES_NAME) {
                let path = file_path.path();
                return Some(Arc::new(VersionPropertiesProject::new(path.parent().unwrap())));
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

struct VersionPropertiesProject {
    project_root: String,
}

impl VersionPropertiesProject {
    fn new(path: &Path) -> Self {
        debug!("Project path: {:?}", path);
        return VersionPropertiesProject {
            project_root: s!(path.to_str().unwrap()),
        };
    }

    fn get_version_file(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(self.project_root.clone());
        path_buf.push(VERSION_PROPERTIES_NAME);

        trace!("Using version.properties located at {:?}", path_buf);
        return path_buf;
    }

    fn read_version_file(&self) -> String {
        let version_file = self.get_version_file();
        let version_path = version_file.as_path();
        return read_file(version_path);
    }

    fn ini_to_string(conf: Ini) -> String {
        let mut version_file_buffer = Vec::new();
        conf.write_to(&mut version_file_buffer).unwrap();

        return String::from_utf8(version_file_buffer).expect("Ini File to be UTF-8");
    }
}

impl Project for VersionPropertiesProject {
    fn project_root(&self) -> PathBuf {
        return PathBuf::from(self.project_root.clone());
    }

    fn get_version_files(&self) -> Vec<PathBuf> {
        return vec![self.get_version_file()];
    }

    fn get_version(&self) -> Version {
        let conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();

        let version_string = conf.section(None::<String>).unwrap().get("version").unwrap();
        debug!("Current project version: {}", version_string);

        return Version::parse(version_string).unwrap();
    }

    fn update_version(&self, version: Version) {
        let mut conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();

        conf.with_section(None::<String>).set("version", version.to_string());

        let body = VersionPropertiesProject::ini_to_string(conf);

        write_file(body, &self.get_version_file());
    }

    fn render_version_files(&self, version: Version) -> HashMap<String, String> {
        let mut conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();
        conf.with_section(None::<String>).set("version", version.to_string());
        let version_text = VersionPropertiesProject::ini_to_string(conf);

        let mut map = HashMap::new();
        map.insert(s!(VERSION_PROPERTIES_NAME), version_text);

        return map;
    }
}

struct CargoProject {
    project_root: String,
}

impl CargoProject {
    fn new(path: &Path) -> Self {
        debug!("Project path: {:?}", path);
        return CargoProject {
            project_root: s!(path.to_str().unwrap()),
        };
    }

    fn get_cargo_file(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(self.project_root.clone());
        path_buf.push("Cargo.toml");

        trace!("Using Cargo.toml located at {:?}", path_buf);
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
