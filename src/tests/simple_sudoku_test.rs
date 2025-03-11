#[cfg(test)]
#[allow(dead_code)] // no warning due to unused functions
mod tests {
    use std::{
        fs::{File, ReadDir},
        io::BufReader,
        sync::{Arc, Mutex},
        time::Instant,
    };

    use crate::simple_sudoku::{Sudoku, SudokuDifficulty};
    use std::io::BufRead;

    #[test]
    fn test_parse_file() {
        let parsed_sudoku = {
            let temp = Sudoku::parse_file("sudoku-3-64-9.txt");
            assert!(temp.is_ok());
            temp.unwrap()
        };
        println!("parsed_sudoku: \n{}", parsed_sudoku);
        if let Some(((x1, y1), (x2, y2))) = parsed_sudoku.get_error() {
            println!(
				"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule",
				x1, y1, x2, y2
			);
            panic!();
        } else {
            let mut expected_sudoku = Sudoku::new(3);
            expected_sudoku.set_value(0, 0, 6);
            expected_sudoku.set_value(2, 0, 2);
            expected_sudoku.set_value(4, 0, 5);
            expected_sudoku.set_value(5, 1, 4);
            expected_sudoku.set_value(7, 1, 3);
            expected_sudoku.set_value(0, 3, 4);
            expected_sudoku.set_value(1, 3, 3);
            expected_sudoku.set_value(5, 3, 8);
            expected_sudoku.set_value(1, 4, 1);
            expected_sudoku.set_value(6, 4, 2);
            expected_sudoku.set_value(6, 5, 7);
            expected_sudoku.set_value(0, 6, 5);
            expected_sudoku.set_value(3, 6, 2);
            expected_sudoku.set_value(4, 6, 7);
            expected_sudoku.set_value(7, 7, 8);
            expected_sudoku.set_value(8, 7, 1);
            expected_sudoku.set_value(3, 8, 6);
            println!("expected_sudoku: \n{}", expected_sudoku);

            assert_eq!(parsed_sudoku, expected_sudoku);
        }
    }

    #[test]
    fn rule_solving() {
        let files: ReadDir = std::fs::read_dir("res/sudoku_samples").unwrap();
        println!("collecting sudokus");
        let sudokus: Vec<String> = files
            .map(|file| {
                file.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap()
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
                while sudoku.get_error().is_none() && !sudoku.is_solved() {
                    match sudoku.rule_solve(None, None) {
                        Ok(None) => break,
                        Ok(Some(rule_used)) => sudoku_rule_usage[rule_used] += 1,
                        Err(((x1, y1), (x2, y2))) => {
                            eprintln!("Error: ({}, {}) and ({}, {})", x1, y1, x2, y2);
                            break;
                        }
                    }
                }
                let mut n_finished = thread_n_finished.lock().unwrap();
                *n_finished += 1;

                println!(
                    "sudoku {} {} • {}",
                    file_name,
                    if sudoku.is_solved() {
                        "solved"
                    } else {
                        "unsolved"
                    },
                    n_finished,
                );
                (
                    file_name,
                    sudoku.is_solved(),
                    sudoku.get_difficulty(),
                    sudoku_rule_usage,
                )
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
                .map(|(file_name, difficulty)| format!(
                    "{}: difficulty {:?}",
                    file_name, difficulty
                ))
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
    #[ignore = "test too long: run it with `cargo test -- tests::simple_sudoku_test::tests::rule_analyse --exact --nocapture --ignored`"]
    fn rule_analyse() {
        let sudoku_data = File::open("res/sudoku_cluewise.csv");
        if sudoku_data.is_err() {
            println!("\n\n#################### MESSAGE DE LEO ####################\nres/sudoku_cluewise.csv not found ! Try executing: \ncurl -L -o res/sudoku_cluewise.zip https://www.kaggle.com/api/v1/datasets/download/informoney/4-million-sudoku-puzzles-easytohard && unzip res/sudoku_data.zip -d res\n#################### MESSAGE DE LEO ####################\n\n");
            panic!();
        }
        let sudoku_data = sudoku_data.unwrap();

        let max_sudoku_per_zeros = 10000;
        let mut reader = BufReader::new(sudoku_data);
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
            let zeros = line.chars().take(81).filter(|char| char.eq(&'0')).count();
            if total_zeroes[zeros] >= max_sudoku_per_zeros {
                line.clear();
                continue;
            }

            total_zeroes[zeros] += 1;

            let sudoku_string = line
                .split_at(81)
                .0
                .chars()
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
                let mut sudoku_rule_usage: Vec<(bool, u128)> =
                    vec![(false, 0); Sudoku::RULES.len()];
                while sudoku.get_error().is_none() && !sudoku.is_solved() {
                    let start = Instant::now();
                    let rule_solve_result = sudoku.rule_solve(None, None);
                    let elapsed = start.elapsed().as_millis();
                    match rule_solve_result {
                        Ok(None) => break,
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

                print!(
                    "\r{} sudoku • {}",
                    n_finished,
                    if sudoku.is_solved() {
                        "solved"
                    } else {
                        "unsolved"
                    },
                );
                (sudoku.is_solved(), sudoku_rule_usage)
            });

            join_handles.push(join_handle);
            line.clear();
        }

        println!("\nwaiting for threads",);

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

        let trailing_zeroes = sudoku_rules_info[0].1 .0.to_string().len() + 1;

        println!(
            "rules info: \n{}",
            sudoku_rules_info
                .into_iter()
                .map(|(rule_id, (n, time))| format!(
                    "rule {:2}: used {:0>trailing_zeroes$} times ({:.3}ms avg)",
                    rule_id,
                    n,
                    time as f64 / n as f64
                ))
                .collect::<Vec<String>>()
                .join("\n")
        );

        println!(
            "\n{}/{} = {:.3}% sudoku solved",
            sudoku_solved,
            sudoku_total,
            100. * sudoku_solved as f64 / sudoku_total as f64
        );
    }

    #[test]
    fn generate_full() {
        for size in 2..=4 {
            println!("testing size {size}");
            for _ in 1..=100 {
                let sudoku = Sudoku::generate_full(size);
                assert!(
                    sudoku.is_solved(),
                    "Generated sudoku that was unsolved: \n{sudoku}"
                );
                if let Some(((x1, y1), (x2, y2))) = sudoku.get_error() {
                    panic!(
                        "Gererated an invalid sudoku in ({x1},{y1}) and ({x2},{y2}): \n{sudoku}"
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "test too long: run it with `cargo test -- tests::simple_sudoku_test::tests::generate --exact --nocapture --ignored`"]
    fn generate() {
        let mut time_samples = SudokuDifficulty::iter()
            .map(|diff| (diff, Vec::new()))
            .collect::<Vec<_>>();
        let iterations: usize = 100;

        let end_function = |time_samples: Vec<(SudokuDifficulty, Vec<u128>)>, iterations: usize| {
            for (difficulty, mut samples) in time_samples {
                samples.sort();

                let min = samples.first().unwrap_or(&0);
                let max = samples.last().unwrap_or(&0);

                let average = samples.iter().sum::<u128>() as f32 / iterations as f32;
                let median = samples.get(samples.len() / 2).unwrap_or(&0);

                println!(
                    "Difficulty {}:\n\tmin: {}ms\n\tmax: {}ms\n\taverage {:.2} ms\n\tmedian: {}ms",
                    difficulty, min, max, average, median
                );
            }
        };

        for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
            println!("testing difficulty {difficulty}");

            for j in 0..iterations {
                println!("iteration {j}: ");

                let start = Instant::now();
                let mut original_sudoku = Sudoku::generate(3, difficulty);
                time_samples[i].1.push(start.elapsed().as_millis());

                println!("Solving...");
                let mut sudoku = original_sudoku.clone();
                loop {
                    match sudoku.rule_solve(None, None) {
                        Ok(Some(_)) => (),
                        Ok(None) => {
                            if !sudoku.is_solved() {
                                eprintln!("ERROR IN SUDOKU SOLVING: Couldn't solve generated sudoku: \nORIGINAL SUDOKU:\n{original_sudoku}\nFINISHED SUDOKU: \n{sudoku}");
                                env_logger::Builder::from_env(
                                    env_logger::Env::default().default_filter_or("debug"),
                                )
                                .init();
                                loop {
                                    match original_sudoku.rule_solve(None, None) {
                                        Ok(None) | Err(_) => {
                                            end_function(time_samples, iterations);
                                            panic!();
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            break;
                        }
                        Err(((x1, y1), (x2, y2))) => {
                            eprintln!(
                            "ERROR IN SUDOKU: cells ({x1},{y1}) == ({x2},{y2}): \nORIGINAL SUDOKU:"
                        );
                            loop {
                                match original_sudoku.rule_solve(None, None) {
                                    Ok(None) | Err(_) => {
                                        eprintln!("MAXIMUM SUDOKU:{original_sudoku}");
                                        end_function(time_samples, iterations);
                                        panic!();
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
                println!("Solved !");
            }
        }

        end_function(time_samples, iterations);
    }
}
