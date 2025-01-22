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

use otter_sat::{
    config::Config,
    dispatch::{
        core::{core_db_builder, CoreDB},
        Dispatch,
    },
    types::err::{self},
};

#[allow(clippy::type_complexity)]
fn hand() -> Box<dyn FnMut(&Dispatch) -> Result<(), err::CoreErrorKind>> {
    Box::new(|_dispatch: &Dispatch| -> Result<(), err::CoreErrorKind> { Ok(()) })
}

#[allow(clippy::result_unit_err)]
#[allow(clippy::single_match)]
pub fn general_recorder(
    rx: Receiver<Dispatch>,
    config: Config,
    config_io: ConfigIO,
    the_graph_ptr: Option<Arc<Mutex<CoreDB>>>,
) -> Result<(), ()> {
    let mut window = ContextWindow::default();
    if config_io.show_stats {
        window.draw_window(&config);
    }

    let mut windower = records::window::window_writer(&mut window);
    let mut frat_writer = records::frat::build_frat_writer(config_io.frat_path.clone());

    let mut grapher = match the_graph_ptr.is_some() {
        true => core_db_builder(&the_graph_ptr),
        false => hand(),
    };

    'reception: while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::Delta(_) => {}
            Dispatch::Stat(_) => {}
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
        let _ = grapher(&dispatch);
    }

    drop(grapher);

    Ok(())
}
