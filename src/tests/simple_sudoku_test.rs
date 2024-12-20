#[cfg(test)]
mod tests {
    use super::super::super::sudoku::simple_sudoku::Sudoku;

    #[test]
    fn simple_sudoku_parse_file() {
        let mut sudoku = Sudoku::parse_file("sudoku-3-facile-1.txt");
        println!("{}", sudoku);
        sudoku.solve(0, 0);
        println!("{}", sudoku);
        if sudoku.is_valid() {
            println!("Sudoku is valid");
        } else {
            println!("AAAAAAAAAAAAAAA \nSudoku isn't valid ! \nAAAAAAAAAAAAAAA");
        }
    }
}
