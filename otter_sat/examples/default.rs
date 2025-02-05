use std::{io::BufReader, path::PathBuf, str::FromStr};

use otter_sat::{config::Config, context::Context};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args {
        println!("{arg}");
    }
    let path = PathBuf::from_str(&args[1]).unwrap();
    let cnf_file = std::fs::File::open(path).unwrap();
    let buf_file = BufReader::new(&cnf_file);

    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };

    let mut the_context: Context = Context::from_config(config, None);

    let _ = the_context.read_dimacs(buf_file);

    let result = the_context.solve();

    println!("{result:?}");
}
