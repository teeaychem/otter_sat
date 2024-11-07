use super::stores::ClauseKey;

pub enum CFG {
    Original,
    Addition,
    Deletion,
    Relocation,
    Finalisation,
    Comment,
}
