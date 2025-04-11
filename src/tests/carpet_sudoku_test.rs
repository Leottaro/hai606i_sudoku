#[cfg(test)]
mod tests {
    use crate::{ carpet_sudoku::CarpetPattern, simple_sudoku::SudokuDifficulty };
    use crate::carpet_sudoku::CarpetSudoku;

    #[test]
    fn canonize_randomize() {
        for pattern in CarpetPattern::iter() {
            for i in 0..10 {
                print!(" {i} \r");
                stdout().flush().unwrap();

                let original = CarpetSudoku::generate_full(3, pattern);
                print!("generated");
                stdout().flush().unwrap();

                let mut randomized = original.clone();
                randomized.randomize(None, None).unwrap();
                print!(", randomized");
                stdout().flush().unwrap();

                let mut canonical = randomized.clone();
                canonical.canonize().unwrap();
                print!(", canonical");
                stdout().flush().unwrap();

                if canonical.ne(&original) {
                    panic!(
                        "PROBLEEEEME: \nORIGINAL:\n{original}\nRANDOMIZED:\n{randomized}\nNORMALIZED:\n{canonical}"
                    );
                }
            }
        }
    }

    #[test]
    #[cfg(feature = "database")]
    fn to_from_db() {
        for pattern in CarpetPattern::iter() {
            let filled1 = CarpetSudoku::generate_full(3, pattern);
            let (db_carpet, db_carpet_sudokus) = filled1.db_to_filled().unwrap();
            let db_sudokus = filled1
                .db_sudokus_to_filled()
                .into_iter()
                .map(|(sudoku, _squares)| sudoku)
                .collect::<Vec<_>>();
            let filled2 = CarpetSudoku::db_from_filled(
                db_carpet.clone(),
                db_sudokus.clone(),
                db_carpet_sudokus.clone()
            );

            if filled1.ne(&filled2) {
                panic!("filled_to_db or db_from_filled : {filled1}\n!=\n{filled2}");
            }

            for difficulty in SudokuDifficulty::iter() {
                let game1 = filled1.generate_from(difficulty);
                let db_game = game1.db_to_game();
                let game2 = CarpetSudoku::db_from_game(
                    db_game,
                    db_carpet.clone(),
                    db_sudokus.clone(),
                    db_carpet_sudokus.clone()
                );
                if game1.ne(&game2) {
                    panic!("game_to_db or db_from_game : {game1}\n!=\n{game2}");
                }
            }
        }
    }

    #[test]
    #[ignore = "test too long: run it with `cargo test -- tests::simple_sudoku_test::tests::generate --exact --nocapture --ignored`"]
    fn generate() {
        let mut time_samples = CarpetPattern::iter()
            .flat_map(|pattern|
                SudokuDifficulty::iter()
                    .map(|diff| (pattern, diff, Vec::new()))
                    .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();
        let iterations: usize = 1000;

        let end_function = |
            time_samples: Vec<(CarpetPattern, SudokuDifficulty, Vec<u128>)>,
            iterations: usize
        | {
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
