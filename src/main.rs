#![allow(unused_imports)]
#![allow(clippy::single_match)]
// #![allow(unused_variables)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::{
    ffi::OsStr,
    fs,
    io::{BufReader, Read},
    path::PathBuf,
};

use otter_lib::{
    config::Config,
    context::{builder::BuildIssue, Context},
    io::{cli::cli, files::context_from_path},
};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use flate2::read::GzDecoder;

// #[rustfmt::skip]
fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let matches = cli().get_matches();
    let config = Config::from_args(&matches);

    if let Some(formula_paths) = matches.get_raw("paths") {
        for path in formula_paths {
            let mut the_context = match context_from_path(PathBuf::from(path), &config) {
                Ok(context) => context,
                Err(BuildIssue::OopsAllTautologies) => {
                    if config.show_stats {
                        println!("c All clauses of the formula are tautological");
                    }
                    println!("s SATISFIABLE");
                    std::process::exit(0);
                }
                Err(BuildIssue::ClauseEmpty) => {
                    if config.show_stats {
                        println!("c The formula contains an empty clause so is interpreted as ⊥");
                    }
                    println!("s UNSATISFIABLE");
                    std::process::exit(0);
                }
                Err(e) => {
                    panic!("Unexpected error when building: {e:?}");
                }
            };
            // let _ = the_context.clause_from_string("p -q");
            let _the_result = the_context.solve();
            the_context.print_status();
            // dbg!(&the_context.config);
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
