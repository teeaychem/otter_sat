use crate::{context::Context, structures::variable::Variable};

impl Context {
    pub fn valuation_string(&self) -> String {
        self.variable_db
            .valuation()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let idx = i as Variable;
                match v {
                    None => None,
                    Some(true) => Some(format!(
                        " {}",
                        self.variable_db.external_representation(idx)
                    )),
                    Some(false) => Some(format!(
                        "-{}",
                        self.variable_db.external_representation(idx)
                    )),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .variable_db
            .valuation()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
