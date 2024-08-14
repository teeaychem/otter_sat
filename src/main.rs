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
            let mut new_solve = TrailSolve::new(the_cnf);
            if let Some(unit) = new_solve.find_unit() {
                println!("unit: {}", unit.0);
            }
            // new_solve.assume(Literal::from_string("6").expect("hek"));
            dbg!(&new_solve.is_unsat());
            new_solve.assume(Literal::from_string("1").expect("hek"));
            new_solve.assume(Literal::from_string("2").expect("hek"));
            new_solve.assume(Literal::from_string("-6").expect("hek"));
            new_solve.assume(Literal::from_string("3").expect("hek"));

            dbg!(&new_solve.is_sat());
        }
    }
}
