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
    path::PathBuf,
};

use otter_lib::{
    config::{Config, StoppingCriteria},
    context::Context,
    io::cli::cli,
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

    let matches = cli().get_matches();
    let config = Config::from_args(&matches);
    // dbg!(&config);

    if let Some(formula_paths) = matches.get_raw("paths") {
        for path in formula_paths {
            let the_path = PathBuf::from(path);

            let mut the_context =
                Context::from_dimacs(&the_path, &config).expect("failed to build context");
            // let _ = the_context.clause_from_string("p -q");
            let _the_result = the_context.solve();
            the_context.print_status();
        }
    }

    // let mut the_basic_context = Context::default_config(&config);

    // let mut require_basic_build =
    //     |clause_string| match the_basic_context.clause_from_string(clause_string) {
    //         Ok(()) => {}
    //         Err(e) => panic!("failed to build: {e:?}"),
    //     };

    // require_basic_build("q");
    // // require_basic_build("-q");
    // require_basic_build("r s t");
    // let assumption = the_basic_context.literal_from_string("-q");
    // match the_basic_context.assume_literal(assumption) {
    //     Ok(_) => {
    //         println!("made assumption");
    //     }
    //     Err(e) => {
    //         println!("failed to build: {e:?}")
    //     }
    // };
    // let _the_result = the_basic_context.solve();
    // the_basic_context.print_status();
}
