use crate::{
    context::Context,
    structures::{atom::Atom, valuation::Valuation},
};

impl Context {
    pub fn valuation_string(&self) -> String {
        self.atom_db
            .valuation()
            .vv_pairs()
            .filter_map(|(i, v)| {
                let idx = i as Atom;
                match v {
                    None => None,
                    Some(true) => Some(format!(" {}", self.atom_db.external_representation(idx))),
                    Some(false) => Some(format!("-{}", self.atom_db.external_representation(idx))),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .atom_db
            .valuation()
            .vv_pairs()
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
