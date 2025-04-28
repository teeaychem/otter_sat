use otter_sat::{
    config::{Activity, Config, PolarityLean, StoppingCriteria, vsids::VSIDS},
    generic::luby::LubyRepresentation,
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

pub enum ConfigError {
    NonSpecific(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ConfigError::NonSpecific(s) => write!(f, "{s}"),
        }
    }
}

/// Parse CLI arguments to a [Config] struct or a [CliConfig] struct.
///
/// If an unrecognised argument or invalid option is found an message is sent and the process is terminated.
pub(super) fn parse_args(
    args: &mut [String],
    cli_options: &mut CliConfig,
) -> Result<Config, ConfigError> {
    let mut cfg = Config::default();

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
            Some("--atom_bump") => {
                let (min, max) = cfg.atom_bump.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c atom_bump set to: {value}");
                            cfg.atom_bump.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                return Err(ConfigError::NonSpecific(
                    "atom_bump requires a value between {min} and {max}",
                ));
            }

            Some("--atom_decay") => {
                let (min, max) = cfg.atom_decay.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<Activity>() {
                        if min <= value && value <= max {
                            println!("c atom_decay set to: {value}");
                            cfg.atom_decay.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                return Err(ConfigError::NonSpecific(
                    "atom_decay requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "clause_bump requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "clause_decay requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "conflict_mod requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "lbd_bound requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "luby_mod requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "luby_u requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "phase_saving requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "polarity_lean requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "preprocessing requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "random_decision_bias requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "restarts requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "stopping_criteria requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "subsumption requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "time_limit requires a value between {min} and {max}",
                ));
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

                return Err(ConfigError::NonSpecific(
                    "vsids requires a value between {min} and {max}",
                ));
            }

            Some(_) | None => {
                return Err(ConfigError::NonSpecific(
                    "Unable to parse argument: {arg:?}",
                ));
            }
        }
    }

    Ok(cfg)
}
