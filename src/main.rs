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
    context::{builder::BuildErr, delta::Delta},
    io::{cli::cli, files::context_from_path},
    types::{errs::ClauseStoreErr, gen::Report},
};

use std::path::PathBuf;

use crossbeam::channel::{unbounded, Sender};
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

    let (tx, rx) = unbounded::<Delta>();

    match formula_paths.len() {
        1 => {
            let the_path = formula_paths.first().unwrap().clone();
            let the_report = thread::spawn(move || report_on_formula(the_path, tx, config));
            match the_report.join().unwrap() {
                Report::Satisfiable => std::process::exit(10),
                Report::Unsatisfiable => std::process::exit(20),
                Report::Unknown => std::process::exit(30),
            }
        }
        _ => {
            config.io.show_stats = false;

            for path in formula_paths {
                let config_clone = config.clone();
                let tx = tx.clone();
                thread::spawn(move || {
                    report_on_formula(path, tx, config_clone);
                });
            }
        }
    };
    drop(tx);
    while let Ok(delta) = rx.recv() {
        match delta {
            Delta::SolveReport(report) => {
                println!("> {report}");
            }
            _ => {}
        }
    }
    std::process::exit(0)
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

fn report_on_formula(path: PathBuf, tx: Sender<Delta>, config: Config) -> Report {
    let config_io_detail = config.io.detail;
    let config_io_frat_path = config.io.frat_path.clone();

    use otter_lib::context::delta::SolveComment;
    let mut the_context = match context_from_path(path, config, tx.clone()) {
        Ok(context) => context,
        Err(BuildErr::OopsAllTautologies) => {
            if config_io_detail > 0 {
                tx.send(Delta::SolveComment(SolveComment::AllTautological))
                    .unwrap();
            }
            // tx.send("s SATISFIABLE\n".to_string()).unwrap();
            std::process::exit(10);
        }
        Err(BuildErr::ClauseStore(ClauseStoreErr::EmptyClause)) => {
            if config_io_detail > 0 {
                tx.send(Delta::SolveComment(SolveComment::FoundEmptyClause))
                    .unwrap();
            }
            // tx.send("s UNSATISFIABLE\n".to_string()).unwrap();
            std::process::exit(20);
        }
        Err(e) => {
            println!("c Unexpected error when building: {e:?}");
            std::process::exit(2);
        }
    };
    if the_context.clause_count() == 0 {
        if config_io_detail > 0 {
            tx.send(Delta::SolveComment(SolveComment::NoClauses))
                .unwrap();
        }
        // tx.send("s SATISFIABLE\n".to_string()).unwrap();
        std::process::exit(10);
    }

    if config_io_frat_path.is_some() {
        the_context.frat_formula()
    }

    let the_report = match the_context.solve() {
        Ok(report) => report,
        Err(e) => {
            println!("Context error: {e:?}");
            std::process::exit(1);
        }
    };

    if config_io_frat_path.is_some() {
        the_context.frat_finalise()
    }

    the_context.print_status(tx);
    the_report
}
