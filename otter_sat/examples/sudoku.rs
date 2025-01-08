/// A solver for sudoku puzzles.
///
/// Puzzles are represented as an array of array of cells, with cells containing either a value [1-9] with '0' used to represent the absence of a value.
use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::Solve::{self},
};

type SudokuGrid = [[usize; 9]; 9];
const GRID_SIZE: usize = 9;

const EMPTY_GRID: SudokuGrid = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
];

fn main() {
    let the_puzzle = [
        [8, 0, 2, 0, 9, 6, 0, 0, 0],
        [0, 0, 5, 0, 1, 8, 0, 3, 0],
        [1, 0, 6, 7, 0, 0, 0, 2, 4],
        [0, 7, 8, 9, 0, 2, 1, 0, 5],
        [0, 0, 0, 1, 0, 5, 6, 0, 3],
        [0, 0, 1, 0, 0, 0, 0, 9, 8],
        [9, 8, 4, 0, 3, 1, 7, 0, 0],
        [2, 5, 0, 0, 4, 9, 0, 8, 0],
        [6, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let config = Config::default();
    let mut the_context: Context = Context::from_config(config, None);

    add_clauses_cell_value_choice(&mut the_context);
    add_clauses_cells_have_unique_value(&mut the_context);
    add_clauses_each_col_has_all_values(&mut the_context);
    add_clauses_each_row_has_all_values(&mut the_context);
    add_clauses_each_subgrid_has_all_values(&mut the_context);
    add_clauses_detailing_puzzle(&mut the_context, the_puzzle);

    match the_context.solve() {
        Ok(_) => {}
        Err(e) => panic!("Solve error: {e:?}"),
    };

    let valuation_string = the_context.atom_db.valuation_string();
    let solution = valuation_to_grid(valuation_string);
    match the_context.report() {
        Solve::Satisfiable => {
            print_sudoku_grid(&solution);
            println!();
            match validate_solution(solution) {
                true => println!("Validation: OK"),
                false => println!("Validation: NOK"),
            }
        }
        Solve::Unsatisfiable => {
            println!("It is not possible to solve the puzzle.")
        }
        _ => {}
    }
}

/// otter_sat supports strings as literals, with few exceptions, so a simple "{row}-{col}-{value}" string is used to represent a cell.
///
/// Note, '-' is safe to use as an infix, but is interpreted as negation when used as a prefix.
fn cell_string(row: usize, col: usize, value: usize) -> String {
    format!("{row}-{col}-{value}")
}

/// A solved sudoku puzzle requires each cell to have some value.
/// This requirement is encoded through disjunctions over values for each row and col pairing.
/// For exmaple, the disjunction for row 1 and col 3 will read 1-3-1 ∨ 1-3-2 ∨ … ∨ 1-3-8 ∨ 1-3-9.
fn add_clauses_cell_value_choice(context: &mut Context) {
    for row in 1..GRID_SIZE + 1 {
        for col in 1..GRID_SIZE + 1 {
            let mut cell_restriction = vec![];
            for value in 1..GRID_SIZE + 1 {
                cell_restriction.push(cell_string(row, col, value));
            }
            let clause_string = cell_restriction.join(" ");
            let clause = context.clause_from_string(&clause_string).unwrap();
            match context.add_clause(clause) {
                Ok(_) => {}
                Err(e) => panic!("Failed to add clause: {e:?}"),
            };
        }
    }
}

/// A solved sudoku puzzle requires each cell to have a unique value.
/// This requirement is encoded through a collection of implications for each cell stating that if the cell has a particular value it does not have any other value.
/// These have the form p → ¬q, which is equivalent to ¬p ∨ ¬q.
/// So, for example, 'if the cell at row 4 and colum 8 has value 2, then the cell does not have value 7' is encoded through the clause: ¬4-8-2 ∨ ¬4-8-7.
fn add_clauses_cells_have_unique_value(context: &mut Context) {
    for row in 1..GRID_SIZE + 1 {
        for col in 1..GRID_SIZE + 1 {
            let mut cell_restriction = vec![];
            for value in 1..GRID_SIZE + 1 {
                cell_restriction.push(cell_string(row, col, value));
                for other_value in 1..GRID_SIZE + 1 {
                    if other_value != value {
                        let clause_string = format!(
                            "-{} -{}",
                            cell_string(row, col, value),
                            cell_string(row, col, other_value)
                        );
                        let clause = context.clause_from_string(&clause_string).unwrap();
                        match context.add_clause(clause) {
                            Ok(_) => {}
                            Err(e) => panic!("Failed to add clause: {e:?}"),
                        };
                    }
                }
            }
        }
    }
}

/// A solved sudoku puzzle requires each row to contain each value in some cell of the row.
/// This requirement is encoded through disjunctions over rows for each col and value pairing.
fn add_clauses_each_row_has_all_values(context: &mut Context) {
    for value in 1..GRID_SIZE + 1 {
        for row in 1..GRID_SIZE + 1 {
            let mut row_value = vec![];
            for col in 1..GRID_SIZE + 1 {
                row_value.push(cell_string(row, col, value));
            }
            let clause = context.clause_from_string(&row_value.join(" ")).unwrap();
            match context.add_clause(clause) {
                Ok(_) => {}
                Err(e) => panic!("Failed to add clause: {e:?}"),
            };
        }
    }
}

/// A solved sudoku puzzle requires each col to contain each value in some cell of the col.
/// This requirement is encoded through disjunctions over rows for each col and value pairing.
fn add_clauses_each_col_has_all_values(context: &mut Context) {
    for value in 1..GRID_SIZE + 1 {
        for col in 1..GRID_SIZE + 1 {
            let mut col_value = vec![];
            for row in 1..GRID_SIZE + 1 {
                col_value.push(cell_string(row, col, value));
            }
            let clause = context.clause_from_string(&col_value.join(" ")).unwrap();
            match context.add_clause(clause) {
                Ok(_) => {}
                Err(e) => panic!("Failed to add clause: {e:?}"),
            };
        }
    }
}

/// A solved sudoku puzzle requires each subgrid to contain each value in some cell of the subgrid.
/// This requirement is encoded through disjunctions over row, col, and value pairing in the subgrid.
fn add_clauses_each_subgrid_has_all_values(context: &mut Context) {
    for value in 1..GRID_SIZE + 1 {
        for sub_grid_r in 0..GRID_SIZE / 3 {
            for sub_grid_c in 0..GRID_SIZE / 3 {
                let mut subgrid_val = vec![];
                for row in 1..(GRID_SIZE / 3) + 1 {
                    for col in 1..(GRID_SIZE / 3) + 1 {
                        subgrid_val.push(cell_string(
                            row + (sub_grid_r * 3),
                            col + (sub_grid_c * 3),
                            value,
                        ));
                    }
                }
                let clause_string = subgrid_val.join(" ");
                let clause = context.clause_from_string(&clause_string).unwrap();
                match context.add_clause(clause) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to add clause: {e:?}"),
                };
            }
        }
    }
}

/// To detail the puzzle, unit clauses are added to represent the existing values.
#[allow(clippy::needless_range_loop)]
fn add_clauses_detailing_puzzle(context: &mut Context, puzzle: SudokuGrid) {
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let value = puzzle[row][col];
            if value != 0 {
                let clause_string = cell_string(row + 1, col + 1, value);
                let clause = context.clause_from_string(&clause_string).unwrap();
                match context.add_clause(clause) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to add clause: {e:?}"),
                };
            }
        }
    }
}

fn print_sudoku_grid(grid: &SudokuGrid) {
    for row in grid {
        let row_string = row
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        println!("{row_string}");
    }
}

fn valuation_to_grid(valuation: String) -> SudokuGrid {
    let mut solution = EMPTY_GRID;
    let cell_info = valuation.split_whitespace().collect::<Vec<_>>();
    for info in cell_info {
        if !info.starts_with('-') {
            let parts = info.split('-').collect::<Vec<_>>();
            let numeric_parts = parts
                .into_iter()
                .map(|s| s.parse::<usize>().unwrap())
                .collect::<Vec<_>>();
            let row = numeric_parts[0] - 1;
            let col = numeric_parts[1] - 1;
            let val = numeric_parts[2];
            solution[row][col] = val;
        }
    }
    solution
}

#[allow(clippy::needless_range_loop)]
fn validate_solution(solution: SudokuGrid) -> bool {
    // Every cell has a value
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            if solution[row][col] == 0 {
                return false;
            }
        }
    }

    for row in solution {
        for value in 1..GRID_SIZE + 1 {
            if !row.iter().any(|cell| *cell == value) {
                return false;
            }
        }
    }

    for col in 0..GRID_SIZE {
        let col_cells = solution.iter().map(|row| row[col]).collect::<Vec<_>>();
        for value in 1..GRID_SIZE + 1 {
            if !col_cells.iter().any(|cell| *cell == value) {
                return false;
            }
        }
    }

    for r in 0..GRID_SIZE / 3 {
        for c in 0..GRID_SIZE / 3 {
            let mut sub_grid_vals = vec![];
            for row in 0..(GRID_SIZE / 3) {
                for col in 0..(GRID_SIZE / 3) {
                    sub_grid_vals.push(solution[row + r * 3][col + c * 3]);
                }
            }
            for value in 1..GRID_SIZE + 1 {
                if !sub_grid_vals.iter().any(|cell| *cell == value) {
                    return false;
                }
            }
        }
    }

    true
}
