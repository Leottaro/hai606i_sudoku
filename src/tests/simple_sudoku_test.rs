#[cfg(test)]
mod tests {
    use std::{
        fs::ReadDir,
        sync::{Arc, Mutex},
        time::Instant,
    };

    use crate::simple_sudoku::Sudoku;

    #[test]
    fn test_parse_file() {
        let parsed_sudoku = {
            let temp = Sudoku::parse_file("sudoku-3-64-9.txt");
            assert!(temp.is_ok());
            temp.unwrap()
        };
        println!("parsed_sudoku: \n{}", parsed_sudoku);
        if let Err(((x1, y1), (x2, y2))) = parsed_sudoku.is_valid() {
            println!(
				"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule",
				x1, y1, x2, y2
			);
            assert!(false);
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
        println!("collecting file_names");
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

        println!("collected file_names");

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
                while sudoku.is_valid().is_ok() && !sudoku.is_solved() {
                    match sudoku.rule_solve(None) {
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

        let mut sudoku_solved: Vec<(String, Option<usize>)> = Vec::new();
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

        assert!(true);
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
                if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
                    assert!(
                        false,
                        "Gererated an invalid sudoku in ({x1},{y1}) and ({x2},{y2}): \n{sudoku}"
                    );
                }
            }
        }
        assert!(true);
    }

    #[test]
    fn generate() {
        for difficulty in 0..Sudoku::RULES.len() {
            let mut total_duration = 0;
            let iterations = 100;

            println!("testing difficulty {difficulty}");
            eprintln!("testing difficulty {difficulty}");

            for i in 0..iterations {
                println!("iteration {i}");
                eprint!("iteration {i}: ");
                let start = Instant::now();
                let mut sudoku = Sudoku::generate(3, difficulty);
                let duration = start.elapsed();
                total_duration += duration.as_millis();
                while !sudoku.is_solved() && sudoku.is_valid().is_ok() {
                    sudoku.rule_solve(None).unwrap();
                }
                if sudoku.is_solved() {
                    eprintln!("solved !");
                    assert_eq!(Some(difficulty), sudoku.get_difficulty());
                } else {
                    eprintln!("unsolved...");
                    assert!(false);
                }
            }

            let average_duration = total_duration as f64 / iterations as f64;
            println!(
                "Average time for difficulty {}: {:.2} ms",
                difficulty, average_duration
            );
            eprintln!(
                "Average time for difficulty {}: {:.2} ms",
                difficulty, average_duration
            );
        }
        assert!(true);
    }
}
