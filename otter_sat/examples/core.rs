use std::{
    io::{BufReader, Write},
    path::PathBuf,
    str::FromStr,
    sync::OnceLock,
};

use xz2::read::XzDecoder;

use otter_sat::{
    config::Config, context::Context, db::ClauseKey, dispatch::library::report::SolveReport,
    structures::clause::Clause,
};

static USAGE: OnceLock<String> = OnceLock::new();

macro_rules! fail_with_usage {
    (  ) => {
        println!(
            "{}",
            USAGE
                .get_or_init(|| {
                    format!(
                        "Usage: {} [--check] [--show] <A DIMACS CNF formula>",
                        env!("CARGO_PKG_NAME")
                    )
                })
                .to_string()
        );
        std::process::exit(1);
    };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let argc = args.len();
    if argc < 2 {
        fail_with_usage!();
    }

    let mut show = false;
    let mut check = false;

    for arg in args.iter().skip(1).rev().skip(1) {
        match arg.as_str() {
            "--show" => show = true,
            "--check" => check = true,
            _ => {
                fail_with_usage!();
            }
        }
    }

    let path = match PathBuf::from_str(&args[argc.saturating_sub(1)]) {
        Ok(path) => path,
        Err(_) => {
            fail_with_usage!();
        }
    };

    let cnf_file = match std::fs::File::open(path) {
        Ok(path) => path,
        Err(_) => {
            panic!("! Failed to open CNF file");
        }
    };

    let buf_file = BufReader::new(XzDecoder::new(&cnf_file));

    let mut the_context: Context = Context::from_config(Config::default(), None);

    let _ = the_context.read_dimacs(buf_file);

    let result = the_context.solve();
    assert_eq!(result, Ok(SolveReport::Unsatisfiable));

    let mut core_dimacs = String::default();

    let core = the_context.core_keys();
    for key in core {
        match key {
            ClauseKey::OriginalUnit(literal) => {
                core_dimacs.push_str(format!("{}\n", literal.as_dimacs(true)).as_str());
            }

            _ => {
                let clause = unsafe {
                    the_context
                        .clause_db
                        .get_unchecked(&key)
                        .expect("Core key missing")
                };
                core_dimacs.push_str(format!("{}\n", clause.as_dimacs(true)).as_str());
            }
        }
    }

    if show {
        println!("{core_dimacs}");
    }

    if check {
        let mut core_context = Context::from_config(Config::default(), None);
        let mut core_dimacs_buf = vec![];
        let _ = core_dimacs_buf.write(core_dimacs.as_bytes());
        let _ = core_context.read_dimacs(core_dimacs_buf.as_slice());
        let core_result = core_context.solve();

        assert_eq!(core_result, Ok(SolveReport::Unsatisfiable));
        println!("c Check ok!")
    }
}
