#[cfg(test)]
mod tests {
    use crate::sudoku::simple_sudoku::Sudoku;

    #[test]
    fn test_parse_file() {
        let parsed_sudoku = {
            let temp = Sudoku::parse_file("sudoku-rule-2.txt");
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
            expected_sudoku.fix_value(2, 3, 8);
            expected_sudoku.fix_value(4, 6, 8);
            expected_sudoku.fix_value(1, 7, 6);
            expected_sudoku.fix_value(0, 8, 9);
            expected_sudoku.fix_value(1, 8, 1);
            println!("expected_sudoku: \n{}", expected_sudoku);

            assert_eq!(parsed_sudoku, expected_sudoku);
        }
    }

    #[test]
    fn rule_solving() {
        for rule in 1..=15 {
            let file_name = format!("sudoku-rule-{}.txt", rule);
            println!("\n\n\n\n\nRule: {}", rule);
            let sudoku_result = Sudoku::parse_file(&file_name);
            if let Err(error) = sudoku_result {
                println!(
                    "Error while parsing file sudoku-rule-{}.txt: \n{}",
                    rule, error
                );
                continue;
            }
            let mut sudoku = sudoku_result.unwrap();
            println!("{}: \n{}", file_name, sudoku);
            sudoku.display_possibilities();

            let rule_solve = sudoku.rule_solve(false);
            println!("{}", sudoku);
            sudoku.display_possibilities();

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
