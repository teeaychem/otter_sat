use std::{io::BufReader, path::PathBuf, str::FromStr};

use otter_sat::context::Context;

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

/// Reads the DIMACS file at `path` to `context` and writes a report.
/// If successful, a [PathBuf] to the read file is returned.
pub(super) fn read_dimacs(path: &str, context: &mut Context) -> PathBuf {
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
        None => context.read_dimacs(BufReader::new(&file)),

        Some(extension) if *extension == "xz" => {
            context.read_dimacs(BufReader::new(xz2::read::XzDecoder::new(&file)))
        }

        Some(_) => context.read_dimacs(BufReader::new(&file)),
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

        Err(e) => println!("c Parse error: {e:?}"),
    }

    path
}
