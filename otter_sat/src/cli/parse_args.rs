use otter_sat::{context::Context, generic::luby::LubyRepresentation};

use crate::CliOptions;

pub fn parse_args(ctx: &mut Context, args: &mut [String], cli_options: &mut CliOptions) {
    'arg_examination: for arg in args.iter().skip(1).rev().skip(1) {
        let mut split = arg.split("=");
        match split.next() {
            Some("--frat") => {
                //frat setup
                println!("c An FRAT proof will be generated");
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
}
