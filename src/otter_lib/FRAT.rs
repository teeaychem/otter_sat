use crate::{
    context::{stores::ClauseKey, unique_id::UniqueIdentifier},
    structures::{self, clause::Clause, literal::Literal},
};

pub struct FRATStr {
    str: String,
}

impl FRATStr {
    pub fn parse_key(key: &ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("f_{index}"),
            ClauseKey::Binary(index) => format!("b_{index}"),
            ClauseKey::Learned(index, _) => format!("l_{index}"),
        }
    }

    pub fn deletion(index: usize) -> Self {
        FRATStr {
            str: format!("d {} 0\n", index),
        }
    }

    // An addition step of a learnt clause
    pub fn learnt(
        add_key: ClauseKey,
        clause: &Vec<Literal>,
        resolution_keys: &Vec<UniqueIdentifier>,
    ) -> Self {
        let mut the_string = String::from("a ");
        the_string.push_str(&FRATStr::parse_key(&add_key));
        the_string.push_str(clause.as_string().as_str());

        if !resolution_keys.is_empty() {
            the_string.push_str(" l ");
            for antecedent in resolution_keys {
                the_string.push_str(format!("{} ", *antecedent as u32).as_str());
            }
        }

        the_string.push_str("0\n");
        FRATStr { str: the_string }
    }

    // A relocation step
    pub fn relocation(from: ClauseKey, to: ClauseKey) -> Self {
        FRATStr {
            str: format!(
                "r {} {} 0\n",
                FRATStr::parse_key(&from),
                FRATStr::parse_key(&to)
            ),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.str.as_bytes()
    }
}
