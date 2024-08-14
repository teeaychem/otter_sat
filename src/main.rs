use clap::Parser;
use std::fs;
mod ideas;
mod io;
mod structures;

use crate::structures::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// file to parse
    #[arg(short, long)]
    file: String,
}

fn main() {
    println!("Hello, world!");
    let args = Args::parse();
    dbg!(&args);
    if let Ok(contents) = fs::read_to_string(args.file) {
        println!("read");
        if let Ok(the_cnf) = Cnf::from_dimacs(&contents) {
            let new_solve = Solve::new(the_cnf);
            if let Some(unit) = new_solve.find_unit() {
                println!("unit: {}", unit.0);
            }
            // dbg!(&new_solve);
        }
    }
}
