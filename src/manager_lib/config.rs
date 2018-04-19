use std::path::PathBuf;

use toml;

use super::file;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub github: GitHubConfig,
    pub artifactory: Option<ArtifactoryConfig>
}

#[derive(Deserialize, Clone)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize, Clone)]
pub struct ArtifactoryConfig {
    pub repo: String,
    pub group: String,
    pub server: String,
    #[serde(rename = "bintray-repo")]
    pub bintray_repo: Option<String>
}

pub fn parse_toml<S: Into<PathBuf>>(path: S) -> Config {
    return toml::from_str(&file::read_file_to_string(&path.into()))
        .expect("Config to be well formed TOML");
}
