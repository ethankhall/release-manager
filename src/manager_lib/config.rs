use std::path::PathBuf;

use toml;

use super::file;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub github: GitHubConfig,
}

#[derive(Deserialize, Clone)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
}

pub fn parse_toml<S: Into<PathBuf>>(path: S) -> Config {
    return toml::from_str(&file::read_file_to_string(&path.into()))
        .expect("Config to be well formed TOML");
}
