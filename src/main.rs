#![allow(dead_code)] // no warning due to unused code

use hai606i_sudoku::{
    carpet_sudoku::{ CarpetPattern, CarpetSudoku },
    simple_sudoku::SudokuDifficulty,
};
use rand::{ thread_rng, Rng };

fn main() {
    loop {
        let n = 3;
        let original_carpet = CarpetSudoku::generate_full(n, CarpetPattern::Diagonal(2));
        println!("\n\nORIGINAL_CARPET: \n{original_carpet}");

        let mut unsolved_carpet;
        loop {
            unsolved_carpet = original_carpet.clone();
            let mut rng = thread_rng();
            for _ in 0..50 {
                let mut sudoku_id = rng.gen_range(0..unsolved_carpet.get_n_sudokus());
                let mut x = rng.gen_range(0..n * n);
                let mut y = rng.gen_range(0..n * n);
                loop {
                    if unsolved_carpet.get_cell_value(sudoku_id, x, y) == 0 {
                        sudoku_id = rng.gen_range(0..unsolved_carpet.get_n_sudokus());
                        x = rng.gen_range(0..n * n);
                        y = rng.gen_range(0..n * n);
                        continue;
                    }
                    break;
                }
                unsolved_carpet.remove_value(sudoku_id, x, y).unwrap();
            }

            if unsolved_carpet.is_unique() {
                break;
            }
        }
        println!("\n\nUNSOLVED_CARPET: \n{unsolved_carpet}");

        let mut solved_carpet = unsolved_carpet.clone();
        loop {
            match solved_carpet.rule_solve(None) {
                Ok((false, _)) => {
                    break;
                }
                Ok((true, _)) => (),
                Err(err) => {
                    eprintln!("\n\nERR: \n{err}");
                    break;
                }
            }
        }
        println!("\n\nSOLVED_CARPET: \n{solved_carpet}");

        if original_carpet.ne(&solved_carpet) {
            panic!();
        }
    }
}
