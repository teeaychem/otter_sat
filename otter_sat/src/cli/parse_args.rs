use otter_sat::{
    config::{vsids::VSIDS, Activity, Config, PolarityLean, StoppingCriteria},
    generic::luby::LubyRepresentation,
};

use crate::CliConfig;

/// Parse CLI arguments to a [Config] struct or a [CliConfig] struct.
///
/// If an unrecognised argument or invalid option is found an message is sent and the process is terminated.
pub fn parse_args(args: &mut [String], cfg: &mut Config, cli_options: &mut CliConfig) {
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
