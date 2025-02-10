use super::{Sudoku, SudokuGroups::*};
#[cfg(debug_assertions)]
use log::debug;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
    ops::Range,
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

    pub fn fix_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for &(x, y) in self.cell_groups.get(&(x, y, ALL)).unwrap() {
            self.possibility_board[y][x].remove(&value);
        }
    }

    // GLOBAL FUNCTIONS

    pub fn get_rows(n: usize) -> Vec<HashSet<(usize, usize)>> {
        let mut lines = Vec::new();
        for y in 0..n * n {
            let mut line = HashSet::new();
            for x in 0..n * n {
                line.insert((x, y));
            }
            lines.push(line);
        }
        lines
    }

    pub fn get_cols(n: usize) -> Vec<HashSet<(usize, usize)>> {
        let mut lines = Vec::new();
        for x in 0..n * n {
            let mut line = HashSet::new();
            for y in 0..n * n {
                line.insert((x, y));
            }
            lines.push(line);
        }
        lines
    }

    pub fn get_squares(n: usize) -> Vec<HashSet<(usize, usize)>> {
        let mut squares = Vec::new();
        for y0 in (0..n * n).step_by(n) {
            for x0 in (0..n * n).step_by(n) {
                let mut square = HashSet::new();
                for j in 0..n {
                    for i in 0..n {
                        square.insert((x0 + i, y0 + j));
                    }
                }
                squares.push(square);
            }
        }
        squares
    }

    pub fn get_groups(n: usize) -> Vec<HashSet<(usize, usize)>> {
        let mut groups = Vec::new();
        groups.extend(Sudoku::get_rows(n));
        groups.extend(Sudoku::get_cols(n));
        groups.extend(Sudoku::get_squares(n));
        groups
    }

    pub fn get_cell_row(n: usize, y: usize) -> HashSet<(usize, usize)> {
        let mut line = HashSet::new();
        for x in 0..n * n {
            line.insert((x, y));
        }
        line
    }

    pub fn get_cell_col(n: usize, x: usize) -> HashSet<(usize, usize)> {
        let mut line = HashSet::new();
        for y in 0..n * n {
            line.insert((x, y));
        }
        line
    }

    pub fn get_cell_square(n: usize, x: usize, y: usize) -> HashSet<(usize, usize)> {
        let mut square = HashSet::new();
        let x0 = x - x % n;
        let y0 = y - y % n;
        for i in 0..n {
            for j in 0..n {
                square.insert((x0 + i, y0 + j));
            }
        }
        square
    }

    pub fn get_cell_groups(n: usize, x: usize, y: usize) -> Vec<HashSet<(usize, usize)>> {
        vec![
            Sudoku::get_cell_row(n, y),
            Sudoku::get_cell_col(n, x),
            Sudoku::get_cell_square(n, x, y),
        ]
    }

    // CREATION

    pub fn new(n: usize) -> Self {
        let n2 = n * n;
        let board = vec![vec![0; n2]; n2];
        let possibility_board = vec![vec![(1..=n2).collect(); n2]; n2];

        let mut groups = HashMap::new();
        let rows = Sudoku::get_rows(n);
        let columns = Sudoku::get_cols(n);
        let squares = Sudoku::get_squares(n);
        let mut lines = rows.clone();
        lines.extend(columns.clone());
        let mut all = lines.clone();
        all.extend(squares.clone());
        groups.insert(ROW, rows);
        groups.insert(COLUMN, columns);
        groups.insert(LINES, lines);
        groups.insert(SQUARE, squares);
        groups.insert(ALL, all);

        let mut cell_groups = HashMap::new();
        for y in 0..n2 {
            for x in 0..n2 {
                let rows = Sudoku::get_cell_row(n, y);
                let columns = Sudoku::get_cell_col(n, x);
                let squares = Sudoku::get_cell_square(n, x, y);
                let mut lines = rows.clone();
                lines.extend(columns.clone());
                let mut all = lines.clone();
                all.extend(squares.clone());
                cell_groups.insert((x, y, ROW), rows);
                cell_groups.insert((x, y, COLUMN), columns);
                cell_groups.insert((x, y, LINES), lines);
                cell_groups.insert((x, y, SQUARE), squares);
                cell_groups.insert((x, y, ALL), all);
            }
        }

        Self {
            n,
            n2,
            board,
            possibility_board,
            groups,
            cell_groups,
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

        let mut sudoku = Self::new(n);
        for (y, line) in lines.take(sudoku.n2).enumerate() {
            for (x, cell) in line.split_whitespace().enumerate() {
                let value: usize = cell.parse().unwrap();
                if value == 0 {
                    continue;
                }
                sudoku.fix_value(x, y, value);
            }
        }

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
    pub fn rule_solve(
        &mut self,
        specific_rules: Option<Range<usize>>,
    ) -> Result<usize, ((usize, usize), (usize, usize))> {
        let mut rules: Vec<(fn(&mut Sudoku) -> bool, usize)> = vec![
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
            (Sudoku::franken_x_wing, 14),
            (Sudoku::finned_mutant_x_xing, 15),
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
        if specific_rules.is_some() {
            rules = rules
                .into_iter()
                .filter(|(_rule, id)| specific_rules.as_ref().unwrap().contains(id))
                .collect()
        }

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
            }

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
        let cell_group: HashSet<(usize, usize)> =
            self.cell_groups.get(&(x, y, ALL)).unwrap().clone();
        self.possibility_board[y][x].clear();
        for value in possible_values.clone().into_iter() {
            self.board[y][x] = value;
            let changing_cells: HashSet<&(usize, usize)> = cell_group
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

    const BASE_64: [char; 65] = [
        '·', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'α', 'β', 'δ',
    ];
    pub fn to_string(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for y in 0..self.n2 {
            if y != 0 && y % self.n == 0 {
                let temp = "━".repeat(2 * self.n2 + 4 * self.n + 1);
                lines.push(format!("━{}", vec![temp; self.n].join("╋")));
            }
            let mut this_row_lines: Vec<String> = vec![" ".to_string(); self.n];
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    for line in this_row_lines.iter_mut() {
                        line.push_str(" ┃");
                    }
                }
                if self.board[y][x] != 0 {
                    for (i, line) in this_row_lines.iter_mut().enumerate() {
                        if i == self.n / 2 {
                            line.push_str(&format!(
                                " {}{}{}",
                                " ".repeat(self.n + 1),
                                Sudoku::BASE_64[self.board[y][x]],
                                " ".repeat(self.n + 1)
                            ));
                        } else {
                            line.push_str(&" ".repeat(2 * (self.n + 2)));
                        }
                    }
                    continue;
                }

                this_row_lines.get_mut(0).unwrap().push_str(" ⎧");
                for line in this_row_lines.iter_mut().skip(1).take(self.n - 2) {
                    line.push_str(" ⎪");
                }
                this_row_lines.get_mut(self.n - 1).unwrap().push_str(" ⎩");

                for i in 0..self.n {
                    for j in 0..self.n {
                        let value = i * self.n + j + 1;
                        let displayed_char = if self.possibility_board[y][x].contains(&value) {
                            Sudoku::BASE_64[value]
                        } else {
                            '·'
                        };
                        this_row_lines
                            .get_mut(i)
                            .unwrap()
                            .push_str(&format!(" {displayed_char}"));
                    }
                }

                this_row_lines.get_mut(0).unwrap().push_str(" ⎫");
                for line in this_row_lines.iter_mut().skip(1).take(self.n - 2) {
                    line.push_str(" ⎪");
                }
                this_row_lines.get_mut(self.n - 1).unwrap().push_str(" ⎭");
            }

            for line in this_row_lines.into_iter() {
                lines.push(line);
            }
        }
        lines.join("\n")
    }

    // UTILITY

    pub fn is_valid(&self) -> Result<(), ((usize, usize), (usize, usize))> {
        // check si un groupe contient 2 fois la même valeur
        for group in self.groups.get(&ALL).unwrap() {
            for (i, &(x1, y1)) in group.iter().enumerate() {
                if self.board[y1][x1] == 0 {
                    continue;
                }
                for (j, &(x2, y2)) in group.iter().enumerate() {
                    if i != j && self.board[y1][x1] == self.board[y2][x2] {
                        return Err(((x1, y1), (x2, y2)));
                    }
                }
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
                if self.board[y][x] == 0 || !self.possibility_board[y][x].is_empty() {
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
