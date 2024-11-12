// #![allow(unused_imports)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    cli::{
        config::ConfigIO,
        parse::{self, config_io},
        window::ContextWindow,
    },
    config::Config,
    context::{builder::BuildErr, Context},
    dispatch::{
        delta::{self},
        report::{self},
        stat::{self},
        Dispatch,
    },
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

    let matches = parse::cli::cli().get_matches();

    let mut config = Config::from_args(&matches);
    let config_io = ConfigIO::from_args(&matches);

    if config_io.detail > 0 {
        println!("c Parsing {} files\n", config_io.files.len());
    }

    #[allow(clippy::collapsible_if)]
    if config_io.frat {
        if config.subsumption {
            if config_io.detail > 0 {
                println!("c Subsumption is disabled for FRAT proofs");
            }
            config.subsumption = false;
        }
    }

    dbg!(&config_io);

    let (tx, rx) = unbounded::<Dispatch>();

    // std::process::exit(2);
    // frat_path.push_str(".frat");
    let listener_handle = {
        let config = config.clone();
        let config_io = config_io.clone();
        thread::spawn(|| listener(rx, config, config_io))
    };

    /*
    The context is in a block as:
    - When the block closes the transmitter for the reciever is dropped
    - Unify different ways to get sat/unsat
    At least for now…
     */
    let report = 'report: {
        let mut the_context = Context::from_config(config, tx);

        for path in config_io.files {
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
                if config_io.show_core {
                    // let _ = self.display_core(clause_key);
                }
                the_context.report_active();
            }
            report::Solve::Satisfiable => {
                if config_io.show_valuation {
                    println!("v {}", the_context.valuation_string());
                }
            }
            _ => {}
        }
        the_report
    };

    match report {
        report::Solve::Satisfiable => std::process::exit(10),
        report::Solve::Unsatisfiable => {
            println!("c Finalising FRAT proof…");
            let _ = listener_handle.join();
            std::process::exit(20)
        }
        report::Solve::Unknown => std::process::exit(30),
    };
}

fn listener(rx: Receiver<Dispatch>, config: Config, config_io: ConfigIO) -> Result<(), ()> {
    let mut frat_writer = crate::FRAT::build_frat_writer(&config_io.frat_path);

    let mut window = ContextWindow::default();
    window.draw_window(&config);
    // window.location.

    while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::SolveComment(comment) => {
                window.location.1 -= 1;
                println!("c {}", comment)
            }
            Dispatch::SolveReport(report) => println!("s {}", report.to_string().to_uppercase()),
            Dispatch::Parser(msg) => {
                window.location.1 -= 1;
                println!("c {msg}")
            }
            Dispatch::Stats(stat) => {
                use otter_lib::cli::window::WindowItem;
                match stat {
                    stat::Count::ICD(i, c, d) => {
                        window.update_item(WindowItem::Iterations, i);
                        window.update_item(WindowItem::Decisions, d);
                        window.update_item(WindowItem::Conflicts, c);
                        window.update_item(WindowItem::Ratio, *c as f64 / *i as f64);
                        window.flush();
                    }

                    stat::Count::Time(t) => {
                        window.update_item(WindowItem::Time, format!("{:.2?}", t))
                    }
                }
            }
            Dispatch::Resolution(_)
            | Dispatch::VariableDB(_)
            | Dispatch::VariableDBReport(_)
            | Dispatch::ClauseDB(_)
            | Dispatch::ClauseDBReport(_)
            | Dispatch::Level(_) => {
                frat_writer(dispatch);
            }
        }
    }

    println!("c FRAT proof finalised");
    Ok(())
}
