use super::{
    CellGroupMap, Coords, GroupMap, Sudoku,
    SudokuDifficulty::{self, *},
    SudokuGroups::{self, *},
};
use crate::debug_only;
use log::{info, warn};
use rand::Rng;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
    ops::{AddAssign, Range},
    sync::{mpsc, Arc, LazyLock, Mutex, RwLock},
    thread::{available_parallelism, JoinHandle},
};

static GROUPS: LazyLock<RwLock<HashMap<usize, GroupMap>>> = LazyLock::new(Default::default);
static CELL_GROUPS: LazyLock<RwLock<HashMap<usize, CellGroupMap>>> =
    LazyLock::new(Default::default);

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

    pub fn get_difficulty(&self) -> SudokuDifficulty {
        self.difficulty
    }

    pub fn get_cell_value(&self, x: usize, y: usize) -> usize {
        self.board[y][x]
    }

    pub fn get_cell_possibilities(&self, x: usize, y: usize) -> &HashSet<usize> {
        &self.possibility_board[y][x]
    }

    pub fn get_group(&self, groups: SudokuGroups) -> Vec<HashSet<Coords>> {
        GROUPS
            .read()
            .unwrap()
            .get(&self.n)
            .unwrap()
            .get(&groups)
            .unwrap()
            .clone()
    }

    pub fn get_cell_group(&self, x: usize, y: usize, groups: SudokuGroups) -> HashSet<Coords> {
        CELL_GROUPS
            .read()
            .unwrap()
            .get(&self.n)
            .unwrap()
            .get(&((x, y), groups))
            .unwrap()
            .clone()
    }

    pub fn get_cell_groups(
        &self,
        x: usize,
        y: usize,
        groups: Vec<SudokuGroups>,
    ) -> Vec<HashSet<Coords>> {
        groups
            .iter()
            .map(|&group| self.get_cell_group(x, y, group))
            .collect()
    }

    pub fn get_error(&self) -> Option<(Coords, Coords)> {
        self.error
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for (x1, y1) in self.get_cell_group(x, y, All) {
            self.possibility_board[y1][x1].remove(&value);
            if self.board[y1][x1] == value && (x, y) != (x1, y1) {
                self.error = Some(((x, y), (x1, y1)));
            } else if self.board[y1][x1] == 0 && self.possibility_board[y1][x1].is_empty() {
                self.error = Some(((x1, y1), (x1, y1)));
            }
        }
    }

    pub fn remove_value(&mut self, x: usize, y: usize) -> usize {
        let removed_value = self.board[y][x];

        self.board[y][x] = 0;
        self.possibility_board[y][x] = (1..=self.n2).collect();

        for (x1, y1) in self.get_cell_group(x, y, All) {
            if let Some((err1, err2)) = self.error {
                if (err1 == (x, y) && err2 == (x1, y1)) || (err1 == (x1, y1) && err2 == (x, y)) {
                    self.error = None;
                }
            }

            if self.board[y1][x1] != 0 {
                self.possibility_board[y][x].remove(&self.board[y1][x1]);
                continue;
            }

            if self
                .get_cell_group(x1, y1, All)
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

    pub fn get_strong_links(&self, value: usize) -> Vec<(Coords, Coords)> {
        let mut strong_links: Vec<(Coords, Coords)> = Vec::new();
        for group in self.get_group(All) {
            let value_cells: Vec<&Coords> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                .collect();
            if value_cells.len() == 2 {
                strong_links.push((*value_cells[0], *value_cells[1]));
            }
        }
        strong_links
    }

    // CREATION

    pub fn new(n: usize) -> Self {
        let n2 = n * n;
        let board = vec![vec![0; n2]; n2];
        let possibility_board = vec![vec![(1..=n2).collect(); n2]; n2];
        let difficulty = Unknown;
        let error = None;

        if GROUPS.read().unwrap().contains_key(&n) && CELL_GROUPS.read().unwrap().contains_key(&n) {
            return Self {
                n,
                n2,
                board,
                possibility_board,
                difficulty,
                error,
            };
        }

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
                cell_groups.insert(((x, y), Row), row);
                cell_groups.insert(((x, y), Column), col);
                cell_groups.insert(((x, y), Square), square);
                cell_groups.insert(((x, y), Lines), lines);
                cell_groups.insert(((x, y), All), all);
            }
        }

        let mut groups = HashMap::new();
        groups.insert(Row, rows);
        groups.insert(Column, cols);
        groups.insert(Lines, lines);
        groups.insert(Square, squares);
        groups.insert(All, all);

        GROUPS.write().unwrap().insert(n, groups);
        CELL_GROUPS.write().unwrap().insert(n, cell_groups);

        Self {
            n,
            n2,
            board,
            possibility_board,
            difficulty,
            error,
        }
    }

    pub fn generate_full(n: usize) -> Self {
        let mut sudoku = Self::new(n);
        sudoku.backtrack_solve(0, 0);
        sudoku
    }

    pub fn solve(&mut self) -> Vec<Vec<usize>> {
        let mut sudoku = self.clone();
        loop {
            match sudoku.rule_solve(None, None) {
                Ok(None) => break,
                Ok(_) => (),
                Err(((x1, y1), (x2, y2))) => eprintln!("Error: {x1},{y1} == {x2},{y2}"),
            }
        }
        if sudoku.is_solved() {
            info!("Sudoku solved !");
        } else {
            warn!("Sudoku not solved !");
        }
        sudoku.get_board()
    }

    /*
    ->	ORIGINAL					->	(n^4)! + (n^4-1)! + ... + 1!
    ->	CALCULABILITY THRESHOLD		->	(n^4)! + (n^4-1)! + ... + 17!
    ->	REMOVE REDUNDANCY 			->	(n^4)! - (n^4-1)! - ... - 17!
    */
    pub fn generate(n: usize, aimed_difficulty: SudokuDifficulty) -> Self {
        let n2 = n * n;
        let start = std::time::Instant::now();
        let thread_count: usize = available_parallelism().unwrap().get();
        let (tx, rx) = mpsc::channel();
        type SudokuFilledCells = (Sudoku, Vec<bool>);

        loop {
            let default = Arc::new(Mutex::new((Self::generate_full(n), vec![true; n2 * n2])));
            let to_explore: Arc<Mutex<Vec<SudokuFilledCells>>> = Arc::new(Mutex::new(Vec::new()));
            let explored_filled_cells: Arc<Mutex<HashSet<Vec<bool>>>> =
                Arc::new(Mutex::new(HashSet::new()));
            let total: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
            let skipped: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

            let mut threads_infos: Vec<(JoinHandle<()>, mpsc::Sender<()>)> = Vec::new();
            for _ in 0..thread_count {
                let thread_default = Arc::clone(&default);
                let thread_to_explore = Arc::clone(&to_explore);
                let thread_explored_filled_cells = Arc::clone(&explored_filled_cells);
                let thread_total = Arc::clone(&total);
                let thread_skipped = Arc::clone(&skipped);
                let thread_tx = tx.clone();

                let (main_tx, thread_rx) = mpsc::channel();

                let join_handle = std::thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    while thread_rx.try_recv().is_err() {
                        let (sudoku, mut filled_cells) = thread_to_explore
                            .lock()
                            .unwrap()
                            .pop()
                            .unwrap_or(thread_default.lock().unwrap().clone());

                        (*thread_total.lock().unwrap()).add_assign(1);
                        print!(
                            "Skipped {}/{} instances with {} filled cells            \r",
                            thread_skipped.lock().unwrap(),
                            thread_total.lock().unwrap(),
                            filled_cells.iter().filter(|b| **b).count()
                        );
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();

                        let mut i1 = rng.gen_range(0..filled_cells.len());
                        let mut i2 = rng.gen_range(0..filled_cells.len());
                        loop {
                            if !filled_cells[i1] {
                                i1 = rng.gen_range(0..filled_cells.len());
                                continue;
                            }
                            if !filled_cells[i2] {
                                i2 = rng.gen_range(0..filled_cells.len());
                                continue;
                            }
                            if i1 == i2 {
                                i2 = rng.gen_range(0..filled_cells.len());
                                continue;
                            }
                            break;
                        }
                        for i in [i1, i2] {
                            let x = i % n2;
                            let y = i / n2;
                            let mut testing_sudoku = sudoku.clone();
                            let removed_value = testing_sudoku.remove_value(x, y);
                            filled_cells[i] = false;

                            if thread_explored_filled_cells
                                .lock()
                                .unwrap()
                                .contains(&filled_cells)
                            {
                                (*thread_skipped.lock().unwrap()).add_assign(1);
                                (*thread_total.lock().unwrap()).add_assign(1);
                                print!(
                                    "Skipped {}/{} instances with {} filled cells            \r",
                                    thread_skipped.lock().unwrap(),
                                    thread_total.lock().unwrap(),
                                    filled_cells.iter().filter(|b| **b).count()
                                );
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                filled_cells[i] = true;
                                continue;
                            }

                            let mut can_solve: bool = false;
                            loop {
                                match testing_sudoku.rule_solve(None, Some(aimed_difficulty)) {
                                    Ok(Some(0 | 1)) => {
                                        if testing_sudoku.board[y][x] == removed_value {
                                            can_solve = true;
                                            break;
                                        }

                                        let testing_filled_cells: Vec<bool> = (0..sudoku.n2
                                            * sudoku.n2)
                                            .map(|i| {
                                                testing_sudoku.board[i / sudoku.n2][i % sudoku.n2]
                                                    .ne(&0)
                                            })
                                            .collect();
                                        if thread_explored_filled_cells
                                            .lock()
                                            .unwrap()
                                            .contains(&testing_filled_cells)
                                        {
                                            (*thread_skipped.lock().unwrap()).add_assign(1);
                                            (*thread_total.lock().unwrap()).add_assign(1);
                                            print!(
                    						"Skipped {}/{} instances with {} filled cells            \r",
                    						thread_skipped.lock().unwrap(),
                    						thread_total.lock().unwrap(),
                    						filled_cells.iter().filter(|b| **b).count()
                    					);
                                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                            break;
                                        }
                                    }
                                    Ok(Some(_rule_used)) => (),
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            if !can_solve {
                                filled_cells[i] = true;
                                continue;
                            }

                            let mut passed_sudoku = sudoku.clone();
                            passed_sudoku.remove_value(x, y);
                            passed_sudoku.difficulty = testing_sudoku.difficulty;

                            if passed_sudoku.difficulty == aimed_difficulty {
                                passed_sudoku.difficulty = Unknown;
                                let _ = thread_tx.send(Some(passed_sudoku));
                                return;
                            } else if filled_cells.len() < n2 * 2 - 1
                                || passed_sudoku.difficulty > aimed_difficulty
                            {
                                thread_explored_filled_cells
                                    .lock()
                                    .unwrap()
                                    .insert(filled_cells.clone());
                            } else {
                                // EXPLORATION EN PROFONDEUR
                                thread_to_explore
                                    .lock()
                                    .unwrap()
                                    .push((passed_sudoku, filled_cells.clone()));
                            }

                            filled_cells[i] = true;
                        }

                        thread_explored_filled_cells
                            .lock()
                            .unwrap()
                            .insert(filled_cells);
                    }
                });
                threads_infos.push((join_handle, main_tx));
            }

            let mut unwrapped_sudoku = 0;
            while unwrapped_sudoku < thread_count {
                let sudoku = rx.recv().unwrap().unwrap();
                unwrapped_sudoku += 1;

                if sudoku.is_unique() {
                    println!(
                        "Skipped {} sudokus, Explored {} possibilities, in {} seconds",
                        skipped.lock().unwrap(),
                        explored_filled_cells.lock().unwrap().len(),
                        start.elapsed().as_secs_f32()
                    );

                    for (handle, tx) in threads_infos {
                        let _ = tx.send(());
                        handle.join().unwrap();
                    }

                    return sudoku;
                }
                println!("GENERATION HAD TO LOOP");
            }
        }
    }

    pub fn parse_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = {
            let mut path_builder = current_dir().unwrap();
            path_builder.push("res/sudoku_samples/");
            path_builder.push(file_name);
            path_builder.into_os_string().into_string().unwrap()
        };
        let file_content = std::fs::read_to_string(file_path)?;
        Self::parse_string(&file_content)
    }

    pub fn parse_string(string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut lines = string.lines();
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

        if let Some(((x1, y1), (x2, y2))) = sudoku.get_error() {
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
        max_difficulty: Option<SudokuDifficulty>,
    ) -> Result<Option<usize>, (Coords, Coords)> {
        let rules: Vec<_> = Sudoku::RULES
            .iter()
            .filter(|(rule_id, difficulty, _rule)| {
                let range_filter = if let Some(range) = &specific_rules {
                    range.contains(rule_id)
                } else {
                    true
                };
                let difficulty_filter = if let Some(max_difficulty) = max_difficulty {
                    *difficulty <= max_difficulty
                } else {
                    *difficulty < Unimplemented
                };
                range_filter
                    && difficulty_filter
                    && *difficulty != Unimplemented
                    && *difficulty != Useless
            })
            .collect();

        let mut rule_used: Option<usize> = None;
        // try the rules and set the difficulty in consequence
        for &&(rule_id, difficulty, rule) in rules.iter() {
            // if the rule can't be applied, then pass to the next one
            if !rule(self) {
                continue;
            }

            debug_only!("règle {} appliquée", rule_id);
            debug_only!("Sudoku actuel:\n{}", self);

            rule_used = Some(rule_id);
            self.difficulty = max(self.difficulty, difficulty);
            if let Some(err) = self.error {
                return Err(err);
            }
            break;
        }

        Ok(rule_used)
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
        let cell_group: HashSet<Coords> = self.get_cell_group(x, y, All).clone();
        self.possibility_board[y][x].clear();
        for value in possible_values.clone().into_iter() {
            self.board[y][x] = value;
            let changing_cells: HashSet<&Coords> = cell_group
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

        false
    }

    // UTILITY

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

    pub fn is_unique(&self) -> bool {
        let mut solutions = 0;
        self.clone()._is_unique(0, 0, &mut solutions);
        solutions <= 1
    }

    fn _is_unique(&mut self, mut x: usize, mut y: usize, solutions: &mut usize) {
        if y == self.n2 - 1 && x == self.n2 {
            solutions.add_assign(1);
            return;
        }

        while self.board[y][x] != 0 {
            x += 1;

            if y == self.n2 - 1 && x == self.n2 {
                solutions.add_assign(1);
                return;
            }

            if x == self.n2 {
                y += 1;
                x = 0;
            }
        }

        let possible_values = self.possibility_board[y][x].clone();
        for value in possible_values.clone().into_iter() {
            self.set_value(x, y, value);

            self._is_unique(x, y, solutions);
            if *solutions > 1 {
                return;
            }

            self.remove_value(x, y);
        }
        self.possibility_board[y][x] = possible_values;
    }
}

const BASE_64: [char; 65] = [
    '·', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', 'α', 'β', 'δ',
];

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                                BASE_64[self.board[y][x]],
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
                            BASE_64[value]
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
        write!(f, "{}", lines.join("\n"))
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
        Self {
            n: self.n,
            n2: self.n2,
            board: self.board.clone(),
            possibility_board: self.possibility_board.clone(),
            difficulty: self.difficulty,
            error: self.error,
        }
    }
}
