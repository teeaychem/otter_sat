use std::{io::BufReader, path::PathBuf, str::FromStr};

use otter_sat::{config::Config, context::Context};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args {
        println!("{arg}");
    }
    let path = match PathBuf::from_str(&args[1]) {
        Ok(path) => path,
        Err(_) => {
            panic!("! Path to CNF required");
        }
    };
    let cnf_file = match std::fs::File::open(path) {
        Ok(path) => path,
        Err(_) => {
            panic!("! Failed to open CNF file");
        }
    };

    let buf_file = BufReader::new(&cnf_file);

    let mut ctx: Context = Context::from_config(Config::default());

    let _ = ctx.read_dimacs(buf_file);

    let result = ctx.solve();

    println!("{result:?}");
}
