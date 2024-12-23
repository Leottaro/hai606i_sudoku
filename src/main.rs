mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("sudoku-3-facile-2.txt");
    println!("{}", sudoku);
    sudoku.rule_solve();
    println!("{}", sudoku);
    if sudoku.is_valid() {
        println!("Sudoku is valid");
    } else {
        println!("AAAAAAAAAAAAAAA \nSudoku isn't valid ! \nAAAAAAAAAAAAAAA");
    }
}
