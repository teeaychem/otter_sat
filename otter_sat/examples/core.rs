use std::{
    io::{BufReader, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use xz2::read::XzDecoder;

use otter_sat::{config::Config, context::Context, structures::clause::Clause};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args {
        println!("{arg}");
    }
    let path = PathBuf::from_str(&args[1]).unwrap();
    let cnf_file = std::fs::File::open(path).unwrap();
    let buf_file = BufReader::new(XzDecoder::new(&cnf_file));

    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };

    let mut the_context: Context = Context::from_config(config, None);

    let _ = the_context.read_dimacs(buf_file);

    let result = the_context.solve();

    println!("{result:?}");

    let key = the_context.conflict_clause().unwrap();

    let mut core_dimacs = String::default();

    let core = the_context.core_keys();
    for key in core {
        let clause = unsafe {
            the_context
                .clause_db
                .get_unchecked(&key)
                .expect("Core key missing")
        };
        core_dimacs.push_str(format!("{}\n", clause.as_dimacs(true)).as_str());
    }

    let mut core_context = Context::from_config(Config::default(), None);
    let mut core_dimacs_buf = vec![];
    let _ = core_dimacs_buf.write(core_dimacs.as_bytes());
    let _ = core_context.read_dimacs(core_dimacs_buf.as_slice());
    let core_result = core_context.solve();

    println!("{:?}", core_result);
}
