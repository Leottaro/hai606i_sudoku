#[cfg(test)]
mod tests {
    use std::fs::ReadDir;

    use crate::simple_sudoku::Sudoku;

    #[test]
    fn test_parse_file() {
        let parsed_sudoku = {
            let temp = Sudoku::parse_file("sudoku-3-difficile-1.txt");
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
            expected_sudoku.fix_value(0, 0, 4);
            expected_sudoku.fix_value(6, 0, 8);
            expected_sudoku.fix_value(8, 0, 5);
            expected_sudoku.fix_value(1, 1, 3);
            expected_sudoku.fix_value(3, 2, 7);
            expected_sudoku.fix_value(1, 3, 2);
            expected_sudoku.fix_value(7, 3, 6);
            expected_sudoku.fix_value(4, 4, 8);
            expected_sudoku.fix_value(6, 4, 4);
            expected_sudoku.fix_value(4, 5, 1);
            expected_sudoku.fix_value(3, 6, 6);
            expected_sudoku.fix_value(5, 6, 3);
            expected_sudoku.fix_value(7, 6, 7);
            expected_sudoku.fix_value(0, 7, 5);
            expected_sudoku.fix_value(3, 7, 2);
            expected_sudoku.fix_value(0, 8, 1);
            expected_sudoku.fix_value(2, 8, 4);
            expected_sudoku.fix_value(0, 0, 4);
            println!("expected_sudoku: \n{}", expected_sudoku);

            assert_eq!(parsed_sudoku, expected_sudoku);
        }
    }

    #[test]
    fn rule_solving() {
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
        for file_name in file_names {
            println!("file_name: {}", file_name);
            if !file_name.starts_with("sudoku-3") || !file_name.ends_with(".txt") {
                continue;
            }
            println!("\n\n\n\n\n");
            let sudoku_result = Sudoku::parse_file(&file_name);
            if let Err(error) = sudoku_result {
                println!("Error while parsing file {}: \n{}", file_name, error);
                continue;
            }
            let mut sudoku = sudoku_result.unwrap();
            println!("{}: \n{}", file_name, sudoku,);

            let mut rule_solve: Result<usize, ((usize, usize), (usize, usize))> =
                sudoku.rule_solve(None);
            while rule_solve != Ok(0) && !sudoku.is_solved() {
                rule_solve = sudoku.rule_solve(None);
            }
            println!("{}", sudoku);

            match rule_solve {
                Ok(difficulty) => println!("Sudoku is valid, difficulty : {}", difficulty),
                Err(((x1, y1), (x2, y2))) => {
                    println!(
						"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule",
						x1, y1, x2, y2
					);
                    assert!(false);
                }
            }
        }
        assert!(true);
    }
}
