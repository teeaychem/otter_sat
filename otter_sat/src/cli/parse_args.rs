use otter_sat::{
    config::{vsids::VSIDS, PolarityLean, StoppingCriteria},
    context::Context,
    generic::luby::LubyRepresentation,
};

use crate::CliOptions;

pub fn parse_args(ctx: &mut Context, args: &mut [String], cli_options: &mut CliOptions) {
    'arg_examination: for arg in args.iter().skip(1).rev().skip(1) {
        let mut split = arg.split("=");
        match split.next() {
            Some("--frat") => {
                //frat setup
                println!("c FRAT proof will be generated aside the cnf file.");
                cli_options.frat = true;
            }

            Some("--core") => {
                println!("c An unsatisfiable core will be written, if one exists.");
                cli_options.core = true;
            }

            Some("--luby") => {
                let (min, max) = ctx.config.luby_u.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<LubyRepresentation>() {
                        if min <= value && value <= max {
                            println!("c luby_u set to: {value}");
                            ctx.config.luby_u.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("luby_u requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--polarity_lean") => {
                let (min, max) = ctx.config.polarity_lean.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<PolarityLean>() {
                        if min <= value && value <= max {
                            println!("c polarity_lean set to: {value}");
                            ctx.config.polarity_lean.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("polarity_lean requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--random_decision_bias") => {
                let (min, max) = ctx.config.random_decision_bias.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<PolarityLean>() {
                        if min <= value && value <= max {
                            println!("c random_decision_bias set to: {value}");
                            ctx.config.random_decision_bias.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("random_decision_bias requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--stopping_criteria") => {
                let (min, max) = ctx.config.stopping_criteria.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<StoppingCriteria>() {
                        if min <= value && value <= max {
                            println!("c stopping_criteria set to: {value}");
                            ctx.config.stopping_criteria.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("stopping_criteria requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--phase_saving") => {
                let (min, max) = ctx.config.phase_saving.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c phase_saving set to: {value}");
                            ctx.config.phase_saving.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("phase_saving requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--preprocessing") => {
                let (min, max) = ctx.config.preprocessing.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c preprocessing set to: {value}");
                            ctx.config.preprocessing.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("preprocessing requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--restarts") => {
                let (min, max) = ctx.config.restarts.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c restarts set to: {value}");
                            ctx.config.restarts.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("restarts requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--subsumption") => {
                let (min, max) = ctx.config.subsumption.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<bool>() {
                        if min <= value && value <= max {
                            println!("c subsumption set to: {value}");
                            ctx.config.subsumption.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("subsumption requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--time_limit") => {
                let (min, max) = ctx.config.time_limit.min_max();
                let min = min.as_secs();
                let max = max.as_secs();

                if let Some(request) = split.next() {
                    if let Ok(seconds) = request.parse::<u64>() {
                        if min <= seconds && seconds <= max {
                            println!("c time_limit set to: {seconds} seconds");
                            ctx.config.time_limit.value = std::time::Duration::from_secs(seconds);
                            continue 'arg_examination;
                        }
                    }
                }

                println!("time_limit requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--vsids") => {
                let (min, max) = ctx.config.vsids.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<VSIDS>() {
                        if min <= value && value <= max {
                            println!("c VSIDS set to: {value}");
                            ctx.config.vsids.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("vsids requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--luby_mod") => {
                let (min, max) = ctx.config.luby_mod.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<u32>() {
                        if min <= value && value <= max {
                            println!("c luby_mod set to: {value}");
                            ctx.config.luby_mod.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("luby_mod requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some("--conflict_mod") => {
                let (min, max) = ctx.config.conflict_mod.min_max();

                if let Some(request) = split.next() {
                    if let Ok(value) = request.parse::<u32>() {
                        if min <= value && value <= max {
                            println!("c luby__mod set to: {value}");
                            ctx.config.conflict_mod.value = value;
                            continue 'arg_examination;
                        }
                    }
                }

                println!("conflict_mod requires a value between {min} and {max}",);
                std::process::exit(1);
            }

            Some(_) | None => {
                println!("Unable to parse argument: {arg:?}");
                std::process::exit(1);
            }
        }
    }
}
