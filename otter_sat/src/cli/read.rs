use std::{ffi::OsString, path::PathBuf, str::FromStr};

use otter_sat::{context::Context, types::err::ErrorKind};

pub(super) enum ReadError {
    NoExtension,
    NoPath,
    ParseError(ErrorKind),
    UnknownExtension(OsString),
    FailedToOpen,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::NoExtension => write!(f, "The file does not have an extension."),
            Self::NoPath => write!(f, "Some path to a CNF formula is required."),
            Self::ParseError(err) => write!(f, "Parse error: '{err:?}'."),
            Self::UnknownExtension(ex) => write!(f, "Unsupported extension '{ex:?}'."),
            Self::FailedToOpen => write!(f, "Failed to open CNF file."),
        }
    }
}

/// Reads the DIMACS file at `path` to `context` and writes a report.
/// Results in a [PathBuf] to the read file on success and otherwise a [ReadError]
pub(super) fn read_dimacs(path: &str, context: &mut Context) -> Result<PathBuf, ReadError> {
    let path = match PathBuf::from_str(path) {
        Ok(path) => path,
        Err(_) => return Err(ReadError::NoPath),
    };

    println!("c Reading DIMACS file from {path:?}");

    let file = match std::fs::File::open(&path) {
        Ok(path) => path,
        Err(_) => return Err(ReadError::FailedToOpen),
    };

    let parse_report = match &path.extension() {
        #[cfg(feature = "xz")]
        Some(extension) if *extension == "xz" => {
            context.read_dimacs(std::io::BufReader::new(xz2::read::XzDecoder::new(&file)))
        }

        Some(extension) if *extension == "cnf" => {
            context.read_dimacs(std::io::BufReader::new(&file))
        }

        Some(unknown) => return Err(ReadError::UnknownExtension(unknown.to_owned().to_owned())),

        None => return Err(ReadError::NoExtension),
    };

    match parse_report {
        Ok(info) => {
            match info.expected_atoms {
                Some(count) => println!("c Expected atoms:   {count}"),
                None => println!("c No preamble was found."),
            }

            println!("c Atom count:       {}", info.added_atoms);

            if let Some(count) = info.expected_clauses {
                println!("c Expected clauses: {count}")
            }

            println!("c Clause count:     {}", info.added_clauses);
        }

        Err(e) => {
            return Err(ReadError::ParseError(e));
        }
    }

    Ok(path.as_path().to_owned())
}
