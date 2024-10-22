#![allow(unused_imports)]
// #![allow(unused_variables)]

#[cfg(feature = "jemalloc")]
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap::Parser;
use std::fs;

use otter_lib::context::{
    config::{Args, Config},
    Context,
};

// #[rustfmt::skip]
fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let config = Config::from_args(Args::parse());

    match fs::read_to_string(config.formula_file.clone().unwrap()) {
        Ok(contents) => {
            let mut the_context = Context::from_dimacs(&contents, &config);
            the_context.clause_from_string("p -q");

            // let _ = the_context.step(&config_clone);

            let _the_result = the_context.solve();

            the_context.print_status();
        }
        Err(e) => println!("Error reading file {e:?}"),
    }
}
