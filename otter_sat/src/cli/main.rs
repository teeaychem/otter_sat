/*!
A simple CLI interface to the library.

# Use

```sh
otter_cli [--option(=value)]* file.cnf
```

## Options

No configuration option is required.
Though, of note:

- `--core` enables printing an unsatisfiable core on an unsatisfiable result.
- `--frat` enables writing an FRAT proof beside the cnf file (with the `.frat` extension).

For full documentation of the supported options, see the source of [parse_args].

## Extensions


The `.cnf` file extension is required, unless the `xz` feature has been enabled.[^1]

[^1]: The [Global Benchmark Database](https://benchmark-database.de) uses xz compression.
*/

use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use otter_sat::{
    config::{vsids::VSIDS, Activity, Config, PolarityLean, StoppingCriteria},
    context::Context,
    db::{clause::db_clause::dbClause, ClauseKey},
    generic::luby::LubyRepresentation,
    reports::{
        frat::{
            callback_templates::{
                transcribe_addition, transcribe_deletion, transcribe_premises,
                transcribe_unsatisfiable,
            },
            Transcriber,
        },
        Report,
    },
    structures::clause::{Clause, ClauseSource},
};

/// A collection of configuration options relevant only to the CLI.
#[derive(Default)]
pub struct CliConfig {
    /// Whether to report and unsatisfiable core, if one exists.
    pub core: bool,

    /// Whether to produce an FRAT proof, if the formula is unsatisfiable.
    pub frat: bool,

    /// Whether to report a model, if one exists.
    pub model: bool,
}

/// Entrypoint to the CLI.
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

// FRAT functions

/// Create a file to write the FRAT proof to and set transiption callbacks where required.
///
/// Returns a smart pointer to the transcriber.
fn frat_setup(cnf_path: &Path, ctx: &mut Context) -> Rc<RefCell<Transcriber>> {
    let mut frat_path = cnf_path.as_os_str().to_os_string();
    frat_path.push(".frat");

    let frat_path = PathBuf::from(&frat_path);

    let transcriber = Transcriber::new(frat_path.clone()).unwrap();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(transcriber));

    let addition_tx = tx.clone();
    let addition_cb = move |clause: &dbClause, source: &ClauseSource| {
        transcribe_addition(&mut addition_tx.borrow_mut(), clause, source)
    };
    ctx.set_callback_addition(Box::new(addition_cb));

    let deletion_tx = tx.clone();
    let deletion_cb =
        move |clause: &dbClause| transcribe_deletion(&mut deletion_tx.borrow_mut(), clause);
    ctx.set_callback_delete(Box::new(deletion_cb));

    let resolution_tx = tx.clone();
    let resolution_cb = move |premises: &HashSet<ClauseKey>| {
        transcribe_premises(&mut resolution_tx.borrow_mut(), premises)
    };
    ctx.resolution_buffer
        .set_callback_resolution_premises(Box::new(resolution_cb));

    let unsatisfiable_tx = tx.clone();
    let unsatisfiable_cb = move |clause: &dbClause| {
        transcribe_unsatisfiable(&mut unsatisfiable_tx.borrow_mut(), clause)
    };
    ctx.set_callback_unsatisfiable(Box::new(unsatisfiable_cb));

    tx
}

/// Finalise the FRAT proof written by the given transcriber.
fn frat_finalise(transcriber: Rc<RefCell<Transcriber>>, context: &mut Context) {
    for (key, literal) in context.clause_db.all_unit_clauses() {
        transcriber.borrow_mut().transcribe_active(key, &literal);
    }

    for (key, clause) in context.clause_db.all_active_nonunit_clauses() {
        transcriber.borrow_mut().transcribe_active(key, clause);
    }

    transcriber.borrow_mut().flush();
}

// Misc functions

/// Reads the DIMACS file at `path` to `context` and writes a report.
/// If successful, a [PathBuf] to the read file is returned.
fn read_dimacs(path: &str, context: &mut Context) -> PathBuf {
    let path = match PathBuf::from_str(path) {
        Ok(path) => path,
        Err(_) => {
            println!("c Path to CNF required.");
            std::process::exit(1);
        }
    };

    println!("c Reading DIMACS file from {path:?}");

    let file = match std::fs::File::open(&path) {
        Ok(path) => path,
        Err(_) => {
            println!("c Failed to open CNF file.");
            std::process::exit(1);
        }
    };

    let parse_report = match &path.extension() {
        #[cfg(feature = "xz")]
        Some(extension) if *extension == "xz" => {
            context.read_dimacs(BufReader::new(xz2::read::XzDecoder::new(&file)))
        }

        Some(extension) if *extension == "cnf" => {
            context.read_dimacs(std::io::BufReader::new(&file))
        }

        _ => {
            println!("c The file does not contain a supported extension.");
            std::process::exit(1);
        }
    };

    match parse_report {
        Ok(info) => {
            match info.expected_atoms {
                Some(count) => println!("c Expected {count} atoms."),

                None => println!("c No preamble was found."),
            }

            println!("c Added    {} atoms.", info.added_atoms);

            if let Some(count) = info.expected_clauses {
                println!("c Expected {count} clauses.")
            }

            println!("c Added    {} clauses.", info.added_clauses);
        }

        Err(e) => {
            println!("c Parse error: {e:?}");
            std::process::exit(1);
        }
    }

    path
}

// Argument parser

/// Parse CLI arguments to a [Config] struct or a [CliConfig] struct.
///
/// If an unrecognised argument or invalid option is found an message is sent and the process is terminated.
fn parse_args(args: &mut [String], cfg: &mut Config, cli_options: &mut CliConfig) {
    'arg_examination: for arg in args.iter().skip(1).rev().skip(1) {
        let mut split = arg.split("=");
        match split.next() {
            Some("--core") => {
                println!("c An unsatisfiable core will be written, if one exists.");
                cli_options.core = true;
            }

            Some("--frat") => {
                //frat setup
                println!("c FRAT proof will be generated aside the cnf file.");
                cli_options.frat = true;
            }

            Some("--model") | Some("--valuation") => {
                println!("c A model will be written, if one exists.");
                cli_options.model = true;
            }

            // The remaining cases follow a common template.
            // If a value is present, may be parsed appropriately, and is valid, the config is updated.
            // Otherwise, a message is sent.
            //
            // Further, the cases should be in lexicographic order.
            //
            Some("--atom_bump") => {
                let (min, max) = cfg.atom_db.bump.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c atom_bump set to: {value}");
                            cfg.atom_db.bump.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("atom_bump requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--atom_decay") => {
                let (min, max) = cfg.atom_db.decay.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c atom_decay set to: {value}");
                            cfg.atom_db.decay.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("atom_decay requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--clause_bump") => {
                let (min, max) = cfg.clause_db.bump.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c clause_bump set to: {value}");
                            cfg.clause_db.bump.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("clause_bump requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--clause_decay") => {
                let (min, max) = cfg.clause_db.decay.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c clause_decay set to: {value}");
                            cfg.clause_db.decay.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("clause_decay requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--conflict_mod") => {
                let (min, max) = cfg.conflict_mod.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<u32>() {
                        if min <= value && value <= max {
                            println!("c conflict_mod set to: {value}");
                            cfg.conflict_mod.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("conflict_mod requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--lbd_bound") => {
                let (min, max) = cfg.clause_db.lbd_bound.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<u8>() {
                        if min <= value && value <= max {
                            println!("c lbd_bound set to: {value}");
                            cfg.clause_db.lbd_bound.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("lbd_bound requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--luby_mod") => {
                let (min, max) = cfg.luby_mod.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<u32>() {
                        if min <= value && value <= max {
                            println!("c luby_mod set to: {value}");
                            cfg.luby_mod.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("luby_mod requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--luby_u") => {
                let (min, max) = cfg.luby_u.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<LubyRepresentation>() {
                        if min <= value && value <= max {
                            println!("c luby_u set to: {value}");
                            cfg.luby_u.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("luby_u requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--phase_saving") => {
                let (min, max) = cfg.phase_saving.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c phase_saving set to: {value}");
                            cfg.phase_saving.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("phase_saving requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--polarity_lean") => {
                let (min, max) = cfg.polarity_lean.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<PolarityLean>() {
                        if min <= value && value <= max {
                            println!("c polarity_lean set to: {value}");
                            cfg.polarity_lean.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("polarity_lean requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--preprocessing") => {
                let (min, max) = cfg.preprocessing.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c preprocessing set to: {value}");
                            cfg.preprocessing.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("preprocessing requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--random_decision_bias") => {
                let (min, max) = cfg.random_decision_bias.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<PolarityLean>() {
                        if min <= value && value <= max {
                            println!("c random_decision_bias set to: {value}");
                            cfg.random_decision_bias.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("random_decision_bias requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--restarts") => {
                let (min, max) = cfg.restarts.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c restarts set to: {value}");
                            cfg.restarts.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("restarts requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--stopping_criteria") => {
                let (min, max) = cfg.stopping_criteria.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<StoppingCriteria>() {
                        if min <= value && value <= max {
                            println!("c stopping_criteria set to: {value}");
                            cfg.stopping_criteria.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("stopping_criteria requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--subsumption") => {
                let (min, max) = cfg.subsumption.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c subsumption set to: {value}");
                            cfg.subsumption.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("subsumption requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--time_limit") => {
                let (min, max) = cfg.time_limit.min_max();
                let min = min.as_secs();
                let max = max.as_secs();

                if let Some(request) = split.next() {
                    if let Ok(seconds) = request.parse::<u64>() {
                        if min <= seconds && seconds <= max {
                            println!("c time_limit set to: {seconds} seconds");
                            cfg.time_limit.value = std::time::Duration::from_secs(seconds);
                            continue 'arg_examination;
                        }
                    }
                }

                println!("time_limit requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some("--vsids") => {
                let (min, max) = cfg.vsids.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<VSIDS>() {
                        if min <= value && value <= max {
                            println!("c VSIDS set to: {value}");
                            cfg.vsids.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("vsids requires a value between {min} and {max}");
                std::process::exit(1);
            }

            Some(_) | None => {
                println!("Unable to parse argument: {arg:?}");
                std::process::exit(1);
            }
        }
    }
}
