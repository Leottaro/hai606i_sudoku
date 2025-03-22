use super::{
    CellGroupMap, Coords, GroupMap, Sudoku,
    SudokuDifficulty::{self, *},
    SudokuError,
    SudokuGroups::{self, *},
};
use crate::{
    database::{
        DBNewSimpleSudokuGame, DBSimpleSudokuCanonical, DBSimpleSudokuCanonicalSquares, Database,
    },
    debug_only,
};
use log::{info, warn};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{AddAssign, Range},
    sync::{mpsc, Arc, LazyLock, Mutex, RwLock},
    thread::{available_parallelism, JoinHandle},
};

static GROUPS: LazyLock<RwLock<HashMap<usize, GroupMap>>> = LazyLock::new(Default::default);
static CELL_GROUPS: LazyLock<RwLock<HashMap<usize, CellGroupMap>>> =
    LazyLock::new(Default::default);

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

    pub fn get_error(&self) -> Option<SudokuError> {
        self.error
    }

    pub fn is_canonical(&self) -> bool {
        self.is_canonical
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) -> Result<(), SudokuError> {
        if self.board[y][x] == value {
            warn!("tried to set cell ({},{}) to its own value", x, y);
            return Ok(());
        }
        if self.board[y][x] != 0 {
            panic!("Cannot set an already setted value");
        }
        self.filled_cells += 1;
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for (x1, y1) in self.get_cell_group(x, y, All) {
            self.possibility_board[y1][x1].remove(&value);
            if self.board[y1][x1] == value && (x, y) != (x1, y1) {
                let error = SudokuError::SameValueCells(((x, y), (x1, y1)));
                self.error = Some(error);
                return Err(error);
            } else if self.board[y1][x1] == 0 && self.possibility_board[y1][x1].is_empty() {
                let error = SudokuError::NoPossibilityCell((x1, y1));
                self.error = Some(error);
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn remove_value(&mut self, x: usize, y: usize) -> usize {
        if self.is_canonical {
            panic!("Cannot modify a canonical sudoku !");
        }
        if self.board[y][x] == 0 {
            panic!("Cannot remove an already empty value");
        }
        let removed_value = self.board[y][x];

        self.filled_cells -= 1;
        self.board[y][x] = 0;
        self.possibility_board[y][x] = (1..=self.n2).collect();

        for (x1, y1) in self.get_cell_group(x, y, All) {
            match self.error {
                Some(SudokuError::SameValueCells((cell1, cell2))) => {
                    if (cell1.eq(&(x, y)) && cell2.eq(&(x1, y1)))
                        || (cell2.eq(&(x, y)) && cell1.eq(&(x1, y1)))
                    {
                        self.error = None;
                    }
                }
                Some(SudokuError::NoPossibilityCell(cell)) => {
                    if cell.eq(&(x1, y1)) {
                        self.error = None;
                    }
                }
                _ => (),
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

    pub fn is_same_group(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
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
        let is_canonical = false;
        let filled_cells = 0;
        let canonical_board_hash = 0;

        if GROUPS.read().unwrap().contains_key(&n) && CELL_GROUPS.read().unwrap().contains_key(&n) {
            return Self {
                n,
                n2,
                board,
                possibility_board,
                filled_cells,
                difficulty,
                error,
                is_canonical,
                canonical_board_hash,
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
            is_canonical,
            filled_cells,
            canonical_board_hash,
        }
    }

    pub fn generate_full(n: usize) -> Self {
        let mut sudoku = Self::new(n);
        let mut rng = thread_rng();

        // upper row is 1 2 3 4 .....
        for x in 0..sudoku.n2 {
            sudoku.set_value(x, 0, x + 1).unwrap();
        }

        // left column filling (with each squares separated)
        let mut column_squares: Vec<Vec<usize>> = Vec::new();

        // upper left square can't have the n first values
        let mut possible_values = {
            let mut temp = (n + 1..=sudoku.n2).collect::<Vec<_>>();
            temp.shuffle(&mut rng);
            temp
        };

        // so we extract the first square
        let mut first_square = vec![1];
        for _y in 1..n {
            first_square.push(possible_values.pop().unwrap());
        }
        column_squares.push(first_square);

        // add the first n values to the options and re shuffle
        possible_values.extend(2..=sudoku.n);
        possible_values.shuffle(&mut rng);

        // extract the remaining squares from those values
        column_squares.extend(possible_values.chunks(n).map(|square| square.to_vec()));

        // within each square, sort the values (rows) in an ascending order
        for square in column_squares.iter_mut() {
            square.sort();
        }

        // sort the squares by first value
        column_squares.sort_by(|square1, square2| square1[0].cmp(&square2[0]));

        // get the rest of the final column and fill it
        let column = column_squares.into_iter().flatten();
        for (y, value) in column.enumerate() {
            sudoku.set_value(0, y, value).unwrap();
        }

        // fill the rest of the sudoku
        sudoku.backtrack_solve(0, 0);

        // get the canonical board hash
        sudoku.canonical_board_hash = {
            let mut hasher = DefaultHasher::new();
            sudoku.board.hash(&mut hasher);
            hasher.finish()
        };

        sudoku.is_canonical = true;
        sudoku
    }

    pub fn randomize(&mut self) {
        if !self.is_filled() {
            panic!("Cannot randomize a non filled sudoku !");
        }
        let mut rng = thread_rng();

        ////////////////////////////////////////////////////
        // swap rows randomly (keep the first line in place)
        let first_line = 0;
        let mut first_floor = (1..self.n).collect::<Vec<_>>();
        let mut floors = (self.n..self.n2)
            .collect::<Vec<_>>()
            .chunks(self.n)
            .map(|floor| floor.to_vec())
            .collect::<Vec<_>>();

        // shuffle each floor (not the first floor)
        floors.shuffle(&mut rng);

        // shuffle each row inside a floor (not the first row)
        first_floor.shuffle(&mut rng);
        for floor in floors.iter_mut() {
            floor.shuffle(&mut rng);
        }

        let rows_swap = {
            let mut temp = vec![first_line];
            temp.extend(first_floor);
            temp.extend(floors.into_iter().flatten());
            temp
        };

        self.board = rows_swap
            .into_iter()
            .map(|shuffled_y| self.board[shuffled_y].clone())
            .collect();

        ////////////////////////////////////////////////////
        // swap random values
        let values_swap: HashMap<usize, usize> = {
            let mut values_input = (1..=self.n2).collect::<Vec<_>>();
            let mut values_output = values_input.clone();
            values_input.shuffle(&mut rng);
            values_output.shuffle(&mut rng);
            values_input.into_iter().zip(values_output).collect()
        };

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                self.board[y][x] = *values_swap.get(&self.board[y][x]).unwrap();
            }
        }

        self.is_canonical = false
    }

    pub fn canonize(&mut self) {
        if !self.is_filled() {
            panic!("Cannot canonize a non filled sudoku !");
        }
        if self.is_canonical {
            panic!("Cannot canonize an already canonical sudoku !");
        }

        ///////////////////////////////////////////////////
        // swap values to get 1 2 3 4 5... in the first row
        let values_swap: HashMap<usize, usize> =
            (0..self.n2).map(|x| (self.board[0][x], x + 1)).collect();

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                self.board[y][x] = *values_swap.get(&self.board[y][x]).unwrap();
            }
        }

        ///////////////////////////////////////////////////
        // swap rows logically

        // extract floors
        let mut floors = self
            .board
            .chunks(self.n)
            .map(|floor| floor.to_vec())
            .collect::<Vec<_>>();

        // within each floor, sort the rows by their first cell
        for floor in floors.iter_mut() {
            floor.sort_by(|row1, row2| row1[0].cmp(&row2[0]));
        }

        // sort the rows batch by their upper left cell
        floors.sort_by(|floor1, floor2| floor1[0][0].cmp(&floor2[0][0]));

        self.board = floors.into_iter().flatten().collect::<Vec<_>>();

        ////////////////////////////////////////////////////////////////////
        // check if the board is the same as the hash of the canonical board
        let board_hash = {
            let mut hasher = DefaultHasher::new();
            self.board.hash(&mut hasher);
            hasher.finish()
        };

        if board_hash != self.canonical_board_hash {
            panic!(
                "SudokuError: Canonization mismatch: expected hash {}, got hash {}",
                self.canonical_board_hash, board_hash
            );
        }
    }

    /*
    ->	ORIGINAL					->	(n^4)! + (n^4-1)! + ... + 1!
    ->	CALCULABILITY THRESHOLD		->	(n^4)! + (n^4-1)! + ... + 17!
    ->	REMOVE REDUNDANCY 			->	(n^4)! - (n^4-1)! - ... - 17!
    ->  ONLY 2 POSSIBILITIES		->  2! + 2! + ... + 17! (un peu moins que ça grâce à REMOVE REDUNDANCY)
    */
    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Self {
        if self.is_canonical {
            panic!("Cannot generate a sudoku from a canonical one");
        }

        let n2 = self.n2;
        let (tx, rx) = mpsc::channel();
        type SudokuFilledCells = (Sudoku, Vec<bool>);

        loop {
            let thread_count: usize = available_parallelism().unwrap().get();
            let default = {
                let filled_cells: Vec<bool> = (0..n2 * n2)
                    .map(|i| self.get_cell_value(i % n2, i / n2) != 0)
                    .collect();
                Arc::new(Mutex::new((self.clone(), filled_cells)))
            };
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
                        let (mut sudoku, filled_cells) = thread_to_explore
                            .lock()
                            .unwrap()
                            .pop()
                            .unwrap_or(thread_default.lock().unwrap().clone());

                        (*thread_total.lock().unwrap()).add_assign(1);
                        print!(
                            " Skipped {}/{} instances with {} filled cells{}\r",
                            thread_skipped.lock().unwrap(),
                            thread_total.lock().unwrap(),
                            filled_cells.iter().filter(|b| **b).count(),
                            " ".repeat(20)
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

                        let mut working_sub_sudokus = 0;
                        for i in [i1, i2] {
                            let x = i % n2;
                            let y = i / n2;
                            let mut testing_sudoku = sudoku.clone();
                            testing_sudoku.difficulty = Unknown;
                            let removed_value = testing_sudoku.remove_value(x, y);

                            if thread_explored_filled_cells
                                .lock()
                                .unwrap()
                                .contains(&filled_cells)
                            {
                                (*thread_skipped.lock().unwrap()).add_assign(1);
                                (*thread_total.lock().unwrap()).add_assign(1);
                                print!(
                                    " Skipped {}/{} instances with {} filled cells{}\r",
                                    thread_skipped.lock().unwrap(),
                                    thread_total.lock().unwrap(),
                                    filled_cells.iter().filter(|b| **b).count(),
                                    " ".repeat(20)
                                );
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
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
                                                " Skipped {}/{} instances with {} filled cells{}\r",
                                                thread_skipped.lock().unwrap(),
                                                thread_total.lock().unwrap(),
                                                filled_cells.iter().filter(|b| **b).count(),
                                                " ".repeat(20)
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
                                continue;
                            }

                            if testing_sudoku.filled_cells < n2 * 2 - 1 {
                                thread_explored_filled_cells
                                    .lock()
                                    .unwrap()
                                    .insert(filled_cells.clone());
                            } else {
                                // EXPLORATION EN PROFONDEUR
                                let mut passed_sudoku = sudoku.clone();
                                passed_sudoku.remove_value(x, y);
                                passed_sudoku.difficulty = testing_sudoku.difficulty;

                                let mut passed_filled_cells = filled_cells.clone();
                                passed_filled_cells[i] = false;

                                thread_to_explore
                                    .lock()
                                    .unwrap()
                                    .push((passed_sudoku, passed_filled_cells));

                                working_sub_sudokus += 1;
                            }
                        }

                        if working_sub_sudokus == 0 && sudoku.difficulty == aimed_difficulty {
                            sudoku.difficulty = Unknown;
                            let _ = thread_tx.send(Some(sudoku));
                            return;
                        }

                        thread_explored_filled_cells
                            .lock()
                            .unwrap()
                            .insert(filled_cells);
                    }
                });
                threads_infos.push((join_handle, main_tx));
            }

            for _ in 0..thread_count {
                let sudoku = rx.recv().unwrap().unwrap();

                // verify that the sudoku is unique
                if !sudoku.is_unique() {
                    continue;
                }

                // panic if generated a wrong sudoku
                let mut verify_sudoku = sudoku.clone();
                loop {
                    match verify_sudoku.rule_solve(None, None) {
                        Ok(Some(_)) => (),
                        Ok(None) => {
                            if !verify_sudoku.is_filled() {
                                panic!("ERROR IN SUDOKU SOLVING: Couldn't solve generated sudoku: \nORIGINAL SUDOKU:\n{sudoku}\nFINISHED SUDOKU: \n{verify_sudoku}");
                            }
                            break;
                        }
                        Err(err) => {
                            panic!("ERROR IN SUDOKU: {err}: \nORIGINAL SUDOKU: {sudoku}");
                        }
                    }
                }

                for (handle, tx) in threads_infos {
                    let _ = tx.send(());
                    handle.join().unwrap();
                }
                return sudoku;
            }
        }
    }

    pub fn generate_new(n: usize, aimed_difficulty: SudokuDifficulty) -> Self {
        let mut sudoku_base = Sudoku::generate_full(n);
        sudoku_base.randomize();
        sudoku_base.generate_from(aimed_difficulty)
    }

    pub fn parse_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file_path = current_dir().unwrap();
        file_path.push("res/sudoku_samples/");
        file_path.push(file_name);
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
                sudoku.set_value(x, y, value).unwrap();
            }
        }

        if let Some(SudokuError::SameValueCells(((x1, y1), (x2, y2)))) = sudoku.get_error() {
            return Err(format!(
				"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in the file",
				x1, y1, x2, y2
			)
			.into());
        }
        Ok(sudoku)
    }

    pub fn board_to_string(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}", self.n));
        for line in self.board.iter() {
            lines.push(
                line.iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        lines.join("\n")
    }

    // RULE SOLVING

    pub fn rule_solve(
        &mut self,
        specific_rules: Option<Range<usize>>,
        max_difficulty: Option<SudokuDifficulty>,
    ) -> Result<Option<usize>, SudokuError> {
        if self.is_canonical {
            panic!("Cannot modify a canonical sudoku !");
        }

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

    pub fn solve(&mut self) -> Vec<Vec<usize>> {
        let mut sudoku = self.clone();
        loop {
            match sudoku.rule_solve(None, None) {
                Ok(None) => break,
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{err}");
                    break;
                }
            }
        }
        if sudoku.is_filled() {
            info!("Sudoku solved !");
        } else {
            warn!("Sudoku not solved !");
        }
        sudoku.get_board()
    }

    // BACKTRACK SOLVING

    pub fn backtrack_solve(&mut self, mut x: usize, mut y: usize) -> bool {
        if self.is_canonical {
            panic!("Cannot modify a canonical sudoku !");
        }

        loop {
            if y == self.n2 - 1 && x == self.n2 {
                return true;
            }

            if x == self.n2 {
                y += 1;
                x = 0;
            }

            if self.board[y][x] == 0 {
                break;
            }
            x += 1;
        }

        let mut possibilities = self.possibility_board[y][x]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut thread_rng());
        for value in possibilities {
            if self.set_value(x, y, value).is_err() {
                self.remove_value(x, y);
                continue;
            }

            if self.backtrack_solve(x + 1, y) {
                return true;
            }

            self.remove_value(x, y);
        }

        false
    }

    // UTILITY

    pub fn is_filled(&self) -> bool {
        self.filled_cells == self.n2 * self.n2
    }

    pub fn is_unique(&self) -> bool {
        let mut solutions = 0;
        self.clone()._is_unique(0, 0, &mut solutions);
        solutions <= 1
    }

    fn _is_unique(&mut self, mut x: usize, mut y: usize, solutions: &mut usize) {
        loop {
            if y == self.n2 - 1 && x == self.n2 {
                solutions.add_assign(1);
                return;
            }

            if x == self.n2 {
                y += 1;
                x = 0;
            }

            if self.board[y][x] == 0 {
                break;
            }
            x += 1;
        }

        let possible_values = self.possibility_board[y][x].clone();
        for value in possible_values.clone().into_iter() {
            if self.set_value(x, y, value).is_err() {
                self.remove_value(x, y);
                continue;
            }

            self._is_unique(x, y, solutions);
            if *solutions > 1 {
                return;
            }

            self.remove_value(x, y);
        }
        self.possibility_board[y][x] = possible_values;
    }

    // DATABASE

    pub fn canonical_to_db(
        &self,
    ) -> (DBSimpleSudokuCanonical, Vec<DBSimpleSudokuCanonicalSquares>) {
        if !self.is_canonical {
            panic!("Can't get the canonical db with a randomized sudoku");
        }

        let board: Vec<u8> = self
            .board
            .iter()
            .flat_map(|line| line.iter().map(|cell| *cell as u8))
            .collect();

        let simple_sudoku_canonical = DBSimpleSudokuCanonical {
            canonical_board_hash: self.canonical_board_hash,
            sudoku_n: self.n as u8,
            canonical_board: board,
        };

        let mut simple_sudoku_canonical_squares = Vec::new();
        for y0 in 0..self.n {
            for x0 in 0..self.n {
                let square_id = y0 * self.n + x0;
                let mut hasher = DefaultHasher::new();
                for y in 0..self.n {
                    for x in 0..self.n {
                        (self.board[y0 * self.n + y][x0 * self.n + x] as u8).hash(&mut hasher);
                    }
                }
                simple_sudoku_canonical_squares.push(DBSimpleSudokuCanonicalSquares {
                    canonical_board_hash: self.canonical_board_hash,
                    square_id: square_id as u8,
                    square_hash: hasher.finish(),
                });
            }
        }
        (simple_sudoku_canonical, simple_sudoku_canonical_squares)
    }

    pub fn randomized_to_db(&self) -> DBNewSimpleSudokuGame {
        if self.is_canonical {
            panic!("Can't get the game db with a canonical sudoku");
        }

        let board: Vec<u8> = self
            .board
            .iter()
            .flat_map(|line| line.iter().map(|cell| *cell as u8))
            .collect();
        DBNewSimpleSudokuGame {
            canonical_board_hash: self.canonical_board_hash,
            game_n: self.n as u8,
            game_board: board,
            game_difficulty: self.difficulty as u8,
            game_filled_cells: self.filled_cells as u8,
        }
    }

    pub fn load_canonical_from_db(database: &mut Database, n: usize) -> Self {
        database.get_random_simple_sudoku_canonical(n).unwrap()
    }

    pub fn load_game_from_db(
        database: &mut Database,
        n: usize,
        difficulty: SudokuDifficulty,
    ) -> Self {
        database
            .get_random_simple_sudoku_game(n, difficulty)
            .unwrap()
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
        lines.push(format!("DIFFICULTY: {}", self.difficulty));

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
                if self.board[y][x].ne(&other.board[y][x])
                    || self.possibility_board[y][x].ne(&other.possibility_board[y][x])
                {
                    return false;
                }
            }
        }

        true
    }
}
