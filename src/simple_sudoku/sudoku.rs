use super::{Sudoku, SudokuGroups::*};
#[cfg(debug_assertions)]
use log::debug;
use macroquad::rand::ChooseRandom;
use rand::Rng;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
    ops::Range,
    process::exit,
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

    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }

    pub fn fix_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for &(x, y) in self.cell_groups.get(&(x, y, ALL)).unwrap() {
            self.possibility_board[y][x].remove(&value);
        }
    }

    pub fn remove_value(&mut self, x: usize, y: usize) -> usize {
        let removed_value = self.board[y][x];

        self.board[y][x] = 0;
        self.possibility_board[y][x] = (1..=self.n2).into_iter().collect();

        for &(x1, y1) in self.cell_groups.get(&(x, y, ALL)).unwrap() {
            if self.board[y1][x1] != 0 {
                self.possibility_board[y][x].remove(&self.board[y1][x1]);
                continue;
            }

            if self
                .cell_groups
                .get(&(x1, y1, ALL))
                .unwrap()
                .iter()
                .all(|&(x2, y2)| self.board[y2][x2] != removed_value)
            {
                self.possibility_board[y1][x1].insert(removed_value);
            }
        }

        removed_value
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
        let difficulty = 0;

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
            difficulty,
            groups,
            cell_groups,
        }
    }

    pub fn generate_full(n: usize) -> Self {
        let mut sudoku = Self::new(n);
        let values: Vec<usize> = (1..=sudoku.n2).into_iter().collect();

        // OPTION 1
        for i in 0..sudoku.n {
            let mut square_values = {
                let mut temp = values.clone();
                temp.shuffle();
                temp.into_iter()
            };
            for dy in 0..sudoku.n {
                for dx in 0..sudoku.n {
                    sudoku.fix_value(
                        i * sudoku.n + dx,
                        i * sudoku.n + dy,
                        square_values.next().unwrap(),
                    );
                }
            }
        }

        // OPTION 2
        // let (cell0, row0_square0, row0_rest) = {
        //     let mut row0_rest = values.clone();
        //     row0_rest.shuffle();
        //     let cell0 = row0_rest.pop().unwrap();
        //     let row0_square0: Vec<usize> = row0_rest.drain(0..sudoku.n - 1).collect();
        //     (cell0, row0_square0, row0_rest)
        // };

        // let (col0_square0, col0_rest) = {
        //     let mut col0_rest = row0_rest.clone();
        //     col0_rest.shuffle();
        //     let col0_square0: Vec<usize> = col0_rest.drain(0..sudoku.n - 1).collect();
        //     col0_rest.extend(row0_square0.iter());
        //     col0_rest.shuffle();
        //     (col0_square0, col0_rest)
        // };

        // let mut row_values = row0_square0.iter().chain(row0_rest.iter());
        // let mut col_values = col0_square0.iter().chain(col0_rest.iter());

        // sudoku.fix_value(0, 0, cell0);
        // for i in 1..sudoku.n2 {
        //     sudoku.fix_value(i, 0, *row_values.next().unwrap());
        //     sudoku.fix_value(0, i, *col_values.next().unwrap());
        // }

        // for i in 1..sudoku.n2 {
        //     let diag_possibilities: Vec<&usize> = sudoku.possibility_board[i][i].iter().collect();
        //     let random_value = diag_possibilities[rnd.gen_range(0..diag_possibilities.len())];
        //     sudoku.fix_value(i, i, *random_value);
        // }

        // for y in 1..sudoku.n {
        //     for x in 1..sudoku.n {
        //         if x == y {
        //             continue;
        //         }
        //         let diag_possibilities: Vec<&usize> =
        //             sudoku.possibility_board[y][x].iter().collect();
        //         let random_value = diag_possibilities[rnd.gen_range(0..diag_possibilities.len())];
        //         sudoku.fix_value(x, y, *random_value);
        //     }
        // }

        sudoku.backtrack_solve(0, 0);

        sudoku
    }

    pub fn generate(n: usize, aimed_difficulty: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut checkpoint_sudoku = Self::generate_full(n);

        let mut last_removed_x: usize = 0;
        let mut last_removed_y: usize = 0;
        let mut last_removed_value: usize = 0;

        while checkpoint_sudoku.difficulty < aimed_difficulty {
            last_removed_x = rng.gen_range(0..checkpoint_sudoku.n2);
            last_removed_y = rng.gen_range(0..checkpoint_sudoku.n2);
            while checkpoint_sudoku.board[last_removed_y][last_removed_x] == 0 {
                last_removed_x = rng.gen_range(0..checkpoint_sudoku.n2);
                last_removed_y = rng.gen_range(0..checkpoint_sudoku.n2);
            }
            last_removed_value = checkpoint_sudoku.remove_value(last_removed_x, last_removed_y);

            let mut sudoku = checkpoint_sudoku.clone();
            let mut did_one_solve = false;
            loop {
                match sudoku.rule_solve(Some(0..aimed_difficulty + 2)) {
                    Ok(false) => {
                        if !did_one_solve {
                            checkpoint_sudoku = Sudoku::generate_full(n);
							println!("ça recommence !");
                        } else {
                            checkpoint_sudoku.fix_value(
                                last_removed_x,
                                last_removed_y,
                                last_removed_value,
                            );
                        }
                        break;
                    }
                    Ok(true) => {
                        did_one_solve = true;
                        if sudoku.board[last_removed_y][last_removed_x] == last_removed_value {
                            checkpoint_sudoku.difficulty = sudoku.difficulty;
                            break;
                        }
                    }
                    Err(((x1, y1), (x2, y2))) => {
                        eprintln!("OULALALALA GROS PROBLÈME MONSIEUR SUR LA GÉNÉRATION LÀÀÀÀ: \n{x1},{y1} et {x2},{y2}: \n{sudoku}");
                        checkpoint_sudoku = Sudoku::generate_full(n);
							println!("ça recommence !");
                        break;
                    }
                }
            }
        }

        // checkpoint_sudoku.fix_value(last_removed_x, last_removed_y, last_removed_value);
        checkpoint_sudoku.difficulty = 0;

        checkpoint_sudoku
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
    ) -> Result<bool, ((usize, usize), (usize, usize))> {
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
            (Sudoku::simple_coloring, 17),
            (Sudoku::y_wing, 18),
            (Sudoku::w_wing, 19),
            (Sudoku::swordfish, 20),
            (Sudoku::finned_swordfish, 21),
            (Sudoku::sashimi_finned_swordfish, 22),
            (Sudoku::xyz_wing, 23),
            (Sudoku::bi_value_universal_grave, 24),
            (Sudoku::xy_chain, 25),
            (Sudoku::jellyfish, 26),
            (Sudoku::finned_jellyfish, 27),
            (Sudoku::sashimi_finned_jellyfish, 28),
            (Sudoku::wxyz_wing, 29),
            (Sudoku::subset_exclusion, 30),
            (Sudoku::empty_rectangle, 31),
            (Sudoku::almost_locked_set_forcing_chain, 32),
            (Sudoku::death_blossom, 33),
            (Sudoku::pattern_overlay, 34),
            (Sudoku::bowmans_bingo, 35),
        ];
        if specific_rules.is_some() {
            rules = rules
                .into_iter()
                .filter(|(_rule, id)| specific_rules.as_ref().unwrap().contains(id))
                .collect()
        }

        let mut did_something = false;
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

            did_something = true;
            self.difficulty = max(self.difficulty, diff);
            let is_valid = self.is_valid();
            if is_valid.is_err() {
                return Err(is_valid.unwrap_err());
            }
            break;
        }

        Ok(did_something)
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
        let mut randomized_possible_values: Vec<usize> = possible_values.iter().cloned().collect();
        randomized_possible_values.shuffle();
        let cell_group: HashSet<(usize, usize)> =
            self.cell_groups.get(&(x, y, ALL)).unwrap().clone();

        self.possibility_board[y][x].clear();
        for value in randomized_possible_values {
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

impl Clone for Sudoku {
    fn clone(&self) -> Self {
        let mut sudoku = Sudoku::new(self.n);
        sudoku.board = self.board.clone();
        sudoku.possibility_board = self.possibility_board.clone();
        sudoku.difficulty = self.difficulty;
        sudoku
    }
}
