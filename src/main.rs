mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("test.txt").unwrap();
    println!("{}", sudoku);
    sudoku.display_possibilities();
    if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
        println!("Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule", x1, y1, x2, y2);
        return;
    }
    let difficulty = sudoku.rule_solve();
    println!("{}", sudoku);
    println!("difficulty : {}", difficulty);
    if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
        println!("Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule", x1, y1, x2, y2);
    } else {
        println!("Sudoku is valid");
    }
}
