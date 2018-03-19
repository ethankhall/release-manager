#![deny(unused_extern_crates)]
extern crate chrono;
extern crate clap;
extern crate fern;
extern crate futures;
extern crate git2;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate json;
#[macro_use]
extern crate log;
extern crate semver;
extern crate tokio_core;
extern crate toml;
extern crate toml_edit;
extern crate url;
extern crate ini;
extern crate mime_guess;
extern crate mime;
#[macro_use]
extern crate serde_derive;

#[macro_export]
macro_rules! s {
    ($x:expr) => ( $x.to_string() );
}

pub mod commands;
pub mod logging;
pub mod errors;
pub mod config;
pub(crate) mod http;
pub(crate) mod file;
pub(crate) mod git;
pub(crate) mod version_manager;
