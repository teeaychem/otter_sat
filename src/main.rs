// #![allow(unused_imports)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    config::Config,
    context::{builder::BuildErr, Context},
    dispatch::{
        delta::{self},
        report::{self},
        Dispatch,
    },
    io::cli::cli,
    types::errs::{self},
    FRAT,
};

use std::path::PathBuf;

use crossbeam::channel::{unbounded, Receiver};
use std::thread;

fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let matches = cli().get_matches();
    let formula_paths = paths(&matches);

    let mut config = Config::from_args(&matches);

    if config.io.detail > 0 {
        println!("c Parsing {} files\n", formula_paths.len());
    }

    let (tx, rx) = unbounded::<Dispatch>();

    let the_path = formula_paths.first().unwrap().clone();
    let frat_file = format!("{}.frat", the_path.file_name().unwrap().to_str().unwrap());
    let mut frat_path = std::env::current_dir().unwrap();
    frat_path.push("frat");
    frat_path.push(frat_file);

    println!("{:?}", frat_path);

    // std::process::exit(2);
    // frat_path.push_str(".frat");
    let frat_path = Some(PathBuf::from(&frat_path));
    let listener_handle = thread::spawn(|| listener(rx, frat_path));

    /*
    The context is in a block as:
    - When the block closes the transmitter for the reciever is dropped
    - Unify different ways to get sat/unsat
    At least for now…
     */
    let report = 'report: {
        let unique_config = config.clone();
        let mut the_context = Context::from_config(unique_config, tx);

        for path in formula_paths {
            println!("{path:?}");
            match the_context.load_dimacs_file(path) {
                Ok(()) => {}
                Err(BuildErr::ClauseStore(errs::ClauseDB::EmptyClause)) => {
                    println!("s UNSATISFIABLE");
                    std::process::exit(20);
                }
                Err(e) => {
                    println!("c Error loading DIMACS: {e:?}")
                }
            };
        }

        if the_context.clause_count() == 0 {
            break 'report report::Solve::Satisfiable;
        }

        let the_report = match the_context.solve() {
            Ok(r) => r,
            Err(e) => {
                println!("Context error: {e:?}");
                std::process::exit(1);
            }
        };

        match the_report {
            report::Solve::Unsatisfiable => {
                if config.io.show_core {
                    // let _ = self.display_core(clause_key);
                }
                the_context.report_active();
            }
            report::Solve::Satisfiable => {
                if config.io.show_valuation {
                    println!("v {}", the_context.valuation_string());
                }
            }
            _ => {}
        }
        the_report
    };

    match report {
        report::Solve::Satisfiable => {
            // println!("v {}", the_context.valuation_string());
            std::process::exit(10)
        }
        report::Solve::Unsatisfiable => {
            println!("c Finalising FRAT proof…");

            let _ = listener_handle.join();
            std::process::exit(20)
        }
        report::Solve::Unknown => std::process::exit(30),
    };
}

fn paths(args: &clap::ArgMatches) -> Vec<PathBuf> {
    let formula_paths = match args.get_many::<PathBuf>("paths") {
        None => {
            println!("c Could not find formula paths");
            std::process::exit(1);
        }
        Some(paths) => paths.cloned().collect(),
    };
    formula_paths
}

fn listener(rx: Receiver<Dispatch>, frat_path: Option<PathBuf>) -> Result<(), ()> {
    let mut frat_transcriber = FRAT::Transcriber::new(frat_path.unwrap());
    let mut resolution_buffer = Vec::default();

    while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::SolveComment(comment) => println!("c {}", comment),
            Dispatch::SolveReport(report) => println!("s {}", report.to_string().to_uppercase()),

            Dispatch::Resolution(r_delta) => match r_delta {
                delta::Resolution::Start => {
                    assert!(resolution_buffer.is_empty())
                }
                delta::Resolution::Used(k) => resolution_buffer.push(*k),
                delta::Resolution::Finish => {
                    frat_transcriber.take_resolution(std::mem::take(&mut resolution_buffer))
                }
                delta::Resolution::Subsumed(_, _) => {
                    // TODO: Someday… maybe…
                }
            },
            Dispatch::Parser(msg) => println!("c {msg}"),
            Dispatch::Level(_) => {
                frat_transcriber.transcripe(dispatch);
            }
            _ => {
                frat_transcriber.transcripe(dispatch);
            }
        }
        frat_transcriber.flush();
    }

    println!("c FRAT proof finalised");
    assert!(frat_transcriber.resolution_buffer.is_empty());
    Ok(())
}

//     match the_status {
//         SolveStatus::FullValuation | SolveStatus::NoClauses => report::Solve::Satisfiable,
//         SolveStatus::NoSolution => report::Solve::Unsatisfiable,
//         _ => report::Solve::Unknown,
//     }
