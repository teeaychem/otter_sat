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

use otter_sat::{context::Context, reports::Report, structures::clause::Clause};

mod config;
use config::{CliConfig, parse_args};

mod frat;
use frat::{frat_finalise, frat_setup};

mod read;
use read::read_dimacs;

/// Entrypoint to the CLI.
fn main() {
    let mut cli_options = CliConfig::default();

    let mut args: Vec<String> = std::env::args().collect();

    let cfg = match parse_args(&mut args, &mut cli_options) {
        Ok(config) => config,
        Err(e) => {
            println!("c {e}");
            std::process::exit(1);
        }
    };

    let mut ctx: Context = Context::from_config(cfg);

    let path_string = args.last().unwrap();

    // Read the DIMACS file and store the path for possible FRAT use.
    let path = match read_dimacs(path_string, &mut ctx) {
        Ok(path) => path,
        Err(e) => {
            println!("c {e}");
            std::process::exit(1);
        }
    };

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
                let valuation = ctx.valuation_strings().collect::<Vec<_>>().join(" ");
                println!("v {valuation}",)
            }
        }

        Report::Unsatisfiable => {
            if cli_options.core {
                let core = ctx.core_keys();
                for clause in core {
                    println!("{}", clause.as_dimacs(true));
                }
            }
        }

        Report::Unknown => {}
    }
}
