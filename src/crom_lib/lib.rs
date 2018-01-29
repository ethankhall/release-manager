#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;

#[macro_export]
macro_rules! s {
    ($x:expr) => ( $x.to_string() );
}

pub static DEFAULT_BASE_URL: &'static str = "http://api.crom.tech";

pub mod requests;
pub mod commands;

pub trait ToCromPath {
    fn to_crom_path(&self) -> String;

    fn friendly_name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub project: String,
    pub repo: String
}

impl ToCromPath for Repo {
    fn to_crom_path(&self) -> String {
        return format!("/api/v1/project/{projectName}/repo/{repoName}", projectName=self.project, repoName=self.repo);
    }

    fn friendly_name(&self) -> String {
        return format!("Project {}; Name {}", self.project, self.repo);
    }
}
