mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("test.txt");
    println!("{}", sudoku);
    sudoku.display_possibilities();
    if !sudoku.is_valid() {
        println!("Sudoku isn't valid !");
        return;
    }
    let difficulty = sudoku.rule_solve();
    println!("{}", sudoku);
    println!("difficulty : {}", difficulty);
    if sudoku.is_valid() {
        println!("Sudoku is valid");
    } else {
        println!("AAAAAAAAAAAAAAA \nSudoku isn't valid ! \nThere must be an error in a rule ! \nAAAAAAAAAAAAAAA");
    }
}
