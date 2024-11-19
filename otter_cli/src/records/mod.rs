pub mod core;
pub mod frat;
pub mod window;

use std::sync::{Arc, Mutex};

use crossbeam::channel::Receiver;

use crate::{
    config_io::ConfigIO,
    records::{self},
    report::{self},
    window::ContextWindow,
};

use otter_lib::{config::context::Config, dispatch::Dispatch};

fn hand() -> Box<dyn FnMut(&Dispatch)> {
    Box::new(|_dispatch: &Dispatch| {})
}

#[allow(clippy::result_unit_err)]
#[allow(clippy::single_match)]
pub fn general_recorder(
    rx: Receiver<Dispatch>,
    config: Config,
    config_io: ConfigIO,
    the_graph_ptr: Option<Arc<Mutex<records::core::CoreDB>>>,
) -> Result<(), ()> {
    let mut window = ContextWindow::default();
    if config_io.show_stats {
        window.draw_window(&config);
    }

    let mut windower = records::window::window_writer(&mut window);
    let mut frat_writer = records::frat::build_frat_writer(config_io.frat_path.clone());

    let mut grapher = match the_graph_ptr.is_some() {
        true => records::core::core_db_builder(&the_graph_ptr),
        false => hand(),
    };

    'reception: while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::Comment(_) => {}
            Dispatch::Delta(_) => {}
            Dispatch::Stats(_) => {}
            Dispatch::Report(the_report) => {
                //
                match the_report {
                    report::Report::Solve(report) => {
                        println!("s {}", report.to_string().to_uppercase())
                    }

                    report::Report::Finish => break 'reception,
                    _ => {}
                }
            }
        }
        if config_io.show_stats {
            windower(&dispatch);
        }
        frat_writer(&dispatch);
        grapher(&dispatch);
    }

    drop(grapher);

    Ok(())
}
