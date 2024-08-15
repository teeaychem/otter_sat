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
    // dbg!(&args);
    if let Ok(contents) = fs::read_to_string(args.file) {
        println!("read");
        if let Ok(new_solve) = Solve::from_dimacs(&contents) {
            let mut the_solve = new_solve;
            // if let Some(unit) = the_solve.find_unit() {
            //     println!("unit: {}", unit.0);
            // }
            // new_solve.assume(Literal::from_string("6").expect("hek"));

            let sat = the_solve.simple_solve();
            println!("SAT? {:?}", sat);

            // dbg!(&the_solve);
        }
    }
}
