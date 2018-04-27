#![feature(assoc_unix_epoch)]
#![deny(unused_extern_crates)]
extern crate chrono;
extern crate clap;
extern crate fern;
extern crate futures;
extern crate git2;
extern crate hyper;
extern crate hyper_tls;
extern crate ini;
#[macro_use]
extern crate json;
#[macro_use]
extern crate log;
extern crate mime;
extern crate mime_guess;
extern crate semver;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate toml;
extern crate toml_edit;
extern crate url;
extern crate tar;
extern crate glob;

#[macro_export]
macro_rules! s {
    ($x:expr) => {
        $x.to_string()
    };
}

pub mod commands;
pub mod config;
pub mod errors;
pub(crate) mod file;
pub(crate) mod git;
pub(crate) mod http;
pub mod logging;
pub(crate) mod version_manager;
