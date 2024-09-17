#![allow(dead_code, unused_imports)]

use clap::Parser;
use std::fs;
mod ideas;
mod io;
mod solve;
mod structures;

use crate::structures::*;
use log::{warn, info};
use log4rs;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// file to parse
    #[arg(short, long)]
    file: String,
}

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let args = Args::parse();
    // dbg!(&args);
    if let Ok(contents) = fs::read_to_string(args.file) {
        if let Ok(formula) = Formula::from_dimacs(&contents) {
            let mut the_solve = Solve::from_formula(&formula);

            let result = the_solve.implication_solve();
            if let Ok(valuation) = result {
                println!("Satisfying assignment: {:?}", valuation);
            }
            // println!("{}", the_solve);
            // dbg!(&the_solve);
        }
    }
}
