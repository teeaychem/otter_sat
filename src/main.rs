#![allow(dead_code, unused_imports)]

use clap::Parser;
use std::fs;
mod ideas;
mod io;
mod solve;
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
    // dbg!(&args);
    if let Ok(contents) = fs::read_to_string(args.file) {
        println!("have string {:?}", contents);

        if let Ok(formula) = Formula::from_dimacs(&contents) {
            let mut the_solve = Solve::from_formula(formula);

            the_solve.literals_of_polarity(true);
            the_solve.literals_of_polarity(false);
            the_solve.hobson_choices();

            let result = the_solve.implication_solve();
            if let Ok((sat, _valuation)) = result {
                println!("SAT? {:?}", sat);
            }

            println!("{}", the_solve.levels.len());
            println!("{}", the_solve);
            // dbg!(&the_solve);
        }
    }
}
