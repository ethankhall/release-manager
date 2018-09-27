use std::path::PathBuf;

use toml;

use super::file;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub github: GitHubConfig,
    pub artifactory: Option<ArtifactoryConfig>
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "version-file")]
    pub verion_file: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct ArtifactoryConfig {
    pub repo: String,
    pub group: String,
    pub server: String,
    #[serde(rename = "bintray-repo")]
    pub bintray_repo: Option<String>
}

pub fn parse_toml(path: &PathBuf) -> Config {
    return toml::from_str(&file::read_file_to_string(path))
        .expect("Config to be well formed TOML");
}
