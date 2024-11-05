// #![allow(unused_imports)]
#![allow(clippy::single_match)]
// #![allow(unused_variables)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    config::Config,
    context::{builder::BuildErr, Report},
    errors::ClauseStoreErr,
    io::{cli::cli, files::context_from_path},
};

use std::path::PathBuf;

fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let matches = cli().get_matches();
    let config = Config::from_args(&matches);

    let Some(mut formula_paths) = matches.get_raw("paths") else {
        println!("c Could not find formula paths");
        std::process::exit(1);
    };

    if config.verbosity > 0 {
        println!("c Found {} formulas\n", formula_paths.len());
    }

    match formula_paths.len() {
        1 => {
            let the_path = PathBuf::from(formula_paths.next().unwrap());
            let the_report = report_on_formula(the_path, &config);
            match the_report {
                otter_lib::context::Report::Satisfiable => std::process::exit(10),
                otter_lib::context::Report::Unsatisfiable => std::process::exit(20),
                otter_lib::context::Report::Unknown => std::process::exit(30),
            }
        }
        _ => {
            for path in formula_paths {
                report_on_formula(PathBuf::from(path), &config);
                println!();
            }
            std::process::exit(0)
        }
    }
}

fn report_on_formula(path: PathBuf, config: &Config) -> Report {
    let mut the_context = match context_from_path(path, config) {
        Ok(context) => context,
        Err(BuildErr::OopsAllTautologies) => {
            if config.verbosity > 0 {
                println!("c All clauses of the formula are tautological");
            }
            println!("s SATISFIABLE");
            std::process::exit(10);
        }
        Err(BuildErr::ClauseStore(ClauseStoreErr::EmptyClause)) => {
            if config.verbosity > 0 {
                println!("c The formula contains an empty clause so is interpreted as ⊥");
            }
            println!("s UNSATISFIABLE");
            std::process::exit(20);
        }
        Err(e) => {
            println!("c Unexpected error when building: {e:?}");
            std::process::exit(2);
        }
    };
    let the_report = match the_context.solve() {
        Ok(report) => report,
        Err(e) => {
            println!("Context error: {e:?}");
            std::process::exit(1);
        }
    };
    the_context.print_status();
    the_report
}
