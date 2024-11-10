// #![allow(unused_imports)]

use clap::ArgMatches;
#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    config::Config,
    context::{
        builder::BuildErr,
        delta::{Dispatch, SolveReport},
    },
    io::{cli::cli, files::context_from_path},
    types::{errs::ClauseStoreErr, gen::SolveStatus},
};

use std::path::PathBuf;

use crossbeam::channel::{unbounded, Receiver, Sender};
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

    let frat = true;
    if frat {
        let frat_path = "temp.txt";
        config.io.frat_path = Some(PathBuf::from(frat_path));
        let _ = std::fs::File::create(frat_path);
    }

    if config.io.detail > 0 {
        println!("c Found {} formulas\n", formula_paths.len());
    }

    let (tx, rx) = unbounded::<Dispatch>();

    thread::spawn(|| listener(rx));

    let formula_count = formula_paths.len();

    let report = match formula_count {
        0 => panic!("no formulas"),
        1 => {
            let the_path = formula_paths.first().unwrap().clone();
            let tx = tx.clone();
            thread::spawn(move || report_on_formula(the_path, tx, config))
                .join()
                .expect("o what the heck")
        }
        _ => {
            config.io.show_stats = false;
            let mut last_report = None;

            for path in formula_paths {
                let config_clone = config.clone();
                let tx = tx.clone();
                let y = thread::spawn(move || report_on_formula(path, tx, config_clone))
                    .join()
                    .unwrap();
                last_report = Some(y)
            }
            last_report.expect("bo")
        }
    };
    // drop(tx);

    match formula_count {
        0 => panic!("o_x"),
        1 => match report {
            SolveReport::Satisfiable => std::process::exit(10),
            SolveReport::Unsatisfiable => std::process::exit(20),
            SolveReport::Unknown => std::process::exit(30),
        },
        _ => std::process::exit(0),
    }
}

fn paths(args: &ArgMatches) -> Vec<PathBuf> {
    let formula_paths = {
        if args.get_many::<PathBuf>("paths").is_none() {
            println!("c Could not find formula paths");
            std::process::exit(1);
        } else {
            args.get_many::<PathBuf>("paths")
                .unwrap()
                .cloned()
                .collect()
        }
    };
    formula_paths
}

fn listener(rx: Receiver<Dispatch>) {
    while let Ok(dispatch) = rx.recv() {
        match dispatch {
            Dispatch::SolveComment(comment) => println!("c {}", comment),
            Dispatch::SolveReport(report) => println!("s {}", report.to_string().to_uppercase()),
            Dispatch::Parser(msg) => println!("c {msg}"),
            _ => {}
        }
    }
}

// TODO: unify the exceptions…
fn report_on_formula(path: PathBuf, tx: Sender<Dispatch>, config: Config) -> SolveReport {
    let config_io_detail = config.io.detail;
    // let config_io_frat_path = config.io.frat_path.clone();

    use otter_lib::context::delta::SolveComment;
    let (the_context, mut the_report) = match context_from_path(path, config.clone(), tx.clone()) {
        Ok(context) => (Some(context), None),
        Err(BuildErr::ClauseStore(ClauseStoreErr::EmptyClause)) => {
            if config_io_detail > 0 {
                let _ = tx.send(Dispatch::SolveComment(SolveComment::FoundEmptyClause));
            }
            (None, Some(SolveStatus::NoSolution))
        }
        Err(e) => {
            println!("c Unexpected error when building: {e:?}");
            std::process::exit(2);
        }
    };

    // if config_io_frat_path.is_some() {
    //     the_context.frat_formula()
    // }

    if let Some(mut the_context) = the_context {
        if the_context.clause_count() == 0 {
            if config_io_detail > 0 {
                let _ = tx.send(Dispatch::SolveComment(SolveComment::NoClauses));
            }
            the_report = Some(SolveStatus::NoClauses);
        } else {
            match the_context.solve() {
                Ok(report) => {
                    match report {
                        SolveReport::Satisfiable => {
                            if config.io.show_valuation {
                                println!("v {}", the_context.valuation_string());
                            }
                        }
                        SolveReport::Unsatisfiable => {
                            if config.io.show_core {
                                // let _ = self.display_core(clause_key);
                            }
                        }
                        SolveReport::Unknown => {}
                    }

                    the_report = Some(the_context.status)
                }
                Err(e) => {
                    println!("Context error: {e:?}");
                    std::process::exit(1);
                }
            }
        }
    };

    // if config_io_frat_path.is_some() {
    //     the_context.frat_finalise()
    // }

    let the_status = the_report.expect("no status");

    match the_status {
        SolveStatus::FullValuation | SolveStatus::NoClauses => SolveReport::Satisfiable,
        SolveStatus::NoSolution => SolveReport::Unsatisfiable,
        _ => SolveReport::Unknown,
    }
}
