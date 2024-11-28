use std::{
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{unbounded, Receiver};
use otter_lib::{
    config::Config,
    context::Context,
    dispatch::{
        core::{core_db_builder, CoreDB},
        library::report::Report,
        Dispatch,
    },
    structures::clause::ClauseT,
};

#[allow(clippy::result_unit_err)]
#[allow(clippy::single_match)]
pub fn database_recorder(
    rx: Receiver<Dispatch>,
    the_graph_ptr: Arc<Mutex<CoreDB>>,
) -> Result<(), ()> {
    let some_ptr = Some(the_graph_ptr);
    let mut grapher = core_db_builder(&some_ptr);
    'reception: while let Ok(dispatch) = rx.recv() {
        match &dispatch {
            Dispatch::Delta(_) | Dispatch::Stat(_) => {}
            Dispatch::Report(the_report) => {
                //
                match the_report {
                    Report::Finish => break 'reception,
                    _ => {}
                }
            }
        }
        let _ = grapher(&dispatch);
    }
    drop(grapher);
    Ok(())
}

fn main() {
    let core_db_ptr = Arc::new(Mutex::new(CoreDB::default()));

    let (tx, rx) = unbounded::<Dispatch>();
    let core_db_ptr_clone = core_db_ptr.clone();
    let rx = thread::spawn(|| database_recorder(rx, core_db_ptr_clone));

    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };
    let mut the_context = Context::from_config(
        config,
        Some(Rc::new(move |d: Dispatch| {
            let _ = tx.send(d);
        })),
    );

    let mut dimacs = vec![];

    let _ = dimacs.write(
        b"
 p  q    0
 p -q    0
-p  q    0
-p -q    0
 p  q  r 0
-p  q -r 0
 r -s    0
",
    );

    println!("The DIMACS representation of 𝐅 reads:");
    println!("{}", std::str::from_utf8(&dimacs).unwrap());

    assert!(the_context.read_dimacs(dimacs.as_slice()).is_ok());
    assert!(the_context.solve().is_ok());

    let _ = rx.join();

    let the_core_db = core_db_ptr;
    let the_core_db = match the_core_db.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Ok(core_keys) = the_core_db.core_clauses() {
        println!("An unsatisfiable core of 𝐅 is:");
        for core_clause in core_keys {
            println!("{}", core_clause.as_dimacs(&the_context.variable_db, true));
        }
    }
}
