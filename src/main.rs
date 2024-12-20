mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("sudoku-4.txt");
    println!("{}", sudoku);
    sudoku.solve(0, 0);
    println!("{}", sudoku);
    if sudoku.is_valid() {
        println!("Sudoku is valid");
    } else {
        println!("AAAAAAAAAAAAAAA \nSudoku isn't valid ! \nAAAAAAAAAAAAAAA");
    }
}
