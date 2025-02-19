use super::{
    Sudoku,
    SudokuGroups::{self, *},
};
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

    pub fn get_groups(&self, groups: SudokuGroups) -> Vec<HashSet<(usize, usize)>> {
        self.groups.get(&groups).unwrap().to_owned()
    }

    pub fn get_cell_group(
        &self,
        x: usize,
        y: usize,
        groups: SudokuGroups,
    ) -> HashSet<(usize, usize)> {
        self.cell_groups.get(&(x, y, groups)).unwrap().to_owned()
    }

    pub fn get_cell_groups(
        &self,
        x: usize,
        y: usize,
        groups: Vec<SudokuGroups>,
    ) -> Vec<HashSet<(usize, usize)>> {
        groups
            .iter()
            .map(|&group| self.get_cell_group(x, y, group))
            .collect()
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for &(x, y) in self.cell_groups.get(&(x, y, ALL)).unwrap() {
            self.possibility_board[y][x].remove(&value);
        }
    }

    pub fn is_same_group(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        x1 == x2 || y1 == y2 || (x1 / self.n == x2 / self.n && y1 / self.n == y2 / self.n)
    }

    pub fn get_strong_links(&self, value: usize) -> Vec<((usize, usize), (usize, usize))> {
        let mut strong_links: Vec<((usize, usize), (usize, usize))> = Vec::new();
        for group in self.groups.get(&ALL).unwrap() {
            let value_cells: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                .collect();
            if value_cells.len() == 2 {
                strong_links.push((value_cells[0].clone(), value_cells[1].clone()));
            }
        }
        strong_links
    }

    // CREATION

    pub fn new(n: usize) -> Self {
        let n2 = n * n;
        let board = vec![vec![0; n2]; n2];
        let possibility_board = vec![vec![(1..=n2).collect(); n2]; n2];

        let mut rows = Vec::new();
        let mut cols = Vec::new();
        for i in 0..n2 {
            let mut row = HashSet::new();
            let mut col = HashSet::new();
            for j in 0..n2 {
                row.insert((j, i));
                col.insert((i, j));
            }
            rows.push(row);
            cols.push(col);
        }
        let mut lines = rows.clone();
        lines.extend(cols.clone());

        let mut squares = Vec::new();
        for y0 in 0..n {
            for x0 in 0..n {
                let mut square = HashSet::new();
                for dy in 0..n {
                    for dx in 0..n {
                        square.insert((x0 * n + dx, y0 * n + dy));
                    }
                }
                squares.push(square);
            }
        }
        let mut all = lines.clone();
        all.extend(squares.clone());

        let mut cell_groups = HashMap::new();
        for y in 0..n2 {
            for x in 0..n2 {
                let row = rows[y].clone();
                let col = cols[x].clone();
                let square = squares[(y / n) * n + (x / n)].clone();
                let lines = row.union(&col).cloned().collect::<HashSet<_>>();
                let all = lines.union(&square).cloned().collect::<HashSet<_>>();
                cell_groups.insert((x, y, ROW), row);
                cell_groups.insert((x, y, COLUMN), col);
                cell_groups.insert((x, y, SQUARE), square);
                cell_groups.insert((x, y, LINES), lines);
                cell_groups.insert((x, y, ALL), all);
            }
        }

        let mut groups = HashMap::new();
        groups.insert(ROW, rows);
        groups.insert(COLUMN, cols);
        groups.insert(LINES, lines);
        groups.insert(SQUARE, squares);
        groups.insert(ALL, all);

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
                sudoku.set_value(x, y, value);
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
            (Sudoku::finned_mutant_x_wing, 15),
            (Sudoku::skyscraper, 16),
            (Sudoku::simple_coloring, 17),
            (Sudoku::y_wing, 18),
            (Sudoku::w_wing, 19),
            (Sudoku::swordfish, 20),
            (Sudoku::finned_swordfish, 21),
            (Sudoku::sashimi_finned_swordfish, 22),
            (Sudoku::franken_swordfish, 23),
            (Sudoku::mutant_swordfish, 24),
            (Sudoku::finned_mutant_swordfish, 25),
            (Sudoku::sashimi_finned_mutant_swordfish, 26),
            (Sudoku::sue_de_coq, 27),
            (Sudoku::xyz_wing, 28),
            (Sudoku::x_cycle, 29),
            (Sudoku::bi_value_universal_grave, 30),
            (Sudoku::xy_chain, 31),
            (Sudoku::three_d_medusa, 32),
            (Sudoku::jellyfish, 33),
            (Sudoku::finned_jellyfish, 34),
            (Sudoku::sashimi_finned_jellyfish, 35),
            (Sudoku::avoidable_rectangle, 36),
            (Sudoku::unique_rectangle, 37),
            (Sudoku::hidden_unique_rectangle, 38),
            (Sudoku::wxyz_wing, 39),
            (Sudoku::firework, 40),
            (Sudoku::subset_exclusion, 41),
            (Sudoku::empty_rectangle, 42),
            (Sudoku::sue_de_coq_extended, 43),
            (Sudoku::sk_loop, 44),
            (Sudoku::exocet, 45),
            (Sudoku::almost_locked_sets, 46),
            (Sudoku::alternating_inference_chain, 47),
            (Sudoku::digit_forcing_chains, 48),
            (Sudoku::nishio_forcing_chains, 49),
            (Sudoku::cell_forcing_chains, 50),
            (Sudoku::unit_forcing_chains, 51),
            (Sudoku::almost_locked_set_forcing_chain, 52),
            (Sudoku::death_blossom, 53),
            (Sudoku::pattern_overlay, 54),
            (Sudoku::bowmans_bingo, 55),
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
        let cell_group: HashSet<(usize, usize)> = self.get_cell_group(x, y, ALL).clone();
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
