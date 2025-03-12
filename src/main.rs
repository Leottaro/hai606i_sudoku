use std::time::Instant;

use macroquad::prelude::*;
use simple_sudoku::{Sudoku, SudokuDifficulty};

mod simple_sudoku;
mod tests;

fn main() {
    let mut average_durations = SudokuDifficulty::iter()
        .map(|diff| (diff, 0.))
        .collect::<Vec<_>>();
    let iterations = 100;

    for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
        let mut total_duration = 0;
        println!("testing difficulty {difficulty}");

        for j in 0..iterations {
            println!("iteration {j}: ");

            let start = Instant::now();
            let original_sudoku = Sudoku::generate(3, difficulty);
            total_duration += start.elapsed().as_millis();

            let mut sudoku = original_sudoku.clone();
            while !sudoku.is_solved() {
                sudoku.rule_solve(None, None).unwrap();
                if let Some(((x1, y1), (x2, y2))) = sudoku.get_error() {
                    eprintln!("ERROR IN SUDOKU: cells ({x1},{y1}) == ({x2},{y2}): \nORIGINAL SUDOKU:\n{original_sudoku}\nFINISHED SUDOKU: \n{sudoku}");
                    panic!();
                }
            }

            if sudoku.is_solved() {
                // println!("ORIGINAL SUDOKU:\n{original_sudoku}\nFINISHED SUDOKU: \n{sudoku}");
                assert_eq!(difficulty, sudoku.get_difficulty());
            } else {
                panic!();
            }
        }

        average_durations[i].1 = total_duration as f64 / iterations as f64;

        println!(
            "Average time for difficulty {}: {:.2} ms",
            difficulty, average_durations[i].1
        );
    }

    for (difficulty, duration) in average_durations {
        println!(
            "Average time for difficulty {}: {:.2} ms",
            difficulty, duration
        );
    }
}
