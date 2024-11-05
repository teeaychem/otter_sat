#![allow(unused_imports)]
#![allow(non_snake_case)]

mod dimacs;
mod misc;
mod uniform_random;

use std::fs;
use std::path::{Path, PathBuf};

use otter_lib::{
    config::Config,
    context::{self, Context, Report},
    io::files::*,
};

fn cnf_path() -> PathBuf {
    Path::new(".").join("tests").join("cnf")
}

fn satlib_path() -> PathBuf {
    cnf_path().join("SATLIB")
}

fn satlib_collection(collection: &str) -> PathBuf {
    satlib_path().join(Path::new(collection))
}
