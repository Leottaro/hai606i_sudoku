use super::Sudoku;
#[cfg(debug_assertions)]
use log::debug;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
};

#[allow(dead_code)] // no warning due to unused functions
impl Sudoku {
    // GETTERS / SETTERS
    pub fn get_n(&self) -> usize {
        self.n
    }
    pub fn get_n2(&self) -> usize {
        self.n2
    }
    pub fn get_board(&self) -> Vec<Vec<usize>> {
        self.board.clone()
    }
    pub fn get_possibility_board(&self) -> Vec<Vec<HashSet<usize>>> {
        self.possibility_board.clone()
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for group in Sudoku::get_cell_groups(self.n, x, y) {
            for (x, y) in group {
                self.possibility_board[y][x].remove(&value);
            }
        }
    }

    // GLOBAL FUNCTIONS

    pub fn get_lines(n: usize) -> Vec<Vec<(usize, usize)>> {
        let mut lines = Vec::new();
        for y in 0..n * n {
            let mut line = Vec::new();
            for x in 0..n * n {
                line.push((x, y));
            }
            lines.push(line);
        }
        lines
    }

    pub fn get_cols(n: usize) -> Vec<Vec<(usize, usize)>> {
        let mut lines = Vec::new();
        for x in 0..n * n {
            let mut line = Vec::new();
            for y in 0..n * n {
                line.push((x, y));
            }
            lines.push(line);
        }
        lines
    }

    pub fn get_squares(n: usize) -> Vec<Vec<(usize, usize)>> {
        let mut squares = Vec::new();
        for y0 in (0..n * n).step_by(n) {
            for x0 in (0..n * n).step_by(n) {
                let mut square = Vec::new();
                for j in 0..n {
                    for i in 0..n {
                        square.push((x0 + i, y0 + j));
                    }
                }
                squares.push(square);
            }
        }
        squares
    }

    pub fn get_groups(n: usize) -> Vec<Vec<(usize, usize)>> {
        let mut groups = Vec::new();
        groups.extend(Sudoku::get_lines(n));
        groups.extend(Sudoku::get_cols(n));
        groups.extend(Sudoku::get_squares(n));
        groups
    }

    pub fn get_cell_line(n: usize, y: usize) -> Vec<(usize, usize)> {
        let mut line = Vec::new();
        for x in 0..n * n {
            line.push((x, y));
        }
        line
    }

    pub fn get_cell_col(n: usize, x: usize) -> Vec<(usize, usize)> {
        let mut line = Vec::new();
        for y in 0..n * n {
            line.push((x, y));
        }
        line
    }

    pub fn get_cell_square(n: usize, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut square = Vec::new();
        let x0 = x - x % n;
        let y0 = y - y % n;
        for i in 0..n {
            for j in 0..n {
                square.push((x0 + i, y0 + j));
            }
        }
        square
    }

    pub fn get_cell_groups(n: usize, x: usize, y: usize) -> Vec<Vec<(usize, usize)>> {
        vec![
            Sudoku::get_cell_line(n, y),
            Sudoku::get_cell_col(n, x),
            Sudoku::get_cell_square(n, x, y),
        ]
    }

    // CREATION

    pub fn new(n: usize) -> Self {
        Self {
            n: n,
            n2: n * n,
            board: vec![vec![0; n * n]; n * n],
            possibility_board: vec![vec![(1..=n * n).collect(); n * n]; n * n],
        }
    }

    pub fn parse_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = {
            let mut path_builder = current_dir().unwrap();
            path_builder.push("res/sudoku_samples/");
            path_builder.push(file_name);
            path_builder.into_os_string().into_string().unwrap()
        };
        let file = std::fs::read_to_string(file_path)?;
        let mut lines = file.lines();
        let n: usize = lines.next().unwrap().parse()?;
        let n2 = n * n;

        let board: Vec<Vec<usize>> = lines
            .take(n2)
            .map(|line| {
                line.split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect()
            })
            .collect();

        let mut possibility_board: Vec<Vec<HashSet<usize>>> =
            vec![vec![(1..=n2).collect(); n2]; n2];

        for y in 0..n2 {
            for x in 0..n2 {
                let value = board[y][x];
                if value == 0 {
                    continue;
                }
                possibility_board[y][x].clear();
                for group in Sudoku::get_cell_groups(n, x, y) {
                    for (x, y) in group {
                        possibility_board[y][x].remove(&value);
                    }
                }
            }
        }

        let sudoku = Self {
            n,
            n2,
            board,
            possibility_board,
        };

        if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
            return Err(format!(
				"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in the file",
				x1, y1, x2, y2
			)
			.into());
        }
        Ok(sudoku)
    }

    // RULE SOLVING
    pub fn rule_solve(&mut self) -> Result<usize, ((usize, usize), (usize, usize))> {
        let rules: Vec<(fn(&mut Sudoku) -> bool, usize)> = vec![
            (Sudoku::naked_singles, 1),
            (Sudoku::hidden_singles, 2),
            (Sudoku::naked_pairs, 3),
            (Sudoku::naked_triples, 4),
            (Sudoku::hidden_pairs, 5),
            (Sudoku::hidden_triples, 6),
            (Sudoku::naked_quads, 7),
            (Sudoku::hidden_quads, 8),
            (Sudoku::pointing_pair, 9),
            (Sudoku::pointing_triple, 10),
            (Sudoku::box_reduction, 11),
            (Sudoku::x_wing, 12),
            (Sudoku::finned_x_wing, 13),
            (Sudoku::sashimi_finned_x_wing, 14),
            (Sudoku::franken_x_wing, 15),
            (Sudoku::skyscraper, 16),
            (Sudoku::y_wing, 17),
            (Sudoku::w_wing, 18),
            (Sudoku::swordfish, 19),
            (Sudoku::finned_swordfish, 20),
            (Sudoku::sashimi_finned_swordfish, 21),
            (Sudoku::xyz_wing, 22),
            (Sudoku::bi_value_universal_grave, 23),
            (Sudoku::xy_chain, 24),
            (Sudoku::jellyfish, 25),
            (Sudoku::finned_jellyfish, 26),
            (Sudoku::sashimi_finned_jellyfish, 27),
            (Sudoku::wxyz_wing, 28),
            (Sudoku::subset_exclusion, 29),
            (Sudoku::empty_rectangle, 30),
            (Sudoku::almost_locked_set_forcing_chain, 31),
            (Sudoku::death_blossom, 32),
            (Sudoku::pattern_overlay, 33),
            (Sudoku::bowmans_bingo, 34),
        ];

        let mut difficulty: usize = 0;
        // try the rules and set the difficulty in consequence
        for &(rule, diff) in rules.iter() {
            // if the rule can't be applied, then pass to the next one
            if !rule(self) {
                continue;
            }
            #[cfg(debug_assertions)]
            {
                debug!("règle {} appliquée", diff);
                debug!("Sudoku actuel:\n{}", self);
            } // self.display_possibilities();

            difficulty = max(difficulty, diff);
            let is_valid = self.is_valid();
            if is_valid.is_err() {
                return Err(is_valid.unwrap_err());
            }
            break;
        }

        Ok(difficulty)
    }

    pub fn solve_once(&mut self) -> Result<usize, ((usize, usize), (usize, usize))> {
        let rules: Vec<(fn(&mut Sudoku) -> bool, usize)> = vec![
            (Sudoku::naked_singles, 1),
            (Sudoku::hidden_singles, 2),
            (Sudoku::naked_pairs, 3),
            (Sudoku::naked_triples, 4),
            (Sudoku::hidden_pairs, 5),
            (Sudoku::hidden_triples, 6),
            (Sudoku::naked_quads, 7),
            (Sudoku::hidden_quads, 8),
            (Sudoku::pointing_pair, 9),
            (Sudoku::pointing_triple, 10),
            (Sudoku::box_reduction, 11),
            (Sudoku::x_wing, 12),
            (Sudoku::finned_x_wing, 13),
            (Sudoku::sashimi_finned_x_wing, 14),
            (Sudoku::franken_x_wing, 15),
            (Sudoku::skyscraper, 16),
            (Sudoku::y_wing, 17),
            (Sudoku::w_wing, 18),
            (Sudoku::swordfish, 19),
            (Sudoku::finned_swordfish, 20),
            (Sudoku::sashimi_finned_swordfish, 21),
            (Sudoku::xyz_wing, 22),
            (Sudoku::bi_value_universal_grave, 23),
            (Sudoku::xy_chain, 24),
            (Sudoku::jellyfish, 25),
            (Sudoku::finned_jellyfish, 26),
            (Sudoku::sashimi_finned_jellyfish, 27),
            (Sudoku::wxyz_wing, 28),
            (Sudoku::subset_exclusion, 29),
            (Sudoku::empty_rectangle, 30),
            (Sudoku::almost_locked_set_forcing_chain, 31),
            (Sudoku::death_blossom, 32),
            (Sudoku::pattern_overlay, 33),
            (Sudoku::bowmans_bingo, 34),
        ];

        let mut difficulty: usize = 0;
        // try the rules and set the difficulty in consequence
        for &(rule, diff) in rules.iter() {
            // if the rule can't be applied, then pass to the next one
            if !rule(self) {
                continue;
            }
            #[cfg(debug_assertions)]
            {
                debug!("règle {} appliquée", diff);
                debug!("Sudoku actuel:\n{}", self);
            } // self.display_possibilities();

            difficulty = max(difficulty, diff);
            let is_valid = self.is_valid();
            if is_valid.is_err() {
                return Err(is_valid.unwrap_err());
            }
            break;
        }

        Ok(difficulty)
    }

    // BACKTRACK SOLVING

    pub fn backtrack_solve(&mut self, mut x: usize, mut y: usize) -> bool {
        if y == self.n2 - 1 && x == self.n2 {
            return true;
        }

        if x == self.n2 {
            y += 1;
            x = 0;
        }

        if self.board[y][x] != 0 {
            return self.backtrack_solve(x + 1, y);
        }

        let possible_values = self.possibility_board[y][x].clone();
        let cell_group: Vec<(usize, usize)> = Sudoku::get_cell_groups(self.n, x, y)
            .into_iter()
            .flatten()
            .collect();
        self.possibility_board[y][x].clear();
        for value in possible_values.clone().into_iter() {
            self.board[y][x] = value;
            let changing_cells: Vec<&(usize, usize)> = cell_group
                .iter()
                .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                .collect();
            for &&(x, y) in changing_cells.iter() {
                self.possibility_board[y][x].remove(&value);
            }
            if self.backtrack_solve(x + 1, y) {
                return true;
            }
            self.board[y][x] = 0;
            for &(x, y) in changing_cells {
                self.possibility_board[y][x].insert(value);
            }
        }
        self.possibility_board[y][x] = possible_values;

        return false;
    }

    // DISPLAY

    const BASE_64: [char; 64] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
        'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a',
        'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
        't', 'u', 'v', 'w', 'x', 'y', 'z', 'α', 'β', 'δ',
    ];
    pub fn to_string(&self) -> String {
        if self.n > 8 {
            return "Can't display a sudoku with n > 8".to_string();
        }
        let mut lines: Vec<String> = Vec::new();
        let n_dash = "─".repeat(2 * self.n + 1);

        lines.push({
            let mut first_line: Vec<String> = Vec::new();
            for _ in 0..self.n {
                first_line.push(n_dash.clone());
            }
            format!("┌{}┐", first_line.join("┬"))
        });

        for y in 0..self.n2 {
            let mut line: Vec<String> = Vec::new();
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    line.push("│".to_string());
                }
                if self.board[y][x] == 0 {
                    line.push("·".to_string());
                } else {
                    line.push(Sudoku::BASE_64[self.board[y][x] - 1].to_string());
                };
            }
            lines.push(format!("│ {} │", line.join(" ")));

            if y != self.n2 - 1 && (y + 1) % self.n == 0 {
                let mut temp: Vec<String> = Vec::new();
                for _ in 0..self.n {
                    temp.push(n_dash.clone());
                }
                lines.push(format!("├{}┤", temp.join("┼")));
            }
        }

        lines.push({
            let mut last_line: Vec<String> = Vec::new();
            for _ in 0..self.n {
                last_line.push(n_dash.clone());
            }
            format!("└{}┘", last_line.join("┴"))
        });

        lines.join("\n")
    }

    pub fn display_possibilities(&self) {
        println!("");
        for y in 0..self.n2 {
            if y != 0 && y % self.n == 0 {
                println!(
                    "{}┼{}┼{}",
                    "─".repeat(self.n2 * self.n + self.n * 4),
                    "─".repeat(self.n2 * self.n + self.n * 4),
                    "─".repeat(self.n2 * self.n + self.n * 4),
                );
            }
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    print!("│");
                }
                print!(" {{");
                for value in 1..=self.n2 {
                    if self.possibility_board[y][x].contains(&value) {
                        print!("{}", value);
                    } else {
                        print!(" ");
                    }
                }
                print!("}} ");
            }
            println!();
        }
        println!("\n\n");
    }

    // UTILITY

    pub fn is_valid(&self) -> Result<(), ((usize, usize), (usize, usize))> {
        // check si un groupe contient 2 fois la même valeur
        for group in Sudoku::get_groups(self.n) {
            let mut already_seen_values: HashMap<usize, (usize, usize)> = HashMap::new();
            for (x, y) in group.into_iter() {
                let cell_value = self.board[y][x];
                if cell_value == 0 {
                    continue;
                }
                if already_seen_values.contains_key(&cell_value) {
                    return Err((
                        (x, y),
                        already_seen_values.get(&cell_value).unwrap().clone(),
                    ));
                }
                already_seen_values.insert(cell_value, (x, y));
            }
        }

        // check si une cellule pas encore fixée n'a plus de possibilités
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.board[y][x] == 0 && self.possibility_board[y][x].is_empty() {
                    return Err(((x, y), (x, y)));
                }
            }
        }

        Ok(())
    }

    pub fn is_solved(&self) -> bool {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.board[y][x] == 0 {
                    return false;
                }
            }
        }
        true
    }
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Sudoku {
    fn eq(&self, other: &Self) -> bool {
        if self.n != other.n {
            return false;
        }

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.board[y][x] != other.board[y][x] {
                    return false;
                }
            }
        }

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if !self.possibility_board[y][x].eq(&other.possibility_board[y][x]) {
                    return false;
                }
            }
        }
        true
    }
}
