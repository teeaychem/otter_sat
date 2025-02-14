use frat::{frat_finalise, frat_setup};
use misc::read_dimacs;
use otter_sat::{
    config::Config, context::Context, db::ClauseKey, reports::Report, structures::clause::Clause,
};
use parse_args::parse_args;

mod frat;

mod misc;
use misc::CliConfig;

mod parse_args;

fn main() {
    let mut cli_options = CliConfig::default();

    let mut cfg = Config::default();

    let mut args: Vec<String> = std::env::args().collect();

    parse_args(&mut args, &mut cfg, &mut cli_options);

    let mut ctx: Context = Context::from_config(cfg);

    // Read the DIMACS file and store the path for possible FRAT use.
    let path = read_dimacs(args.last().unwrap(), &mut ctx);

    // Setup a transcriber if an FRAT proof is requested and initialise relevant callbacks.
    // If returned, the pointer to the transcriber is used to finalise the proof.
    let tx = match cli_options.frat {
        true => Some(frat_setup(&path, &mut ctx)),
        false => None,
    };

    let result = match ctx.solve() {
        Ok(yes) => yes,

        Err(e) => {
            println!("c Solve error: {e:?}");
            std::process::exit(2);
        }
    };

    // Finalise the FRAT proof, if one is being written.
    if let Some(tx) = tx {
        frat_finalise(tx, &mut ctx);
    }

    println!("s {}", ctx.report());

    // Further actions, depending on the configuration.
    match result {
        Report::Satisfiable => {
            if cli_options.model {
                println!("v {}", ctx.atom_db.valuation_string())
            }
        }

        Report::Unsatisfiable => {
            if cli_options.core {
                let core = ctx.core_keys();
                for key in core {
                    match key {
                        ClauseKey::OriginalUnit(literal) => {
                            println!("{}", literal.as_dimacs(true));
                        }

                        _ => {
                            let clause = unsafe {
                                ctx.clause_db
                                    .get_unchecked(&key)
                                    .expect("c Core key missing")
                            };
                            println!("{}", clause.as_dimacs(true));
                        }
                    }
                }
            }
        }

        _ => {}
    }
}
