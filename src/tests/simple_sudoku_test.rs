#[cfg(test)]
mod tests {
    use std::{ io::{ stdout, BufRead, Write }, sync::{ Arc, Mutex } };

    use crate::simple_sudoku::{ Sudoku, SudokuDifficulty };

    #[test]
    fn test_parse_file() {
        let parsed_sudoku = {
            let temp = Sudoku::parse_file("sudoku-3-64-9.txt");
            if let Err(error) = temp {
                panic!("{}", error);
            }
            temp.unwrap()
        };
        println!("parsed_sudoku: \n{}", parsed_sudoku);

        let mut expected_sudoku = Sudoku::new(3);
        expected_sudoku.set_value(0, 0, 6).unwrap();
        expected_sudoku.set_value(2, 0, 2).unwrap();
        expected_sudoku.set_value(4, 0, 5).unwrap();
        expected_sudoku.set_value(5, 1, 4).unwrap();
        expected_sudoku.set_value(7, 1, 3).unwrap();
        expected_sudoku.set_value(0, 3, 4).unwrap();
        expected_sudoku.set_value(1, 3, 3).unwrap();
        expected_sudoku.set_value(5, 3, 8).unwrap();
        expected_sudoku.set_value(1, 4, 1).unwrap();
        expected_sudoku.set_value(6, 4, 2).unwrap();
        expected_sudoku.set_value(6, 5, 7).unwrap();
        expected_sudoku.set_value(0, 6, 5).unwrap();
        expected_sudoku.set_value(3, 6, 2).unwrap();
        expected_sudoku.set_value(4, 6, 7).unwrap();
        expected_sudoku.set_value(7, 7, 8).unwrap();
        expected_sudoku.set_value(8, 7, 1).unwrap();
        expected_sudoku.set_value(3, 8, 6).unwrap();
        println!("expected_sudoku: \n{}", expected_sudoku);

        assert!(parsed_sudoku.eq(&expected_sudoku));
    }

    #[test]
    fn rule_solving() {
        let files: std::fs::ReadDir = std::fs::read_dir("res/sudoku_samples").unwrap();
        println!("collecting sudokus");
        let sudokus: Vec<String> = files
            .map(|file| {
                file.unwrap().path().file_name().unwrap().to_os_string().into_string().unwrap()
            })
            .take(1000)
            .collect();

        println!("collected sudokus");

        let n_finished: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let mut join_handles = Vec::new();

        for file_name in sudokus {
            if !file_name.starts_with("sudoku-3") || !file_name.ends_with(".txt") {
                continue;
            }
            let sudoku_result = Sudoku::parse_file(&file_name);
            if let Err(error) = sudoku_result {
                eprintln!("Error while parsing file {}: \n{}", file_name, error);
                continue;
            }

            let mut sudoku = sudoku_result.unwrap();
            let thread_n_finished = Arc::clone(&n_finished);
            let join_handle = std::thread::spawn(move || {
                let mut sudoku_rule_usage: Vec<usize> = vec![0; Sudoku::RULES.len()];
                loop {
                    match sudoku.rule_solve(None, None) {
                        Ok(None) => {
                            break;
                        }
                        Ok(Some(rule_used)) => {
                            sudoku_rule_usage[rule_used] += 1;
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            break;
                        }
                    }
                }
                let mut n_finished = thread_n_finished.lock().unwrap();
                *n_finished += 1;

                println!(
                    "sudoku {} {} • {}",
                    file_name,
                    if sudoku.is_filled() {
                        "solved"
                    } else {
                        "unsolved"
                    },
                    n_finished
                );
                (file_name, sudoku.is_filled(), sudoku.get_difficulty(), sudoku_rule_usage)
            });

            join_handles.push(join_handle);
        }

        let mut sudoku_solved: Vec<(String, SudokuDifficulty)> = Vec::new();
        let mut sudoku_unsolved: Vec<String> = Vec::new();
        let mut sudoku_rules_usage: Vec<usize> = vec![0; Sudoku::RULES.len()];

        for join_handle in join_handles {
            let (file_name, solved, difficulty, rule_usage) = join_handle.join().unwrap();
            if solved {
                sudoku_solved.push((file_name, difficulty));
            } else {
                sudoku_unsolved.push(file_name);
            }
            for (rule_id, usage) in rule_usage.into_iter().enumerate() {
                sudoku_rules_usage[rule_id] += usage;
            }
        }

        sudoku_solved.sort_by(|(_, d1), (_, d2)| d1.cmp(d2));

        println!(
            "solved sudokus:\n{}",
            sudoku_solved
                .iter()
                .map(|(file_name, difficulty)| format!("{}: difficulty {}", file_name, difficulty))
                .collect::<Vec<String>>()
                .join("\n")
        );
        println!("\nunsolved sudokus: {}", sudoku_unsolved.join(", "));
        println!(
            "\n{}/{} sudoku solved",
            sudoku_solved.len(),
            sudoku_solved.len() + sudoku_unsolved.len()
        );

        println!(
            "used rules: \n{}",
            sudoku_rules_usage
                .into_iter()
                .enumerate()
                .map(|(rule_id, n)| format!("rule {} used {} times", rule_id, n))
                .collect::<Vec<String>>()
                .join("\n")
        );
    }

    #[test]
    fn canonize_randomize() {
        for i in 0..100 {
            print!(" {i} \r");
            stdout().flush().unwrap();

            let original = Sudoku::generate_full(3);
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

    #[test]
    #[cfg(feature = "database")]
    fn to_from_db() {
        let filled1 = Sudoku::generate_full(3);
        let (db_canonical_sudoku, _db_canonical_squares) = filled1.filled_to_db().unwrap();
        let filled2 = Sudoku::db_from_filled(db_canonical_sudoku.clone());

        if filled1.ne(&filled2) {
            panic!("filled_to_db or db_from_filled : {filled1}\n!=\n{filled2}");
        }

        for difficulty in SudokuDifficulty::iter() {
            let game1 = filled1.generate_from(difficulty);
            let db_game = game1.game_to_db().unwrap();
            let game2 = Sudoku::db_from_game(db_game, db_canonical_sudoku.clone());
            if game1.ne(&game2) {
                panic!("game_to_db or db_from_game : {game1}\n!=\n{game2}");
            }
        }
    }

    #[test]
    #[ignore = "test too long: run it with `cargo test -- tests::simple_sudoku_test::tests::rule_analyse --exact --nocapture --ignored`"]
    fn rule_analyse() {
        let sudoku_data = std::fs::File::open("res/sudoku_cluewise.csv");
        if sudoku_data.is_err() {
            panic!(
                "\n\n#################### MESSAGE DE LEO ####################\nres/sudoku_cluewise.csv not found ! Try executing: \ncurl -L -o res/sudoku_cluewise.zip https://www.kaggle.com/api/v1/datasets/download/informoney/4-million-sudoku-puzzles-easytohard && unzip res/sudoku_data.zip -d res\n#################### MESSAGE DE LEO ####################\n\n"
            );
        }
        let sudoku_data = sudoku_data.unwrap();

        let max_sudoku_per_zeros = 10000;
        let mut reader = std::io::BufReader::new(sudoku_data);
        let mut line = String::new();

        // skip the header
        reader.read_line(&mut line).unwrap();
        line.clear();

        let n_finished: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let mut join_handles = Vec::new();

        let mut total_zeroes: Vec<usize> = vec![0; 82];
        while let Ok(len) = reader.read_line(&mut line) {
            if len == 0 {
                break;
            }
            let zeros = line
                .chars()
                .take(81)
                .filter(|char| char.eq(&'0'))
                .count();
            if total_zeroes[zeros] >= max_sudoku_per_zeros {
                line.clear();
                continue;
            }

            total_zeroes[zeros] += 1;

            let sudoku_string = line
                .split_at(81)
                .0.chars()
                .enumerate()
                .flat_map(|(index, char)| {
                    if index % 9 == 0 {
                        vec!['\n', char].into_iter()
                    } else {
                        vec![' ', char].into_iter()
                    }
                })
                .collect::<String>();
            let sudoku_string = format!("3{sudoku_string}\n");

            let mut sudoku = Sudoku::parse_string(&sudoku_string).unwrap();
            let thread_n_finished = Arc::clone(&n_finished);

            let join_handle = std::thread::spawn(move || {
                let mut sudoku_rule_usage: Vec<
                    (bool, u128)
                > = vec![(false, 0); Sudoku::RULES.len()];
                loop {
                    let start = std::time::Instant::now();
                    let rule_solve_result = sudoku.rule_solve(None, None);
                    let elapsed = start.elapsed().as_millis();
                    match rule_solve_result {
                        Ok(None) => {
                            break;
                        }
                        Ok(Some(rule_used)) => {
                            sudoku_rule_usage[rule_used].0 = true;
                            sudoku_rule_usage[rule_used].1 += elapsed;
                        }
                        Err(_) => {
                            eprintln!("\nError: sudoku isn't valid: \n{sudoku_string}\n");
                            return (false, sudoku_rule_usage);
                        }
                    }
                }
                let mut n_finished = thread_n_finished.lock().unwrap();
                *n_finished += 1;

                print!("{} sudoku • {}\r", n_finished, if sudoku.is_filled() {
                    "solved"
                } else {
                    "unsolved"
                });
                (sudoku.is_filled(), sudoku_rule_usage)
            });

            join_handles.push(join_handle);
            line.clear();
        }

        println!("\nwaiting for threads");

        let mut sudoku_solved: usize = 0;
        let mut sudoku_total: usize = 0;
        let mut sudoku_rules_info: Vec<(u128, u128)> = vec![(0, 0); Sudoku::RULES.len()];

        for join_handle in join_handles {
            let (solved, rule_usage) = join_handle.join().unwrap();
            sudoku_total += 1;
            if solved {
                sudoku_solved += 1;
            }
            for (rule_id, (used, total_time)) in rule_usage.into_iter().enumerate() {
                if used {
                    sudoku_rules_info[rule_id].0 += 1;
                }
                sudoku_rules_info[rule_id].1 += total_time;
            }
        }
        println!();

        let mut sudoku_rules_info: Vec<_> = sudoku_rules_info.into_iter().enumerate().collect();
        sudoku_rules_info.sort_by(|(_, (usage1, _)), (_, (usage2, _))| usage2.cmp(usage1));

        let trailing_zeroes = sudoku_rules_info[0].1.0.to_string().len() + 1;

        println!(
            "rules info: \n{}",
            sudoku_rules_info
                .into_iter()
                .map(|(rule_id, (n, time))|
                    format!(
                        "rule {:2}: used {:0>trailing_zeroes$} times ({:.3}ms avg)",
                        rule_id,
                        n,
                        (time as f64) / (n as f64)
                    )
                )
                .collect::<Vec<String>>()
                .join("\n")
        );

        println!(
            "\n{}/{} = {:.3}% sudoku solved",
            sudoku_solved,
            sudoku_total,
            (100.0 * (sudoku_solved as f64)) / (sudoku_total as f64)
        );
    }

    #[test]
    #[ignore = "test too long: run it with `cargo test -- tests::simple_sudoku_test::tests::generate --exact --nocapture --ignored`"]
    fn generate() {
        let mut time_samples = SudokuDifficulty::iter()
            .map(|diff| (diff, Vec::new()))
            .collect::<Vec<_>>();
        let iterations: usize = 1000;

        let end_function = |time_samples: Vec<(SudokuDifficulty, Vec<u128>)>, iterations: usize| {
            for (difficulty, mut samples) in time_samples {
                samples.sort();

                let min = samples.first().unwrap_or(&0);
                let max = samples.last().unwrap_or(&0);

                let average = (samples.iter().sum::<u128>() as f32) / (iterations as f32);
                let median = samples.get(samples.len() / 2).unwrap_or(&0);

                println!(
                    "Difficulty {}:\n\tmin: {}ms\n\tmax: {}ms\n\taverage {:.2} ms\n\tmedian: {}ms",
                    difficulty,
                    min,
                    max,
                    average,
                    median
                );
            }
        };

        for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
            println!("testing difficulty {difficulty}");

            for j in 0..iterations {
                println!("iteration {j}: ");

                let start = std::time::Instant::now();
                let _sudoku = Sudoku::generate_new(3, difficulty);
                time_samples[i].1.push(start.elapsed().as_millis());
            }
        }

        end_function(time_samples, iterations);
    }
}
