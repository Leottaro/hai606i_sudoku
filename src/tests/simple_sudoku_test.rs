#[cfg(test)]
mod tests {
    use std::fs::ReadDir;

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
        // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        // #[cfg(debug_assertions)]
        let files: ReadDir = std::fs::read_dir("res/sudoku_samples").unwrap();
        let file_names: Vec<String> = files
            .map(|file| {
                file.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap()
            })
            .collect();
        let mut sudoku_solved: Vec<(String, usize)> = Vec::new();
        let mut sudoku_unsolved: Vec<String> = Vec::new();
        for file_name in file_names.iter() {
            if !file_name.starts_with("sudoku-3") || !file_name.ends_with(".txt") {
                continue;
            }
            println!("\n\n\nfile {}:", file_name);
            let sudoku_result = Sudoku::parse_file(&file_name);
            if let Err(error) = sudoku_result {
                println!("Error while parsing file {}: \n{}", file_name, error);
                continue;
            }
            let mut sudoku = sudoku_result.unwrap();

            println!("{}", sudoku);
            let mut difficulty: usize = 0;
            loop {
                match sudoku.rule_solve(None) {
                    Ok(0) => {
                        if sudoku.is_solved() {
                            println!("sudoku solved!");
                            sudoku_solved.push((file_name.clone(), difficulty));
                        } else {
                            println!("can't do anything more...");
                            sudoku_unsolved.push(file_name.clone());
                        }
                        break;
                    }
                    Ok(diff) => difficulty = usize::max(difficulty, diff),
                    Err(((x1, y1), (x2, y2))) => {
                        println!("Error: ({}, {}) and ({}, {})", x1, y1, x2, y2);
                        assert!(false);
                        break;
                    }
                }
            }
            println!("{}", sudoku);
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
        assert!(true);
    }
}
