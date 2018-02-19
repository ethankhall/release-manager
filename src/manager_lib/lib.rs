#![feature(rustc_private)]
extern crate serialize;
#[macro_use]
extern crate log;
#[macro_use]
extern crate json;
extern crate clap;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;
extern crate fern;
extern crate chrono;
extern crate git2;
extern crate semver;
extern crate toml;
extern crate toml_edit;

#[macro_export]
macro_rules! s {
    ($x:expr) => ( $x.to_string() );
}

pub mod commands;
pub mod logging;
pub(crate) mod file;
pub(crate) mod repo;
pub(crate) mod version_manager;