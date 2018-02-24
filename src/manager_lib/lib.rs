#![feature(slice_concat_ext)] 
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
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate url;
#[macro_use]
extern crate lazy_static;


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