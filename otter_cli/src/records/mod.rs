pub mod core;
pub mod frat;
pub mod window;

use std::sync::{Arc, Mutex};

use crossbeam::channel::Receiver;

use crate::{
    config_io::ConfigIO,
    records::{self},
    window::ContextWindow,
};

use otter_lib::{config::Config, dispatch::Dispatch};

fn hand() -> Box<dyn FnMut(&Dispatch)> {
    Box::new(|_dispatch: &Dispatch| {})
}

#[allow(clippy::result_unit_err)]
pub fn general_recorder(
    rx: Receiver<Dispatch>,
    config: Config,
    config_io: ConfigIO,
    the_graph_ptr: Option<Arc<Mutex<records::core::CoreDB>>>,
) -> Result<(), ()> {
    let mut window = ContextWindow::default();
    window.draw_window(&config);

    let mut windower = records::window::window_writer(&mut window);
    let mut frat_writer = records::frat::build_frat_writer(config_io.frat_path.clone());

    let mut grapher = match the_graph_ptr.is_some() {
        true => records::core::core_db_builder(&the_graph_ptr),
        false => hand(),
    };

    while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::SolveComment(_) => {}
            Dispatch::SolveReport(report) => println!("s {}", report.to_string().to_uppercase()),
            Dispatch::Parser(_) => {}
            Dispatch::Stats(_) => {}
            Dispatch::BCP(_)
            | Dispatch::Resolution(_)
            | Dispatch::VariableDB(_)
            | Dispatch::VariableDBReport(_)
            | Dispatch::ClauseDB(_)
            | Dispatch::ClauseDBReport(_)
            | Dispatch::Level(_) => {}
            Dispatch::Finish => break,
        }

        windower(&dispatch);
        frat_writer(&dispatch);
        grapher(&dispatch);
    }

    drop(grapher);

    Ok(())
}
