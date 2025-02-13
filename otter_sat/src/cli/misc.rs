use otter_sat::{builder::ParserInfo, types::err::ErrorKind};

pub fn examine_parser_report(parse_report: Result<ParserInfo, ErrorKind>) {
    match parse_report {
        Ok(info) => {
            match info.expected_atoms {
                Some(count) => println!("c Expected {count} atoms."),

                None => println!("c No preamble was found."),
            }

            println!("c Added    {} atoms.", info.added_atoms);

            if let Some(count) = info.expected_clauses {
                println!("c Expected {count} clauses.")
            }

            println!("c Added    {} clauses.", info.added_clauses);
        }
        Err(e) => println!("c Parse error: {e:?}"),
    }
}
