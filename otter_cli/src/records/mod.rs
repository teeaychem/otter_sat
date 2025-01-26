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
    dispatch::Dispatch,
    types::err::{self},
};

#[allow(clippy::type_complexity)]
fn hand() -> Box<dyn FnMut(&Dispatch) -> Result<(), err::CoreError>> {
    Box::new(|_dispatch: &Dispatch| -> Result<(), err::CoreError> { Ok(()) })
}

#[allow(clippy::result_unit_err)]
#[allow(clippy::single_match)]
pub fn general_recorder(
    rx: Receiver<Dispatch>,
    config: Config,
    config_io: ConfigIO,
) -> Result<(), ()> {
    let mut window = ContextWindow::default();
    if config_io.show_stats {
        window.draw_window(&config);
    }

    let mut windower = records::window::window_writer(&mut window);
    let mut frat_writer = records::frat::build_frat_writer(config_io.frat_path.clone());

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
    }

    Ok(())
}
