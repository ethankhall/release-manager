use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec::Vec;

use ini::Ini;
use semver::Version;
use toml;
use toml_edit::{value, Document};

use file::{read_file_to_string, write_file};

const VERSION_PROPERTIES_NAME: &'static str = "version.properties";
const CARGO_TOML_NAME: &'static str = "Cargo.toml";

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
            if file_name == OsString::from(CARGO_TOML_NAME) {
                let path = file_path.path();
                return Some(Arc::new(CargoProject::new(s!(CARGO_TOML_NAME), &path)));
            } else if file_name == OsString::from(VERSION_PROPERTIES_NAME) {
                let path = file_path.path();
                return Some(Arc::new(VersionPropertiesProject::new(s!(VERSION_PROPERTIES_NAME), &path)));
            }
        }

        if path.parent().is_none() {
            break;
        }
    }

    return None;
}

pub(crate) fn project_from_path(file_names: String) -> Option<Arc<Project>> {
    let working_dir = env::current_dir().unwrap();

    let mut file_path = working_dir.to_path_buf();
    file_path.push(file_names.clone());

    trace!("Looking for project file at {:?}", file_path);

    if let Some(file_name) = file_path.file_name() {
        if file_name == OsString::from(CARGO_TOML_NAME) {
            let path = file_path.as_path();
            return Some(Arc::new(CargoProject::new(file_names.clone(), &path)));
        } else if file_name == OsString::from(VERSION_PROPERTIES_NAME) {
            let path = file_path.as_path();
            return Some(Arc::new(VersionPropertiesProject::new(file_names.clone(), &path)));
        }
    }

    return None;
}

pub(crate) trait Project {
    fn get_version(&self) -> Version;
    fn update_version(&self, Version);
    fn render_version_files(&self, Version) -> HashMap<String, String>;
    fn get_version_files(&self) -> Vec<PathBuf>;
}

struct VersionPropertiesProject {
    project_path: String,
    version_file: String,
}

impl VersionPropertiesProject {
    fn new(project_path: String, path: &Path) -> Self {
        debug!("Project path: {:?}", path);
        return VersionPropertiesProject {
            project_path: project_path, version_file: s!(path.to_str().unwrap()),
        };
    }

    fn get_version_file(&self) -> PathBuf {
        let path_buf = PathBuf::from(self.version_file.clone());

        trace!("Using version.properties located at {:?}", self.version_file);
        return path_buf;
    }

    fn read_version_file(&self) -> String {
        let version_file = self.get_version_file();
        let version_path = version_file.as_path();
        return read_file_to_string(version_path);
    }

    fn ini_to_string(conf: Ini) -> String {
        let mut version_file_buffer = Vec::new();
        conf.write_to(&mut version_file_buffer).unwrap();

        return String::from_utf8(version_file_buffer).expect("Ini File to be UTF-8");
    }
}

impl Project for VersionPropertiesProject {
    fn get_version_files(&self) -> Vec<PathBuf> {
        return vec![self.get_version_file()];
    }

    fn get_version(&self) -> Version {
        let conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();

        let version_string = conf.section(None::<String>)
            .unwrap()
            .get("version")
            .unwrap();
        debug!("Current project version: {}", version_string);

        return Version::parse(version_string).unwrap();
    }

    fn update_version(&self, version: Version) {
        let mut conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();

        conf.with_section(None::<String>)
            .set("version", version.to_string());

        let body = VersionPropertiesProject::ini_to_string(conf);

        write_file(body, &self.get_version_file());
    }

    fn render_version_files(&self, version: Version) -> HashMap<String, String> {
        let mut conf: Ini = Ini::load_from_str(&self.read_version_file()).unwrap();
        conf.with_section(None::<String>)
            .set("version", version.to_string());
        let version_text = VersionPropertiesProject::ini_to_string(conf);

        let mut map = HashMap::new();
        map.insert(self.project_path.clone(), version_text);

        return map;
    }
}

struct CargoProject {
    project_path: String,
    version_file: String,
}

impl CargoProject {
    fn new(project_path: String, path: &Path) -> Self {
        debug!("Project path: {:?}", path);
        return CargoProject {
            project_path: project_path, version_file: s!(path.to_str().unwrap()),
        };
    }

    fn get_cargo_file(&self) -> PathBuf {
        let path_buf = PathBuf::from(self.version_file.clone());

        trace!("Using Cargo.toml located at {:?}", path_buf);
        return path_buf;
    }
}

impl Project for CargoProject {
    fn get_version_files(&self) -> Vec<PathBuf> {
        return vec![self.get_cargo_file()];
    }

    fn get_version(&self) -> Version {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let parsed: toml::value::Value = toml::from_str(&read_file_to_string(cargo_path)).unwrap();

        let version = match parsed.get("package").and_then(|x| x.get("version")) {
            Some(value) => value.as_str().unwrap(),
            None => panic!("Unable to get version for cargo.toml located at {:?}", cargo_path),
        };

        debug!("Current project version: {}", version);

        return Version::parse(version).unwrap();
    }

    fn update_version(&self, version: Version) {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let text = read_file_to_string(cargo_path);
        let mut doc = text.parse::<Document>().expect("invalid doc");
        doc["package"]["version"] = value(version.to_string());

        write_file(doc.to_string(), cargo_path);
    }

    fn render_version_files(&self, version: Version) -> HashMap<String, String> {
        let cargo_path = self.get_cargo_file();
        let cargo_path = cargo_path.as_path();

        let text = read_file_to_string(cargo_path);
        let mut doc = text.parse::<Document>().expect("invalid doc");
        doc["package"]["version"] = value(version.to_string());

        let mut map = HashMap::new();
        map.insert(self.project_path.clone(), doc.to_string());

        return map;
    }
}
