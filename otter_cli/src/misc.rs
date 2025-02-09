use std::{fs::File, io::BufReader, path::PathBuf};

use otter_sat::{
    context::Context,
    types::err::{self},
};
use xz2::read::XzDecoder;

pub fn load_dimacs(context: &mut Context, path: PathBuf) -> Result<(), err::ErrorKind> {
    let file = match File::open(&path) {
        Err(e) => panic!("c COULD NOT LOAD FILE: {e:?}"),
        Ok(f) => f,
    };

    match &path.extension() {
        None => {
            context.read_dimacs(BufReader::new(&file))?;
        }
        Some(extension) if *extension == "xz" => {
            context.read_dimacs(BufReader::new(XzDecoder::new(&file)))?;
        }
        Some(_) => {
            context.read_dimacs(BufReader::new(&file))?;
        }
    };
    Ok(())
}
