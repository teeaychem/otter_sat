use std::{fs::File, io::BufReader, path::PathBuf};

use otter_lib::{
    context::Context,
    types::err::{self},
};
use xz2::read::XzDecoder;

pub fn load_dimacs(context: &mut Context, path: PathBuf) -> Result<(), err::Build> {
    let file = match File::open(&path) {
        Err(_) => panic!("c COULD NOT LOAD FILE"),
        Ok(f) => f,
    };

    match &path.extension() {
        None => {
            context.load_dimacs_file(BufReader::new(&file))?;
        }
        Some(extension) if *extension == "xz" => {
            context.load_dimacs_file(BufReader::new(XzDecoder::new(&file)))?;
        }
        Some(_) => {
            context.load_dimacs_file(BufReader::new(&file))?;
        }
    };
    Ok(())
}
