mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("sudoku-2-testr3.txt");
    sudoku.display_possibilities();
    println!("{}", sudoku);
    let difficulty = sudoku.rule_solve();
    println!("{}", sudoku);
    println!("difficulty : {}", difficulty);
    if sudoku.is_valid() {
        println!("Sudoku is valid");
    } else {
        println!("AAAAAAAAAAAAAAA \nSudoku isn't valid ! \nAAAAAAAAAAAAAAA");
    }
}
