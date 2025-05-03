#[cfg(test)]
mod tests {
    use std::io::{stdout, Write};

    use crate::carpet_sudoku::CarpetSudoku;
    use crate::{carpet_sudoku::CarpetPattern, simple_sudoku::SudokuDifficulty};

    #[test]
    fn canonize_randomize() {
        for pattern in CarpetPattern::iter() {
            for i in 0..10 {
                print!("{pattern} {i}: ");
                stdout().flush().unwrap();

                let original = CarpetSudoku::generate_new(3, pattern, SudokuDifficulty::Easy);
                print!("generated");
                stdout().flush().unwrap();

                let mut randomized = original.clone();
                randomized.randomize().unwrap();
                print!(", randomized");
                stdout().flush().unwrap();

                let mut randomized_verify = randomized.clone();
                randomized_verify.rule_solve_until((false, false), None);
                if !randomized_verify.is_filled() {
                    panic!("PROBLEEEEME: COULD NOT SOLVE RANDOMIZED");
                }
                print!(", solved");
                stdout().flush().unwrap();
            }
        }
    }

    #[test]
    #[cfg(feature = "database")]
    fn to_from_db() {
        for pattern in CarpetPattern::iter() {
            println!("\n\n\nPattern: {pattern}");

            let filled1 = CarpetSudoku::generate_full(3, pattern);
            let (db_carpet, db_carpet_sudokus) = filled1.db_to_filled().unwrap();
            let db_sudokus = filled1
                .db_sudokus_to_filled()
                .unwrap()
                .into_iter()
                .map(|(sudoku, _squares)| sudoku)
                .collect::<Vec<_>>();
            let filled2 = CarpetSudoku::db_from_filled(
                db_carpet.clone(),
                db_carpet_sudokus.clone(),
                db_sudokus.clone(),
            );

            if filled1.ne(&filled2) {
                panic!(
                    "\nORIGINAL FILLED {}links: {:?}\n\n!=\n\nRECONSTRUCTED FILLED {}links: {:?}",
                    filled1,
                    filled1.get_links(),
                    filled2,
                    filled2.get_links()
                );
            }

            println!("filled OK");

            for difficulty in SudokuDifficulty::iter() {
                println!("\ndifficulty: {difficulty}");
                let game1 = filled1.generate_from(difficulty).unwrap();
                let db_game = game1.db_to_game();
                let game2 = CarpetSudoku::db_from_game(
                    db_game,
                    db_carpet.clone(),
                    db_carpet_sudokus.clone(),
                    db_sudokus.clone(),
                );
                if game1.ne(&game2) {
                    panic!(
                        "\nORIGINAL GAME {}links: {:?}\n\n!=\n\nRECONSTRUCTED GAME {}links: {:?}",
                        game1,
                        game1.get_links(),
                        game2,
                        game2.get_links()
                    );
                }
                println!("OK");
            }
        }
    }

    #[test]
    #[ignore = "test too long: run it with `cargo test -- tests::carpet_sudoku_test::tests::generate --exact --nocapture --ignored`"]
    fn generate() {
        let mut time_samples = CarpetPattern::iter()
            .flat_map(|pattern| {
                SudokuDifficulty::iter()
                    .map(|diff| (pattern, diff, Vec::new()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let iterations: usize = 1000;

        let end_function = |time_samples: Vec<(CarpetPattern, SudokuDifficulty, Vec<u128>)>,
                            iterations: usize| {
            for (pattern, difficulty, mut samples) in time_samples {
                samples.sort();

                let min = samples.first().unwrap_or(&0);
                let max = samples.last().unwrap_or(&0);

                let average = (samples.iter().sum::<u128>() as f32) / (iterations as f32);
                let median = samples.get(samples.len() / 2).unwrap_or(&0);

                println!(
                    "Pattern: {}, Difficulty {}:\n\tmin: {}ms\n\tmax: {}ms\n\taverage {:.2} ms\n\tmedian: {}ms",
                    pattern,
                    difficulty,
                    min,
                    max,
                    average,
                    median
                );
            }
        };

        for pattern in CarpetPattern::iter() {
            for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
                println!("testing difficulty {difficulty}");

                for j in 0..iterations {
                    println!("iteration {j}: ");

                    let start = std::time::Instant::now();
                    let _sudoku = CarpetSudoku::generate_new(3, pattern, difficulty);
                    time_samples[i].2.push(start.elapsed().as_millis());
                }
            }
        }

        end_function(time_samples, iterations);
    }
}
