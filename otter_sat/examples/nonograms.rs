use std::collections::HashMap;

/// A nonogram solver.
///
/// Given the representation of a nonogram a cnf fomula is built and passed to an otter_sat.
/// A satisfying valuation is a solution to the puzzle.
/// If a solution, the valuation is parsed, and the completed nonogram is displayed.
use otter_sat::{
    config::Config,
    context::Context,
    reports::Report::{self},
    structures::{
        atom::Atom,
        clause::CClause,
        literal::{CLiteral, Literal},
    },
};

/// A struct to hold the nonogram puzzle, clause generation follows the main function.
struct Nonogram {
    row_length: usize,
    col_length: usize,
    rows: Vec<Vec<usize>>,
    cols: Vec<Vec<usize>>,

    atom_map: HashMap<String, Atom>,
    context: Context,
}

fn main() {
    // https://en.wikipedia.org/wiki/Nonogram#/media/File:Nonogram_wiki.svg
    let mut puzzle = Nonogram {
        row_length: 30,
        col_length: 20,
        rows: vec![
            vec![8, 7, 5, 7],
            vec![5, 4, 3, 3],
            vec![3, 3, 2, 3],
            vec![4, 3, 2, 2],
            vec![3, 3, 2, 2],
            vec![3, 4, 2, 2],
            vec![4, 5, 2],
            vec![3, 5, 1],
            vec![4, 3, 2],
            vec![3, 4, 2],
            vec![4, 4, 2],
            vec![3, 6, 2],
            vec![3, 2, 3, 1],
            vec![4, 3, 4, 2],
            vec![3, 2, 3, 2],
            vec![6, 5],
            vec![4, 5],
            vec![3, 3],
            vec![3, 3],
            vec![1, 1],
        ],
        cols: vec![
            vec![1],
            vec![1],
            vec![2],
            vec![4],
            vec![7],
            vec![9],
            vec![2, 8],
            vec![1, 8],
            vec![8],
            vec![1, 9],
            vec![2, 7],
            vec![3, 4],
            vec![6, 4],
            vec![8, 5],
            vec![1, 11],
            vec![1, 7],
            vec![8],
            vec![1, 4, 8],
            vec![6, 8],
            vec![4, 7],
            vec![2, 4],
            vec![1, 4],
            vec![5],
            vec![1, 4],
            vec![1, 5],
            vec![7],
            vec![5],
            vec![3],
            vec![1],
            vec![1],
        ],
        atom_map: HashMap::default(),
        context: Context::from_config(Config::default()),
    };

    for clause in puzzle.clauses() {
        let _ = puzzle.context.add_clause(clause.clone());
    }

    let clause_count = puzzle.context.clause_db.total_clause_count();
    println!("Context build with {clause_count} clauses",);

    let result = puzzle.context.solve();

    if matches!(result, Ok(Report::Unsatisfiable)) {
        println!("No solution");
        std::process::exit(0);
    }
    println!(
        "Solution:
"
    );

    let valuation = puzzle.context.valuations_ints();

    let mut display = vec![vec![0; puzzle.row_length]; puzzle.col_length];

    let reverse_map = puzzle
        .atom_map
        .iter()
        .map(|(k, v)| (*v, k.clone()))
        .collect::<HashMap<Atom, String>>();

    for literal in valuation {
        let atom_string = reverse_map.get(&(literal.unsigned_abs() as Atom)).unwrap();

        if literal.is_positive() && atom_string.starts_with("Fill") {
            let idx = &atom_string[5..atom_string.len() - 1]
                .split(",")
                .map(|s| s.parse::<usize>().unwrap())
                .collect::<Vec<_>>();
            display[idx[0]][idx[1]] = 1;
        }
    }

    for row in display {
        for col in row {
            match col {
                0 => print!(" "),
                1 => print!("x"),
                _ => panic!("! An incomplete nonogram valuation"),
            }
        }
        println!();
    }
}

/// Clause generation
///
/// This is quite inefficient!
///
/// Roughly, the idea is to:
///
/// - Create literals capturing when a cell has been filled.
/// - Create literals capturing when blocks exist.
///
/// The puzzle can then be represented in terms of requirements and constraints on blocks.
/// Specifically, that a block exists on a row or column, and than a block occurs prior to some other block.
///
/// The representation of blocks is split into two literals.
///
/// - A literal capturing the start position of a block
/// - A literal capturing the length of a block.
///
/// The blocks are linked by a block id, given by when the block occurs in the row or colum on the puzzle position.
/// Coherence between rows and columns is guaranteed by the fill literals.
///
/// In this respect, the generation of row/column clauses is a equivalent to the generation of column/row clauses, with a few variables changed.
impl Nonogram {
    fn fill_literal(&mut self, row: usize, col: usize, polarity: bool) -> CLiteral {
        let atom_string = format!("Fill({row},{col})");
        let atom = match self.atom_map.get(&atom_string) {
            Some(atom) => *atom,
            None => {
                let atom = self.context.fresh_or_max_atom();
                self.atom_map.insert(atom_string, atom);
                atom
            }
        };

        CLiteral::new(atom, polarity)
    }

    fn block_start_row_literal(
        &mut self,
        row: usize,
        col: usize,
        block_idx: usize,
        polarity: bool,
    ) -> CLiteral {
        let atom_string = format!("BlockStartRow({row},{col},{block_idx})");
        let atom = match self.atom_map.get(&atom_string) {
            Some(atom) => *atom,
            None => {
                let atom = self.context.fresh_or_max_atom();
                self.atom_map.insert(atom_string, atom);
                atom
            }
        };

        CLiteral::new(atom, polarity)
    }

    fn block_start_col_literal(
        &mut self,
        row: usize,
        col: usize,
        block_idx: usize,
        polarity: bool,
    ) -> CLiteral {
        let atom_string = format!("BlockStartCol({row},{col},{block_idx})");
        let atom = match self.atom_map.get(&atom_string) {
            Some(atom) => *atom,
            None => {
                let atom = self.context.fresh_or_max_atom();
                self.atom_map.insert(atom_string, atom);
                atom
            }
        };

        CLiteral::new(atom, polarity)
    }

    fn block_legnth_row_literal(
        &mut self,
        row: usize,
        block_idx: usize,
        length: usize,
        polarity: bool,
    ) -> CLiteral {
        let atom_string = format!("BlockLengthRow({row},{block_idx},{length})");
        let atom = match self.atom_map.get(&atom_string) {
            Some(atom) => *atom,
            None => {
                let atom = self.context.fresh_or_max_atom();
                self.atom_map.insert(atom_string, atom);
                atom
            }
        };

        CLiteral::new(atom, polarity)
    }

    fn block_length_col_literal(
        &mut self,
        col: usize,
        block_idx: usize,
        length: usize,
        polarity: bool,
    ) -> CLiteral {
        let atom_string = format!("BlockLengthCol({col},{block_idx},{length})");
        let atom = match self.atom_map.get(&atom_string) {
            Some(atom) => *atom,
            None => {
                let atom = self.context.fresh_or_max_atom();
                self.atom_map.insert(atom_string, atom);
                atom
            }
        };

        CLiteral::new(atom, polarity)
    }
}

impl Nonogram {
    fn row_clauses_block_start(&mut self, row: usize, total_blocks: usize) -> Vec<CClause> {
        let mut the_clauses: Vec<CClause> = vec![];

        let mut starts = vec![];
        for block_idx in 0..total_blocks {
            starts.push(self.block_start_row_literal(row, 0, block_idx, true));
        }

        let start_literal = self.fill_literal(row, 0, false);
        let mut required_fill = vec![start_literal];
        required_fill.extend_from_slice(&starts);

        the_clauses.push(required_fill);

        for idx in 1..self.row_length {
            let mut starts = vec![];
            for block_idx in 0..total_blocks {
                starts.push(self.block_start_row_literal(row, idx, block_idx, true));
            }
            let prior_literal = self.fill_literal(row, idx - 1, true);
            let start_literal = self.fill_literal(row, idx, false);
            let mut required_fill = vec![prior_literal, start_literal];
            required_fill.extend_from_slice(&starts);

            the_clauses.push(required_fill);
        }

        the_clauses
    }

    fn row_clauses_block_start_fills(&mut self, row: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for col in 0..self.row_length {
            let start_literal = self.block_start_row_literal(row, col, block_idx, false);
            let must_fill_literal = self.fill_literal(row, col, true);
            let clause = vec![start_literal, must_fill_literal];
            clauses.push(clause);
            if col > 0 {
                let no_fill_literal = self.fill_literal(row, col - 1, false);
                let clause = vec![start_literal, no_fill_literal];
                clauses.push(clause);
            }
        }

        clauses
    }

    fn row_clauses_block_fill(&mut self, row: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for col in 0..self.row_length {
            let start_literal = self.block_start_row_literal(row, col, block_idx, false);
            for length in 2..(self.row_length - col) {
                let length_literal = self.block_legnth_row_literal(row, block_idx, length, false);
                for offest in 0..length {
                    let fill_literal = self.fill_literal(row, col + offest, true);
                    let clause = vec![start_literal, length_literal, fill_literal];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_length(&mut self, row: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for start_col in 0..self.row_length {
            let start_block_literal =
                self.block_start_row_literal(row, start_col, block_idx, false);

            for length in 1..=(self.row_length - start_col) {
                let mut the_clause = vec![start_block_literal];
                for offset in 1..length {
                    let fill_literal = self.fill_literal(row, start_col + offset, false);
                    the_clause.push(fill_literal);
                }

                if start_col + length < self.row_length {
                    let length_literal = self.fill_literal(row, start_col + length, true);
                    the_clause.push(length_literal);
                }
                let length_literal = self.block_legnth_row_literal(row, block_idx, length, true);
                the_clause.push(length_literal);
                clauses.push(the_clause);
            }
        }

        clauses
    }

    fn row_clauses_block_starts_somewhere(&mut self, row: usize, block_idx: usize) -> CClause {
        let mut clause = vec![];

        for col in 0..self.row_length {
            clause.push(self.block_start_row_literal(row, col, block_idx, true));
        }

        clause
    }

    fn row_clauses_block_start_unique_col(&mut self, row: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for col_idx in 0..self.row_length {
            let block_start_col = self.block_start_row_literal(row, col_idx, block_idx, false);
            for other_col_idx in 0..self.row_length {
                if col_idx != other_col_idx {
                    let block_start_other_col =
                        self.block_start_row_literal(row, other_col_idx, block_idx, false);
                    let clause = vec![block_start_col, block_start_other_col];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_length_unique(&mut self, row: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for length in 1..=self.row_length {
            let block_length_literal = self.block_legnth_row_literal(row, block_idx, length, false);
            for other_length in 1..=self.row_length {
                if length != other_length {
                    let other_block_length_literal =
                        self.block_legnth_row_literal(row, block_idx, other_length, false);
                    let clause = vec![block_length_literal, other_block_length_literal];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_start_unique_idx(
        &mut self,
        row: usize,
        total_blocks: usize,
    ) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for col_idx in 0..self.row_length {
            for block_idx in 0..total_blocks {
                let block_literal = self.block_start_row_literal(row, col_idx, block_idx, false);
                for other_idx in 0..total_blocks {
                    if block_idx != other_idx {
                        let other_literal =
                            self.block_start_row_literal(row, col_idx, other_idx, false);
                        let clause = vec![block_literal, other_literal];
                        clauses.push(clause);
                    }
                }
            }
        }

        clauses
    }

    fn row_clauses_precedence(
        &mut self,
        row: usize,
        block_a_idx: usize,
        block_b_idx: usize,
    ) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for col_idx in 0..2 {
            clauses.push(vec![self.block_start_row_literal(
                row,
                col_idx,
                block_b_idx,
                false,
            )]);
        }

        for col_idx in 2..self.row_length {
            let mut literals = vec![];
            let block_b_at_idx = self.block_start_row_literal(row, col_idx, block_b_idx, false);
            literals.push(block_b_at_idx);
            for prior_col in 0..col_idx {
                let block_a_prior = self.block_start_row_literal(row, prior_col, block_a_idx, true);
                literals.push(block_a_prior);
            }
            clauses.push(literals);
        }

        clauses
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_range_loop)]
    fn row_clauses(&mut self) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for (row_idx, row) in self.rows.clone().iter().enumerate() {
            if row.is_empty() {
                continue;
            }

            let mut row_clauses = vec![];
            row_clauses.append(&mut self.row_clauses_block_start(row_idx, row.len()));
            row_clauses.append(&mut self.row_clauses_block_start_unique_idx(row_idx, row.len()));

            clauses.append(&mut row_clauses);

            for block_idx in 0..row.len() {
                let mut block_clauses = vec![];
                block_clauses.push(self.row_clauses_block_starts_somewhere(row_idx, block_idx));
                block_clauses.push(vec![self.block_legnth_row_literal(
                    row_idx,
                    block_idx,
                    row[block_idx],
                    true,
                )]);

                block_clauses.append(&mut self.row_clauses_block_start_fills(row_idx, block_idx));
                block_clauses.append(&mut self.row_clauses_block_length(row_idx, block_idx));

                block_clauses.append(&mut self.row_clauses_block_fill(row_idx, block_idx));
                block_clauses.append(&mut self.row_clauses_block_start_unique_col(row_idx, block_idx));
                block_clauses.append(&mut self.row_clauses_block_length_unique(row_idx, block_idx));

                clauses.append(&mut block_clauses);
            }

            let mut precedence_clauses = vec![];
            for block_idx in 0..row.len() - 1 {
                let mut pair_clauses = self.row_clauses_precedence(row_idx, block_idx, block_idx + 1);
                precedence_clauses.append(&mut pair_clauses);
            }
            clauses.append(&mut precedence_clauses);
        }

        clauses
    }
}

impl Nonogram {
    fn col_clauses_block_start(&mut self, col: usize, total_blocks: usize) -> Vec<CClause> {
        let mut the_clauses = vec![];

        let mut starts = vec![];
        for block_idx in 0..total_blocks {
            starts.push(self.block_start_col_literal(0, col, block_idx, true));
        }

        let start_literal = self.fill_literal(0, col, false);
        let mut required_fill = vec![start_literal];
        required_fill.extend_from_slice(&starts);

        the_clauses.push(required_fill);

        for idx in 1..self.col_length {
            let mut starts = vec![];
            for block_idx in 0..total_blocks {
                starts.push(self.block_start_col_literal(idx, col, block_idx, true));
            }
            let prior_literal = self.fill_literal(idx - 1, col, true);
            let start_literal = self.fill_literal(idx, col, false);
            let mut required_fill = vec![prior_literal, start_literal];
            required_fill.extend_from_slice(&starts);
            the_clauses.push(required_fill);
        }

        the_clauses
    }

    fn col_clauses_block_start_fills(&mut self, col: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for row in 0..self.col_length {
            let start_literal = self.block_start_col_literal(row, col, block_idx, false);
            let must_fill_literal = self.fill_literal(row, col, true);
            let clause = vec![start_literal, must_fill_literal];
            clauses.push(clause);
            if row > 0 {
                let no_fill_literal = self.fill_literal(row - 1, col, false);
                let clause = vec![start_literal, no_fill_literal];
                clauses.push(clause);
            }
        }

        clauses
    }

    fn col_clauses_block_fill(&mut self, col: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for row in 0..self.col_length {
            let start_literal = self.block_start_col_literal(row, col, block_idx, false);
            for length in 2..(self.col_length - row) {
                let length_literal = self.block_length_col_literal(col, block_idx, length, false);
                for offset in 0..length {
                    let fill_literal = self.fill_literal(row + offset, col, true);
                    let clause = vec![start_literal, length_literal, fill_literal];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_length(&mut self, col: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for start_row in 0..self.col_length {
            let start_block_literal =
                self.block_start_col_literal(start_row, col, block_idx, false);

            for length in 1..=(self.col_length - start_row) {
                let mut the_clause = vec![start_block_literal];
                for offset in 1..length {
                    let fill_literal = self.fill_literal(start_row + offset, col, false);
                    the_clause.push(fill_literal);
                }

                if start_row + length < self.col_length {
                    let length_literal = self.fill_literal(start_row + length, col, true);
                    the_clause.push(length_literal);
                }

                let length_literal = self.block_length_col_literal(col, block_idx, length, true);
                the_clause.push(length_literal);
                clauses.push(the_clause);
            }
        }

        clauses
    }

    fn col_clauses_block_starts_somewhere(&mut self, col: usize, block_idx: usize) -> CClause {
        let mut literals = vec![];

        for row in 0..self.col_length {
            literals.push(self.block_start_col_literal(row, col, block_idx, true));
        }

        literals
    }

    fn col_clauses_block_start_unique_row(&mut self, col: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for row_idx in 0..self.col_length {
            let block_start_row = self.block_start_col_literal(row_idx, col, block_idx, false);
            for other_row_idx in 0..self.col_length {
                if row_idx != other_row_idx {
                    let block_start_other_row =
                        self.block_start_col_literal(other_row_idx, col, block_idx, false);
                    let clause = vec![block_start_row, block_start_other_row];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_length_unique(&mut self, col: usize, block_idx: usize) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for length in 1..=self.col_length {
            let block_length_literal = self.block_length_col_literal(col, block_idx, length, false);
            for other_length in 1..=self.col_length {
                if length != other_length {
                    let other_block_length_literal =
                        self.block_length_col_literal(col, block_idx, other_length, false);
                    let clause = vec![block_length_literal, other_block_length_literal];
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_start_unique_idx(
        &mut self,
        col: usize,
        total_blocks: usize,
    ) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for row_idx in 0..self.col_length {
            for block_idx in 0..total_blocks {
                let block_idx_literal =
                    self.block_start_col_literal(row_idx, col, block_idx, false);
                for other_block_idx in 0..total_blocks {
                    if block_idx != other_block_idx {
                        let other_block_idx_literal =
                            self.block_start_col_literal(row_idx, col, other_block_idx, false);
                        let clause = vec![block_idx_literal, other_block_idx_literal];
                        clauses.push(clause);
                    }
                }
            }
        }

        clauses
    }

    fn col_clauses_precedence(
        &mut self,
        col: usize,
        block_a_position: usize,
        block_b_position: usize,
    ) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for excluded in 0..2 {
            clauses.push(vec![self.block_start_col_literal(
                col,
                excluded,
                block_b_position,
                false,
            )]);
        }

        for row_idx in 2..self.col_length {
            let mut literals = vec![];
            let block_b_at_idx =
                self.block_start_col_literal(row_idx, col, block_b_position, false);
            literals.push(block_b_at_idx);
            for prior_row in 0..row_idx {
                let block_a_prior =
                    self.block_start_col_literal(prior_row, col, block_a_position, true);
                literals.push(block_a_prior);
            }
            clauses.push(literals);
        }

        clauses
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_range_loop)]
    fn col_clauses(&mut self) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        for (col_idx, col) in self.cols.clone().iter().enumerate() {
            if col.is_empty() {
                continue;
            }

            let mut col_clauses = vec![];
            col_clauses.append(&mut self.col_clauses_block_start(col_idx, col.len()));
            col_clauses.append(&mut self.col_clauses_block_start_unique_idx(col_idx, col.len()));

            clauses.append(&mut col_clauses);

            for block_idx in 0..col.len() {
                let mut block_clauses = vec![];
                block_clauses.push(self.col_clauses_block_starts_somewhere(col_idx, block_idx));
                block_clauses.push(vec![self.block_length_col_literal(
                    col_idx,
                    block_idx,
                    col[block_idx],
                    true,
                )]);

                block_clauses.append(&mut self.col_clauses_block_start_fills(col_idx, block_idx));
                block_clauses.append(&mut self.col_clauses_block_length(col_idx, block_idx));

                block_clauses.append(&mut self.col_clauses_block_fill(col_idx, block_idx));
                block_clauses.append(&mut self.col_clauses_block_start_unique_row(col_idx, block_idx));
                block_clauses.append(&mut self.col_clauses_block_length_unique(col_idx, block_idx));

                clauses.append(&mut block_clauses);
            }

            let mut precedence_clauses = vec![];
            for block_idx in 0..col.len() - 1 {
                let mut pair_clauses = self.col_clauses_precedence(col_idx, block_idx, block_idx + 1);
                precedence_clauses.append(&mut pair_clauses);
            }
            clauses.append(&mut precedence_clauses);
        }

        clauses
    }

    fn clauses(&mut self) -> Vec<CClause> {
        let mut clauses: Vec<CClause> = vec![];

        clauses.append(&mut self.row_clauses());
        clauses.append(&mut self.col_clauses());

        clauses
    }
}
