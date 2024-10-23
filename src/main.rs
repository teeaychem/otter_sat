#![allow(unused_imports)]
// #![allow(unused_variables)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap::Parser;
use std::{
    fs,
    io::{BufReader, Read},
};

use otter_lib::{
    context::{
        config::{Args, Config},
        Context,
    },
    structures::literal::{Literal, Source},
};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// #[rustfmt::skip]
fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let config = Config::from_args(Args::parse());

    // let mut the_basic_context = Context::default_config(&config);
    // the_basic_context.clause_from_string("r s t");
    // the_basic_context.clause_from_string("-q");
    // let _ = the_basic_context.literal_update(Literal::new(0, true), 0, Source::Assumption);
    // let _the_result = the_basic_context.solve();
    // the_basic_context.print_status();

    let mut the_context = Context::from_dimacs(&config.formula_file.clone().unwrap(), &config);
    the_context.clause_from_string("p -q");
    let _the_result = the_context.solve();
    the_context.print_status();
}
