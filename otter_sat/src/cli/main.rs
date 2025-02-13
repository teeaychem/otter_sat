use std::{io::BufReader, path::PathBuf, str::FromStr};

use otter_sat::{
    config::Config, context::Context, db::ClauseKey, reports::Report, structures::clause::Clause,
};
use parse_args::parse_args;

mod parse_args;

#[derive(Default)]
struct CliOptions {
    core: bool,
    frat: bool,
}

fn main() {
    let mut cli_options = CliOptions::default();

    let mut ctx: Context = Context::from_config(Config::default());

    let mut args: Vec<String> = std::env::args().collect();

    parse_args(&mut ctx, &mut args, &mut cli_options);

    let path = match PathBuf::from_str(args.last().unwrap()) {
        Ok(path) => path,
        Err(_) => {
            panic!("! Path to CNF required");
        }
    };
    println!("Reading DIMACS file from {path:?}");

    let file = match std::fs::File::open(&path) {
        Ok(path) => path,
        Err(_) => {
            panic!("! Failed to open CNF file");
        }
    };

    let _ = match &path.extension() {
        None => ctx.read_dimacs(BufReader::new(&file)),

        Some(extension) if *extension == "xz" => {
            ctx.read_dimacs(BufReader::new(xz2::read::XzDecoder::new(&file)))
        }

        Some(_) => ctx.read_dimacs(BufReader::new(&file)),
    };

    let Ok(result) = ctx.solve() else {
        println!("hm");
        std::process::exit(2);
    };
    println!("{}", ctx.report());

    if result == Report::Unsatisfiable && cli_options.core {
        let core = ctx.core_keys();
        for key in core {
            match key {
                ClauseKey::OriginalUnit(literal) => {
                    println!("{}", literal.as_dimacs(true));
                }

                _ => {
                    let clause =
                        unsafe { ctx.clause_db.get_unchecked(&key).expect("Core key missing") };
                    println!("{}", clause.as_dimacs(true));
                }
            }
        }
    }
}
