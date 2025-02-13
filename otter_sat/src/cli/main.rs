use std::{io::BufReader, path::PathBuf, str::FromStr};

use otter_sat::{
    config::Config, context::Context, db::ClauseKey, generic::luby::LubyRepresentation,
    reports::Report, structures::clause::Clause,
};

fn main() {
    let mut core = false;

    let mut ctx: Context = Context::from_config(Config::default());

    let args: Vec<String> = std::env::args().collect();
    'arg_examination: for arg in args.iter().skip(1).rev().skip(1) {
        let mut split = arg.split("=");
        match split.next() {
            Some("--frat") => {
                //frat setup
                println!("c An FRAT proof will be generated");
            }

            Some("--core") => {
                //frat setup
                println!("c An unsatisfiable core will be written, if one exists.");
                core = true;
            }

            Some("--luby") => {
                let (min, max) = ctx.config.luby_u.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<LubyRepresentation>() {
                        if min <= value && value <= max {
                            println!("c Luby u value set to: {value}");
                            ctx.config.luby_u.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("The luby configuration option requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some(_) | None => {
                println!("Unable to parse argument: {arg:?}");
                std::process::exit(1);
            }
        }
    }

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

    match &path.extension() {
        None => {
            let _ = ctx.read_dimacs(BufReader::new(&file));
        }
        Some(extension) if *extension == "xz" => {
            let _ = ctx.read_dimacs(BufReader::new(xz2::read::XzDecoder::new(&file)));
        }
        Some(_) => {
            let _ = ctx.read_dimacs(BufReader::new(&file));
        }
    };

    let result = ctx.solve();

    if result.is_ok_and(|report| report == Report::Unsatisfiable) && core {
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
