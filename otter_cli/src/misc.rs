use std::{fs::File, io::BufReader, path::PathBuf};

use otter_lib::context::{builder::BuildErr, Context};
use xz2::read::XzDecoder;

pub fn load_dimacs(context: &mut Context, path: PathBuf) -> Result<(), BuildErr> {
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
