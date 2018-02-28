#![feature(slice_concat_ext)]
#![feature(rustc_private)]
extern crate chrono;
extern crate clap;
extern crate fern;
extern crate futures;
extern crate git2;
extern crate hyper;
#[macro_use]
extern crate json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serialize;
extern crate tokio_core;
extern crate toml;
extern crate toml_edit;
extern crate url;

#[macro_export]
macro_rules! s {
    ($x:expr) => ( $x.to_string() );
}

pub mod commands;
pub mod logging;
pub mod errors;
pub(crate) mod file;
pub(crate) mod repo;
pub(crate) mod version_manager;
