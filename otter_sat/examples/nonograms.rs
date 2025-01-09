/// A nonogram solver.
///
/// Given the representation of a nonogram a cnf fomula is built and passed to an otter_sat.
/// A satisfying valuation is a solution to the puzzle.
/// If a solution, the valuation is parsed, and the completed nonogram is displayed.
use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::Solve::{self},
};

/// A struct to hold the nonogram puzzle, clause generation follows the main function.
struct Nonogram {
    row_length: usize,
    col_length: usize,
    rows: Vec<Vec<usize>>,
    cols: Vec<Vec<usize>>,
}

fn main() {
    let config = Config::default();
    let mut the_context = Context::from_config(config, None);

    // https://en.wikipedia.org/wiki/Nonogram#/media/File:Nonogram_wiki.svg
    let puzzle = Nonogram {
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
    };

    for clause in puzzle.clauses() {
        let as_a_clause = the_context.clause_from_string(&clause).unwrap();
        let _ = the_context.add_clause(as_a_clause);
    }

    let clause_count = the_context.clause_db.total_clause_count();
    println!("Context build with {clause_count} clauses",);

    let result = the_context.solve().unwrap();

    if matches!(result, Solve::Unsatisfiable) {
        println!("No solution");
        std::process::exit(0);
    }
    println!(
        "Solution:
"
    );

    let valuation = the_context.atom_db.valuation_string();
    let atoms = valuation.split(" ");

    let mut display = vec![vec![0; puzzle.row_length]; puzzle.col_length];

    for atom in atoms {
        if atom.starts_with("Fill") {
            let idx = &atom[5..atom.len() - 1]
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
                _ => panic!("!"),
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
    fn fill_literal(row: usize, col: usize, polarity: bool) -> String {
        match polarity {
            true => format!("Fill({row},{col})"),
            false => format!("-Fill({row},{col})"),
        }
    }

    fn block_start_row_literal(row: usize, col: usize, block_idx: usize, polarity: bool) -> String {
        match polarity {
            true => format!("BlockStartRow({row},{col},{block_idx})"),
            false => format!("-BlockStartRow({row},{col},{block_idx})"),
        }
    }

    fn block_start_col_literal(row: usize, col: usize, block_idx: usize, polarity: bool) -> String {
        match polarity {
            true => format!("BlockStartCol({row},{col},{block_idx})"),
            false => format!("-BlockStartCol({row},{col},{block_idx})"),
        }
    }

    fn block_legnth_row_literal(
        row: usize,
        block_idx: usize,
        length: usize,
        polarity: bool,
    ) -> String {
        match polarity {
            true => format!("BlockLengthRow({row},{block_idx},{length})"),
            false => format!("-BlockLengthRow({row},{block_idx},{length})"),
        }
    }

    fn block_length_col_literal(
        col: usize,
        block_idx: usize,
        length: usize,
        polarity: bool,
    ) -> String {
        match polarity {
            true => format!("BlockLengthCol({col},{block_idx},{length})"),
            false => format!("-BlockLengthCol({col},{block_idx},{length})"),
        }
    }
}

impl Nonogram {
    fn row_clauses_block_start(&self, row: usize, total_blocks: usize) -> Vec<String> {
        let mut the_clauses = vec![];

        let mut starts = vec![];
        for block_idx in 0..total_blocks {
            starts.push(Nonogram::block_start_row_literal(row, 0, block_idx, true));
        }

        let start_literal = Nonogram::fill_literal(row, 0, false);
        let required_fill = format!("{start_literal} {}", &starts.join(" "));
        the_clauses.push(required_fill);

        for idx in 1..self.row_length {
            let mut starts = vec![];
            for block_idx in 0..total_blocks {
                starts.push(Nonogram::block_start_row_literal(row, idx, block_idx, true));
            }
            let prior_literal = Nonogram::fill_literal(row, idx - 1, true);
            let start_literal = Nonogram::fill_literal(row, idx, false);
            let required_fill = format!("{prior_literal} {start_literal} {}", &starts.join(" "));
            the_clauses.push(required_fill);
        }

        the_clauses
    }

    fn row_clauses_block_start_fills(&self, row: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for col in 0..self.row_length {
            let start_literal = Nonogram::block_start_row_literal(row, col, block_idx, false);
            let must_fill_literal = Nonogram::fill_literal(row, col, true);
            let clause = format!("{start_literal} {must_fill_literal}");
            clauses.push(clause);
            if col > 0 {
                let no_fill_literal = Nonogram::fill_literal(row, col - 1, false);
                let clause = format!("{start_literal} {no_fill_literal}");
                clauses.push(clause);
            }
        }

        clauses
    }

    fn row_clauses_block_fill(&self, row: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for col in 0..self.row_length {
            let start_literal = Nonogram::block_start_row_literal(row, col, block_idx, false);
            for length in 2..(self.row_length - col) {
                let length_literal =
                    Nonogram::block_legnth_row_literal(row, block_idx, length, false);
                for offest in 0..length {
                    let fill_literal = Nonogram::fill_literal(row, col + offest, true);
                    let clause = format!("{start_literal} {length_literal} {fill_literal}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_length(&self, row: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for start_col in 0..self.row_length {
            let start_block_literal =
                Nonogram::block_start_row_literal(row, start_col, block_idx, false);

            for length in 1..=(self.row_length - start_col) {
                let mut the_clause = start_block_literal.clone();
                for offset in 1..length {
                    the_clause.push(' ');
                    let fill_literal = Nonogram::fill_literal(row, start_col + offset, false);
                    the_clause.push_str(&fill_literal);
                }

                if start_col + length < self.row_length {
                    the_clause.push(' ');
                    let length_literal = Nonogram::fill_literal(row, start_col + length, true);
                    the_clause.push_str(&length_literal);
                }
                the_clause.push(' ');
                let length_literal =
                    Nonogram::block_legnth_row_literal(row, block_idx, length, true);
                the_clause.push_str(&length_literal);
                clauses.push(the_clause);
            }
        }

        clauses
    }

    fn row_clauses_block_starts_somewhere(&self, row: usize, block_idx: usize) -> String {
        let mut literals = vec![];

        for col in 0..self.row_length {
            literals.push(Nonogram::block_start_row_literal(row, col, block_idx, true));
        }

        literals.join(" ")
    }

    fn row_clauses_block_start_unique_col(&self, row: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for col_idx in 0..self.row_length {
            let block_start_col = Nonogram::block_start_row_literal(row, col_idx, block_idx, false);
            for other_col_idx in 0..self.row_length {
                if col_idx != other_col_idx {
                    let block_start_other_col =
                        Nonogram::block_start_row_literal(row, other_col_idx, block_idx, false);
                    let clause = format!("{block_start_col} {block_start_other_col}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_length_unique(&self, row: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for length in 1..=self.row_length {
            let block_length_literal =
                Nonogram::block_legnth_row_literal(row, block_idx, length, false);
            for other_length in 1..=self.row_length {
                if length != other_length {
                    let other_block_length_literal =
                        Nonogram::block_legnth_row_literal(row, block_idx, other_length, false);
                    let clause = format!("{block_length_literal} {other_block_length_literal}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn row_clauses_block_start_unique_idx(&self, row: usize, total_blocks: usize) -> Vec<String> {
        let mut clauses = vec![];

        for col_idx in 0..self.row_length {
            for block_idx in 0..total_blocks {
                let block_literal =
                    Nonogram::block_start_row_literal(row, col_idx, block_idx, false);
                for other_idx in 0..total_blocks {
                    if block_idx != other_idx {
                        let other_literal =
                            Nonogram::block_start_row_literal(row, col_idx, other_idx, false);
                        let clause = format!("{block_literal} {other_literal}");
                        clauses.push(clause);
                    }
                }
            }
        }

        clauses
    }

    fn row_clauses_precedence(
        &self,
        row: usize,
        block_a_idx: usize,
        block_b_idx: usize,
    ) -> Vec<String> {
        let mut clauses = vec![];

        for col_idx in 0..2 {
            clauses.push(Nonogram::block_start_row_literal(
                row,
                col_idx,
                block_b_idx,
                false,
            ));
        }

        for col_idx in 2..self.row_length {
            let mut literals = vec![];
            let block_b_at_idx =
                Nonogram::block_start_row_literal(row, col_idx, block_b_idx, false);
            literals.push(block_b_at_idx);
            for prior_col in 0..col_idx {
                let block_a_prior =
                    Nonogram::block_start_row_literal(row, prior_col, block_a_idx, true);
                literals.push(block_a_prior);
            }
            clauses.push(literals.join(" "));
        }

        clauses
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_range_loop)]
    fn row_clauses(&self) -> Vec<String> {
        let mut clauses = vec![];

        for (row_idx, row) in self.rows.iter().enumerate() {
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
                block_clauses.push(Nonogram::block_legnth_row_literal(
                    row_idx,
                    block_idx,
                    row[block_idx],
                    true,
                ));

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
    fn col_clauses_block_start(&self, col: usize, total_blocks: usize) -> Vec<String> {
        let mut the_clauses = vec![];

        let mut starts = vec![];
        for block_idx in 0..total_blocks {
            starts.push(Nonogram::block_start_col_literal(0, col, block_idx, true));
        }

        let start_literal = Nonogram::fill_literal(0, col, false);
        let required_fill = format!("{start_literal} {}", &starts.join(" "));
        the_clauses.push(required_fill);

        for idx in 1..self.col_length {
            let mut starts = vec![];
            for block_idx in 0..total_blocks {
                starts.push(Nonogram::block_start_col_literal(idx, col, block_idx, true));
            }
            let prior_literal = Nonogram::fill_literal(idx - 1, col, true);
            let start_literal = Nonogram::fill_literal(idx, col, false);
            let required_fill = format!("{prior_literal} {start_literal} {}", &starts.join(" "));
            the_clauses.push(required_fill);
        }

        the_clauses
    }

    fn col_clauses_block_start_fills(&self, col: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for row in 0..self.col_length {
            let start_literal = Nonogram::block_start_col_literal(row, col, block_idx, false);
            let must_fill_literal = Nonogram::fill_literal(row, col, true);
            let clause = format!("{start_literal} {must_fill_literal}");
            clauses.push(clause);
            if row > 0 {
                let no_fill_literal = Nonogram::fill_literal(row - 1, col, false);
                let clause = format!("{start_literal} {no_fill_literal}");
                clauses.push(clause);
            }
        }

        clauses
    }

    fn col_clauses_block_fill(&self, col: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for row in 0..self.col_length {
            let start_literal = Nonogram::block_start_col_literal(row, col, block_idx, false);
            for length in 2..(self.col_length - row) {
                let length_literal =
                    Nonogram::block_length_col_literal(col, block_idx, length, false);
                for offset in 0..length {
                    let fill_literal = Nonogram::fill_literal(row + offset, col, true);
                    let clause = format!("{start_literal} {length_literal} {fill_literal}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_length(&self, col: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for start_row in 0..self.col_length {
            let start_block_literal =
                Nonogram::block_start_col_literal(start_row, col, block_idx, false);

            for length in 1..=(self.col_length - start_row) {
                let mut the_clause = start_block_literal.clone();
                for offset in 1..length {
                    the_clause.push(' ');
                    let fill_literal = Nonogram::fill_literal(start_row + offset, col, false);
                    the_clause.push_str(&fill_literal);
                }

                if start_row + length < self.col_length {
                    the_clause.push(' ');
                    let length_literal = Nonogram::fill_literal(start_row + length, col, true);
                    the_clause.push_str(&length_literal);
                }
                the_clause.push(' ');
                let length_literal =
                    Nonogram::block_length_col_literal(col, block_idx, length, true);
                the_clause.push_str(&length_literal);
                clauses.push(the_clause);
            }
        }

        clauses
    }

    fn col_clauses_block_starts_somewhere(&self, col: usize, block_idx: usize) -> String {
        let mut literals = vec![];

        for row in 0..self.col_length {
            literals.push(Nonogram::block_start_col_literal(row, col, block_idx, true));
        }

        literals.join(" ")
    }

    fn col_clauses_block_start_unique_row(&self, col: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for row_idx in 0..self.col_length {
            let block_start_row = Nonogram::block_start_col_literal(row_idx, col, block_idx, false);
            for other_row_idx in 0..self.col_length {
                if row_idx != other_row_idx {
                    let block_start_other_row =
                        Nonogram::block_start_col_literal(other_row_idx, col, block_idx, false);
                    let clause = format!("{block_start_row} {block_start_other_row}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_length_unique(&self, col: usize, block_idx: usize) -> Vec<String> {
        let mut clauses = vec![];

        for length in 1..=self.col_length {
            let block_length_literal =
                Nonogram::block_length_col_literal(col, block_idx, length, false);
            for other_length in 1..=self.col_length {
                if length != other_length {
                    let other_block_length_literal =
                        Nonogram::block_length_col_literal(col, block_idx, other_length, false);
                    let clause = format!("{block_length_literal} {other_block_length_literal}");
                    clauses.push(clause);
                }
            }
        }

        clauses
    }

    fn col_clauses_block_start_unique_idx(&self, col: usize, total_blocks: usize) -> Vec<String> {
        let mut clauses = vec![];

        for row_idx in 0..self.col_length {
            for block_idx in 0..total_blocks {
                let block_idx_literal =
                    Nonogram::block_start_col_literal(row_idx, col, block_idx, false);
                for other_block_idx in 0..total_blocks {
                    if block_idx != other_block_idx {
                        let other_block_idx_literal =
                            Nonogram::block_start_col_literal(row_idx, col, other_block_idx, false);
                        let clause = format!("{block_idx_literal} {other_block_idx_literal}");
                        clauses.push(clause);
                    }
                }
            }
        }

        clauses
    }

    fn col_clauses_precedence(
        &self,
        col: usize,
        block_a_position: usize,
        block_b_position: usize,
    ) -> Vec<String> {
        let mut clauses = vec![];

        for excluded in 0..2 {
            clauses.push(Nonogram::block_start_col_literal(
                col,
                excluded,
                block_b_position,
                false,
            ));
        }

        for row_idx in 2..self.col_length {
            let mut literals = vec![];
            let block_b_at_idx =
                Nonogram::block_start_col_literal(row_idx, col, block_b_position, false);
            literals.push(block_b_at_idx);
            for prior_row in 0..row_idx {
                let block_a_prior =
                    Nonogram::block_start_col_literal(prior_row, col, block_a_position, true);
                literals.push(block_a_prior);
            }
            clauses.push(literals.join(" "));
        }

        clauses
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_range_loop)]
    fn col_clauses(&self) -> Vec<String> {
        let mut clauses = vec![];

        for (col_idx, col) in self.cols.iter().enumerate() {
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
                block_clauses.push(Nonogram::block_length_col_literal(
                    col_idx,
                    block_idx,
                    col[block_idx],
                    true,
                ));

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

    fn clauses(&self) -> Vec<String> {
        let mut clauses = vec![];

        clauses.append(&mut self.row_clauses());
        clauses.append(&mut self.col_clauses());

        clauses
    }
}
