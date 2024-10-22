use crate::{
    context::{config::Config, Context},
    structures::{
        clause::stored::Source as ClauseSource,
        literal::{Literal, Source as LiteralSource},
        variable::{list::VariableList, Variable, VariableId},
    },
};

impl Context {
    pub fn literal_from_string(&mut self, string: &str) -> Literal {
        let trimmed_string = string.trim();
        assert!(
            !trimmed_string.is_empty() && trimmed_string != "-",
            "No variable when creating literal from string"
        );
        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..];
        }

        self.literal_ensure(the_name, polarity)
    }

    pub fn literal_ensure(&mut self, name: &str, polarity: bool) -> Literal {
        let the_variable = {
            match self.variables.iter().find(|v| v.name() == name) {
                Some(variable) => variable.id(),
                None => {
                    let the_id = self.variables.len() as VariableId;
                    self.variables.add_variable(Variable::new(name, the_id));
                    the_id
                }
            }
        };
        Literal::new(the_variable, polarity)
    }

    pub fn clause_from_string(&mut self, string: &str) {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            let the_literal = self.literal_from_string(string_literal);
            the_clause.push(the_literal);
        }
        the_clause.sort_unstable();
        the_clause.dedup();

        assert!(
            !the_clause.is_empty(),
            "c The formula contains an empty clause"
        );

        match the_clause.len() {
            1 => {
                self.literal_update(
                    *the_clause.first().expect("literal vanish"),
                    0,
                    LiteralSource::Assumption,
                );
            }
            _ => {
                self.store_clause(the_clause, ClauseSource::Formula);
            }
        }
    }

    pub fn from_dimacs(string: &str, config: &Config) -> Self {
        let mut the_context = Context::default();

        let mut from = 0;
        let mut to = 0;
        let mut reading_comment = false;
        let mut reading_literal = false;
        while let Some(ch) = string.chars().nth(to) {
            if !reading_literal {
                if ['-', '1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&ch) {
                    reading_literal = true;
                } else if ch == '0' {
                    if !reading_comment {
                        the_context.clause_from_string(&string[from..to]);
                    }
                    from = to + 1;
                }
            }
            if reading_literal && ch.is_whitespace() {
                reading_literal = false;
            }

            if ch == 'c' {
                reading_comment = true;
                from += 1;
            } else if ch == 0xA as char {
                // newline check
                from = to;
                reading_comment = false;
            } else if !reading_comment && ch == 'p' {
                loop {
                    to += 1;
                    if string.chars().nth(to).expect("IO: Parse failure") == 0xA as char {
                        break;
                    }
                }
                let the_preface = &string[from..to];
                let preface_parts = the_preface.split_whitespace().collect::<Vec<_>>();

                assert!(preface_parts.len() == 4, "IO: Puzzled by preface length");
                assert!(preface_parts[0] == "p", "IO: Puzzled by preface format");
                assert!(preface_parts[1] == "cnf", "IO: Puzzled by preface format");

                let variables = match preface_parts[2].parse::<usize>() {
                    Ok(count_number) => count_number,
                    Err(e) => panic!("IO: Parse failure {e:?}"),
                };

                let clauses = match preface_parts[3].parse::<usize>() {
                    Ok(count_number) => count_number,
                    Err(e) => panic!("IO: Parse failure {e:?}"),
                };
                from = to;

                assert!(the_context.variables().slice().is_empty());

                the_context = Context::with_size_hints(variables, clauses, config.clone())
            }

            to += 1;
        }
        log::trace!("Context made from DIMACS");
        the_context
    }
}
