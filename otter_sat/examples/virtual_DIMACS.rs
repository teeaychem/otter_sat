use std::io::Write;

use otter_sat::{config::Config, context::Context};

fn main() {
    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };
    let mut context_two = Context::from_config(config, None);
    let mut dimacs = vec![];
    let _ = dimacs.write(b"-p  q  0\n");
    let _ = dimacs.write(b" p -q 0\n p    0");

    println!("The DIMACS representation of 𝐅 reads:");
    println!("{}", std::str::from_utf8(&dimacs).unwrap());

    assert!(context_two.read_dimacs(dimacs.as_slice()).is_ok());
    assert!(context_two.solve().is_ok());

    let status = context_two.report();
    let valuation = context_two.atom_db.valuation_string();
    println!("After solving the status of 𝐅 is: {status} (with valuation 𝐕: {valuation})");
}
