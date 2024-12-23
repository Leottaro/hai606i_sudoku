#[cfg(test)]
mod tests {
    use crate::sudoku::simple_sudoku::Sudoku;

    #[test]
    fn simple_sudoku_parse_file() {
        let mut sudoku = Sudoku::parse_file("sudoku-3-moyen-1.txt");
        println!("{}", sudoku);
        let difficulty = sudoku.rule_solve();
        if sudoku.is_valid() {
            println!("{}", sudoku);
            println!("passed, difficulty = {}", difficulty);
            assert!(true);
        } else {
            println!("{}", sudoku);
            println!("failed, difficulty > {}", difficulty);
            assert!(false);
        }
    }
}
