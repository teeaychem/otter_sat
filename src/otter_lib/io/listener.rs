use std::path::PathBuf;

use crossbeam::channel::Receiver;

use crate::{
    cli::{config::ConfigIO, window::ContextWindow},
    config::Config,
    dispatch::{
        delta::{self},
        report::{self},
        stat::{self},
        Dispatch,
    },
    FRAT::{self},
};

#[allow(clippy::result_unit_err)]
pub fn general_receiver(
    rx: Receiver<Dispatch>,
    config: Config,
    config_io: ConfigIO,
) -> Result<(), ()> {
    let mut frat_writer = build_frat_writer(&config_io.frat_path);

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
                use crate::cli::window::WindowItem;
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
            | Dispatch::Level(_) => {}
        }
        frat_writer(dispatch);
    }

    Ok(())
}

pub fn frat_receiver(rx: Receiver<Dispatch>, frat_path: PathBuf) {
    let mut frat_writer = build_frat_writer(&Some(frat_path));

    while let Ok(dispatch) = rx.recv() {
        frat_writer(dispatch);
    }
}

/// If given a path the writer transcribes dispatches to the path as an FRAT proof.
/// Otherwise, then writer does nothing
pub fn build_frat_writer(frat_path: &Option<PathBuf>) -> Box<dyn FnMut(Dispatch)> {
    match frat_path {
        None => {
            let hand = |_: Dispatch| {};
            Box::new(hand)
        }
        Some(path) => {
            let mut transcriber = FRAT::Transcriber::new(path);
            let handler = move |dispatch: Dispatch| {
                transcriber.transcribe(dispatch);
                transcriber.flush()
            };
            Box::new(handler)
        }
    }
}