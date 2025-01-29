mod sudoku;
use sudoku::simple_sudoku::Sudoku;

mod tests;

fn main() {
    let mut sudoku = Sudoku::parse_file("sudoku-rule-9.txt").unwrap();
    println!("{}", sudoku);
    sudoku.display_possibilities();
    if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
        println!("Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule", x1, y1, x2, y2);
        return;
    }

    let rule_solve = sudoku.rule_solve(true);
    println!("{}", sudoku);
    match rule_solve {
		Ok(difficulty) => println!("Sudoku is valid, difficulty : {}", difficulty),
		Err(((x1, y1), (x2, y2))) => println!("Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule", x1, y1, x2, y2),
	}
}
