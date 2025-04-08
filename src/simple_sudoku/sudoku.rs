use super::{
    CellGroupMap,
    Coords,
    GroupMap,
    Sudoku,
    SudokuDifficulty::{ self, * },
    SudokuError,
    SudokuGroups::{ self, * },
};
#[cfg(feature = "database")]
use crate::database::{
    DBNewSimpleSudokuGame,
    DBSimpleSudokuCanonical,
    DBSimpleSudokuCanonicalSquares,
    Database,
};
use crate::debug_only;
use log::{ info, warn };
use rand::{ seq::SliceRandom, thread_rng, Rng };
use std::{
    cmp::max,
    collections::{ HashMap, HashSet },
    env::current_dir,
    hash::{ DefaultHasher, Hash, Hasher },
    ops::{ AddAssign, Range },
    sync::{ mpsc, Arc, LazyLock, Mutex, RwLock },
    thread::{ available_parallelism, JoinHandle },
};

static GROUPS: LazyLock<RwLock<HashMap<usize, GroupMap>>> = LazyLock::new(Default::default);
static CELL_GROUPS: LazyLock<RwLock<HashMap<usize, CellGroupMap>>> = LazyLock::new(
    Default::default
);

impl Sudoku {
    ///////////////////////////////////////////////////////////////////////////////////////////////////
    // GETTERS / SETTERS
    pub fn get_n(&self) -> usize {
        self.n
    }

    pub fn get_n2(&self) -> usize {
        self.n2
    }

    pub fn get_board(&self) -> &Vec<Vec<usize>> {
        &self.board
    }

    pub fn get_possibility_board(&self) -> &Vec<Vec<HashSet<usize>>> {
        &self.possibility_board
    }

    pub fn get_filled_cells(&self) -> usize {
        self.filled_cells
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
    pub fn get_cell_possibilities_mut(&mut self, x: usize, y: usize) -> &mut HashSet<usize> {
        &mut self.possibility_board[y][x]
    }

    pub fn get_group(&self, groups: SudokuGroups) -> Vec<HashSet<Coords>> {
        GROUPS.read().unwrap()[&self.n][&groups].clone()
    }

    pub fn get_cell_group(&self, x: usize, y: usize, groups: SudokuGroups) -> HashSet<Coords> {
        CELL_GROUPS.read().unwrap()[&self.n][&((x, y), groups)].clone()
    }

    pub fn get_cell_groups(
        &self,
        x: usize,
        y: usize,
        groups: Vec<SudokuGroups>
    ) -> Vec<HashSet<Coords>> {
        let owned_cell_groups = &CELL_GROUPS.read().unwrap()[&self.n];
        groups
            .into_iter()
            .map(|group| owned_cell_groups[&((x, y), group)].clone())
            .collect()
    }

    pub fn is_canonical(&self) -> bool {
        self.is_canonical
    }

    pub fn get_values_swap(&self) -> HashMap<usize, (usize, usize)> {
        self.values_swap.clone()
    }

    pub fn get_rows_swap(&self) -> HashMap<usize, (usize, usize)> {
        self.rows_swap.clone()
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) -> Result<(), SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(
                SudokuError::WrongInput(
                    format!("set_value({x}, {y}, {value}); value should be in [1..{}]", self.n2)
                )
            );
        }
        if self.board[y][x] != 0 {
            return Err(
                SudokuError::InvalidState(
                    format!("set_value({x}, {y}, {value}) when board[y][x] = {}", self.board[y][x])
                )
            );
        }

        self.filled_cells += 1;
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        let mut res = Ok(());
        for (x1, y1) in self.get_cell_group(x, y, All) {
            self.possibility_board[y1][x1].remove(&value);
            if self.board[y1][x1] == value && (x, y) != (x1, y1) {
                res = Err(SudokuError::SameValueCells(((x, y), (x1, y1))));
            } else if self.board[y1][x1] == 0 && self.possibility_board[y1][x1].is_empty() {
                res = Err(SudokuError::NoPossibilityCell((x1, y1)));
            }
        }
        res
    }

    pub fn insert_possibility(
        &mut self,
        x: usize,
        y: usize,
        value: usize
    ) -> Result<bool, SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(
                SudokuError::WrongInput(
                    format!("set_value({x}, {y}, {value}); value should be in [1..{}]", self.n2)
                )
            );
        }
        if self.board[y][x] != 0 {
            return Err(
                SudokuError::InvalidState(
                    format!("remove_value({x}, {y}) when board[y][x] = {}", self.board[y][x])
                )
            );
        }

        Ok(self.possibility_board[y][x].insert(value))
    }

    pub fn remove_possibility(
        &mut self,
        x: usize,
        y: usize,
        value: usize
    ) -> Result<bool, SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(
                SudokuError::WrongInput(
                    format!(
                        "remove_possibility({x}, {y}, {value}); value should be in [1..{}]",
                        self.n2
                    )
                )
            );
        }
        if self.board[y][x] != 0 {
            return Err(
                SudokuError::InvalidState(
                    format!(
                        "remove_possibility({x}, {y}, {value}) when board[y][x] = {}",
                        self.board[y][x]
                    )
                )
            );
        }

        Ok(self.possibility_board[y][x].remove(&value))
    }

    pub fn remove_value(&mut self, x: usize, y: usize) -> Result<usize, SudokuError> {
        if self.board[y][x] == 0 {
            return Err(
                SudokuError::InvalidState(
                    format!("remove_value({x}, {y}) when board[y][x] = {}", self.board[y][x])
                )
            );
        }
        let removed_value = self.board[y][x];

        self.filled_cells -= 1;
        self.board[y][x] = 0;
        self.possibility_board[y][x] = (1..=self.n2).collect();

        for (x1, y1) in self.get_cell_group(x, y, All) {
            if self.board[y1][x1] != 0 {
                self.possibility_board[y][x].remove(&self.board[y1][x1]);
                continue;
            }

            if
                self
                    .get_cell_group(x1, y1, All)
                    .iter()
                    .all(|&(x2, y2)| self.board[y2][x2] != removed_value)
            {
                self.possibility_board[y1][x1].insert(removed_value);
            }
        }

        Ok(removed_value)
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

    ///////////////////////////////////////////////////////////////////////////////////////////////////
    // CREATION

    pub fn new(n: usize) -> Self {
        let n2 = n * n;
        let board = vec![vec![0; n2]; n2];
        let possibility_board = vec![vec![(1..=n2).collect(); n2]; n2];
        let difficulty = Unknown;
        let is_canonical = false;
        let filled_cells = 0;
        let canonical_board_hash = 0;
        let values_swap = HashMap::new();
        let rows_swap = HashMap::new();

        if GROUPS.read().unwrap().contains_key(&n) && CELL_GROUPS.read().unwrap().contains_key(&n) {
            return Self {
                n,
                n2,
                board,
                possibility_board,
                filled_cells,
                difficulty,

                is_canonical,
                canonical_board_hash,
                values_swap,
                rows_swap,
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
        for y0 in (0..n2).step_by(n) {
            for x0 in (0..n2).step_by(n) {
                let mut square = HashSet::new();
                for y in y0..y0 + n {
                    for x in x0..x0 + n {
                        square.insert((x, y));
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
                let square = squares[(y / n) * n + x / n].clone();
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
            filled_cells,

            is_canonical,
            canonical_board_hash,
            values_swap,
            rows_swap,
        }
    }

    pub fn generate_full(n: usize) -> Self {
        Self::new(n).into_generate_full_from()
    }

    pub fn generate_full_from(&mut self) -> Self {
        self.clone().into_generate_full_from()
    }

    pub fn into_generate_full_from(mut self) -> Self {
        // fill the upper row with an ascending order
        let first_line_possibilities = (0..self.n2)
            .map(|x| {
                let mut vec = self.possibility_board[0][x].iter().cloned().collect::<Vec<_>>();
                vec.sort();
                vec.into_iter()
            })
            .collect::<Vec<_>>();

        let mut current_possibilities = first_line_possibilities.clone();
        let mut x = 0;
        while x < self.n2 {
            let next_value = current_possibilities[x].next();
            if next_value.is_none() {
                x -= 1;
                current_possibilities[x] = first_line_possibilities[x].clone();
                continue;
            }

            match self.set_value(x, 0, next_value.unwrap()) {
                Err(_) => {
                    let _ = self.remove_value(x, 0);
                }
                Ok(()) => {
                    x += 1;
                }
            }
        }

        // fill the left collumn with an ascending order
        let first_column_possibilities = (0..self.n2)
            .map(|y| {
                let mut vec = self.possibility_board[y][0].iter().cloned().collect::<Vec<_>>();
                vec.sort();
                vec.into_iter()
            })
            .collect::<Vec<_>>();

        let mut current_possibilities = first_column_possibilities.clone();
        let mut y = 0;
        while y < self.n2 {
            let next_value = current_possibilities[y].next();
            if next_value.is_none() {
                y -= 1;
                current_possibilities[y] = first_column_possibilities[y].clone();
                continue;
            }

            match self.set_value(0, y, next_value.unwrap()) {
                Err(_) => {
                    let _ = self.remove_value(0, y);
                }
                Ok(()) => {
                    y += 1;
                }
            }
        }

        // get the canonical board hash
        self.canonical_board_hash = {
            let mut hasher = DefaultHasher::new();
            self.board.hash(&mut hasher);
            hasher.finish()
        };

        self.is_canonical = true;
        self
    }

    pub fn randomize(
        &mut self,
        rows_swap: Option<HashMap<usize, (usize, usize)>>,
        values_swap: Option<HashMap<usize, (usize, usize)>>
    ) -> Result<(), SudokuError> {
        if !self.is_filled() {
            return Err(
                SudokuError::InvalidState(
                    format!("randomize() when this sudoku isn't filled: {self}")
                )
            );
        }
        if !self.is_canonical {
            return Err(
                SudokuError::InvalidState(
                    format!("randomize() when this sudoku is already randomized: {self}")
                )
            );
        }
        let mut rng = thread_rng();

        self.rows_swap = rows_swap.unwrap_or({
            let mut floors = (0..self.n2)
                .collect::<Vec<_>>()
                .chunks(self.n)
                .map(|floor| floor.to_vec())
                .collect::<Vec<_>>();

            // shuffle each floor (not the first floor)
            floors.shuffle(&mut rng);

            // shuffle each row inside a floor (not the first row)
            for floor in floors.iter_mut() {
                floor.shuffle(&mut rng);
            }

            let shuffled_rows = floors.into_iter().flatten().enumerate().collect::<Vec<_>>();

            let mut rows_swap = (0..self.n2).map(|y| (y, (0, 0))).collect::<HashMap<_, _>>();
            for (y, to_y) in shuffled_rows {
                rows_swap.get_mut(&y).unwrap().0 = to_y;
                rows_swap.get_mut(&to_y).unwrap().1 = y;
            }
            rows_swap
        });

        self.values_swap = values_swap.unwrap_or({
            let mut values_swap = (1..=self.n2).map(|y| (y, (0, 0))).collect::<HashMap<_, _>>();

            let mut values = (1..=self.n2).collect::<Vec<_>>();
            values.shuffle(&mut rng);

            for (i, to_value) in values.into_iter().enumerate() {
                let value = i + 1;
                values_swap.get_mut(&value).unwrap().0 = to_value;
                values_swap.get_mut(&to_value).unwrap().1 = value;
            }

            values_swap
        });

        // swap rows randomly following self.rows_swap rules
        let mut new_board = vec![Vec::new(); self.n2];
        for y in 0..self.n2 {
            let (to_y, _) = self.rows_swap[&y];
            new_board[to_y] = self.board[y].clone();
        }
        self.board = new_board;

        // swap value randomly
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let value = self.board[y][x];
                let (to_value, _) = self.values_swap[&value];
                self.board[y][x] = to_value;
            }
        }

        self.is_canonical = false;
        Ok(())
    }

    pub fn canonize(&mut self) -> Result<(), SudokuError> {
        if !self.is_filled() {
            return Err(
                SudokuError::InvalidState(
                    format!("canonize() when this sudoku isn't filled: {self}")
                )
            );
        }
        if self.is_canonical {
            return Err(
                SudokuError::InvalidState(
                    format!("canonize() when this sudoku is already canonized: {self}")
                )
            );
        }

        // swap back rows using self.rows_swap
        let mut new_board = vec![Vec::new(); self.n2];
        for y in 0..self.n2 {
            let (_, from_y) = self.rows_swap[&y];
            new_board[from_y] = self.board[y].clone();
        }
        self.board = new_board;

        // swap back values using self.values_swap
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let value = self.board[y][x];
                let (_, from_value) = self.values_swap[&value];
                self.board[y][x] = from_value;
            }
        }

        // check if the board is the same as the hash of the canonical board
        let board_hash = {
            let mut hasher = DefaultHasher::new();
            self.board.hash(&mut hasher);
            hasher.finish()
        };

        if board_hash != self.canonical_board_hash {
            Err(SudokuError::CanonizationMismatch(Box::new(self.clone()), board_hash))
        } else {
            self.is_canonical = false;
            self.rows_swap.clear();
            self.values_swap.clear();
            Ok(())
        }
    }

    /*
    ->	ORIGINAL					->	(n^4)! + (n^4-1)! + ... + 1!
    ->	CALCULABILITY THRESHOLD		->	(n^4)! + (n^4-1)! + ... + 17!
    ->	REMOVE REDUNDANCY 			->	(n^4)! - (n^4-1)! - ... - 17!
    ->  ONLY 2 POSSIBILITIES		->  2! + 2! + ... + 17! (un peu moins que ça grâce à REMOVE REDUNDANCY)
    */
    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Result<Self, SudokuError> {
        let n2 = self.n2;
        let (tx, rx) = mpsc::channel();
        type SudokuFilledCells = (Sudoku, Vec<bool>);

        loop {
            let thread_count: usize = available_parallelism().unwrap().get();
            let default = {
                let filled_cells: Vec<bool> = (0..n2 * n2)
                    .map(|i| self.board[i / n2][i % n2] != 0)
                    .collect();
                Arc::new(Mutex::new((self.clone(), filled_cells)))
            };
            let to_explore: Arc<Mutex<Vec<SudokuFilledCells>>> = Arc::new(Mutex::new(Vec::new()));
            let explored_filled_cells: Arc<Mutex<HashSet<Vec<bool>>>> = Arc::new(
                Mutex::new(HashSet::new())
            );
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
                            filled_cells
                                .iter()
                                .filter(|b| **b)
                                .count(),
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
                            let removed_value = testing_sudoku.remove_value(x, y).unwrap();

                            if thread_explored_filled_cells.lock().unwrap().contains(&filled_cells) {
                                (*thread_skipped.lock().unwrap()).add_assign(1);
                                (*thread_total.lock().unwrap()).add_assign(1);
                                print!(
                                    " Skipped {}/{} instances with {} filled cells{}\r",
                                    thread_skipped.lock().unwrap(),
                                    thread_total.lock().unwrap(),
                                    filled_cells
                                        .iter()
                                        .filter(|b| **b)
                                        .count(),
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

                                        let testing_filled_cells: Vec<bool> = (0..sudoku.n2 *
                                            sudoku.n2)
                                            .map(|i| {
                                                testing_sudoku.board[i / sudoku.n2][
                                                    i % sudoku.n2
                                                ].ne(&0)
                                            })
                                            .collect();
                                        if
                                            thread_explored_filled_cells
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
                                                filled_cells
                                                    .iter()
                                                    .filter(|b| **b)
                                                    .count(),
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
                                passed_sudoku.remove_value(x, y).unwrap();
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

                        thread_explored_filled_cells.lock().unwrap().insert(filled_cells);
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
                                panic!(
                                    "ERROR IN SUDOKU SOLVING: Couldn't solve generated sudoku: \nORIGINAL SUDOKU:\n{sudoku}\nFINISHED SUDOKU: \n{verify_sudoku}"
                                );
                            }
                            break;
                        }
                        Err(err) => {
                            panic!(
                                "ERROR IN SUDOKU: {err}: \nORIGINAL SUDOKU: {sudoku}\nLAST SUDOKU: {verify_sudoku}"
                            );
                        }
                    }
                }

                for (handle, tx) in threads_infos {
                    let _ = tx.send(());
                    handle.join().unwrap();
                }
                return Ok(sudoku);
            }
        }
    }

    pub fn generate_new(n: usize, aimed_difficulty: SudokuDifficulty) -> Result<Self, SudokuError> {
        let mut sudoku_base = Sudoku::generate_full(n);
        sudoku_base.randomize(None, None).unwrap();
        sudoku_base.generate_from(aimed_difficulty)
    }

    pub fn parse_file(file_name: &str) -> Result<Self, SudokuError> {
        let mut file_path = current_dir().unwrap();
        file_path.push("res/sudoku_samples/");
        file_path.push(file_name);
        let file_content = std::fs
            ::read_to_string(&file_path)
            .map_err(|error| {
                SudokuError::ReadFile((
                    file_path.into_os_string().into_string().unwrap(),
                    error.to_string(),
                ))
            })?;
        Self::parse_string(&file_content)
    }

    pub fn parse_string(string: &str) -> Result<Self, SudokuError> {
        let mut lines = string.lines();
        let first_line = lines.next().unwrap();
        let n = first_line
            .parse::<usize>()
            .map_err(|error| {
                SudokuError::ParseString((first_line.to_string(), error.to_string()))
            })?;

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

        Ok(sudoku)
    }

    pub fn board_to_string(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}", self.n));
        for line in self.board.iter() {
            lines.push(
                line
                    .iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        lines.join("\n")
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////
    // RULE SOLVING

    pub fn rule_solve(
        &mut self,
        specific_rules: Option<Range<usize>>,
        max_difficulty: Option<SudokuDifficulty>
    ) -> Result<Option<usize>, SudokuError> {
        let rules: Vec<_> = Sudoku::RULES.iter()
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
                range_filter &&
                    difficulty_filter &&
                    *difficulty != Unimplemented &&
                    *difficulty != Useless
            })
            .collect();

        let mut rule_used: Option<usize> = None;
        // try the rules and set the difficulty in consequence
        for &&(rule_id, difficulty, rule) in rules.iter() {
            // if the rule can't be applied, then pass to the next one
            if !rule(self).unwrap_or(false) {
                continue;
            }

            debug_only!("règle {} appliquée", rule_id);
            debug_only!("Sudoku actuel:\n{}", self);

            rule_used = Some(rule_id);
            self.difficulty = max(self.difficulty, difficulty);
            break;
        }

        Ok(rule_used)
    }

    pub fn solve(&mut self) -> Vec<Vec<usize>> {
        let mut sudoku = self.clone();
        loop {
            match sudoku.rule_solve(None, None) {
                Ok(None) => {
                    break;
                }
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
        sudoku.board
    }

    // BACKTRACK SOLVING

    pub fn backtrack_solve(&mut self, mut x: usize, mut y: usize) -> bool {
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

        let mut possibilities = self.possibility_board[y][x].iter().cloned().collect::<Vec<_>>();
        possibilities.shuffle(&mut thread_rng());
        for value in possibilities {
            if self.set_value(x, y, value).is_err() {
                self.remove_value(x, y).unwrap();
                continue;
            }

            if self.backtrack_solve(x + 1, y) {
                return true;
            }

            self.remove_value(x, y).unwrap();
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
                self.remove_value(x, y).unwrap();
                continue;
            }

            self._is_unique(x, y, solutions);
            if *solutions > 1 {
                return;
            }

            self.remove_value(x, y).unwrap();
        }
        self.possibility_board[y][x] = possible_values;
    }

    pub fn to_string_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if self.is_canonical {
            lines.push(
                format!(
                    "CANONICAL, difficulty:{}, filled_cells:{}, canonical_board_hash:{}",
                    self.difficulty,
                    self.filled_cells,
                    self.canonical_board_hash
                )
            );
        } else {
            lines.push(
                format!(
                    "RANDOMIZED, difficulty:{}, filled_cells:{}, canonical_board_hash:{}, values_swap:{:?}, rows_swap:{:?}",
                    self.difficulty,
                    self.filled_cells,
                    self.canonical_board_hash,
                    self.values_swap,
                    self.rows_swap
                )
            );
        }

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
                            line.push_str(
                                &format!(
                                    " {}{}{}",
                                    " ".repeat(self.n + 1),
                                    BASE_64[self.board[y][x]],
                                    " ".repeat(self.n + 1)
                                )
                            );
                        } else {
                            line.push_str(&" ".repeat(2 * (self.n + 2)));
                        }
                    }
                    continue;
                }

                this_row_lines.get_mut(0).unwrap().push_str(" ⎧");
                for line in this_row_lines
                    .iter_mut()
                    .skip(1)
                    .take(self.n - 2) {
                    line.push_str(" ⎪");
                }
                this_row_lines
                    .get_mut(self.n - 1)
                    .unwrap()
                    .push_str(" ⎩");

                for i in 0..self.n {
                    for j in 0..self.n {
                        let value = i * self.n + j + 1;
                        let displayed_char = if self.possibility_board[y][x].contains(&value) {
                            BASE_64[value]
                        } else {
                            '·'
                        };
                        this_row_lines.get_mut(i).unwrap().push_str(&format!(" {displayed_char}"));
                    }
                }

                this_row_lines.get_mut(0).unwrap().push_str(" ⎫");
                for line in this_row_lines
                    .iter_mut()
                    .skip(1)
                    .take(self.n - 2) {
                    line.push_str(" ⎪");
                }
                this_row_lines
                    .get_mut(self.n - 1)
                    .unwrap()
                    .push_str(" ⎭");
            }

            for line in this_row_lines.into_iter() {
                lines.push(line);
            }
        }
        lines
    }

    // DATABASE

    #[cfg(feature = "database")]
    pub fn db_from_canonical(origin: DBSimpleSudokuCanonical) -> Self {
        let mut sudoku = Sudoku::new(origin.sudoku_n as usize);
        sudoku.canonical_board_hash = origin.canonical_board_hash;
        for y in 0..sudoku.n2 {
            for x in 0..sudoku.n2 {
                let value = origin.canonical_board
                    [y * (origin.sudoku_n as usize) * (origin.sudoku_n as usize) + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku
    }

    #[cfg(feature = "database")]
    pub fn db_from_game(origin: impl Into<DBNewSimpleSudokuGame>) -> Self {
        let origin: DBNewSimpleSudokuGame = origin.into();
        let mut sudoku = Sudoku::new(origin.game_n as usize);
        sudoku.canonical_board_hash = origin.game_canonical_board_hash;
        for y in 0..sudoku.n2 {
            for x in 0..sudoku.n2 {
                let value = origin.game_board
                    [y * (origin.game_n as usize) * (origin.game_n as usize) + x] as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku.difficulty = SudokuDifficulty::from(origin.game_difficulty);
        sudoku
    }

    #[cfg(feature = "database")]
    pub fn db_to_canonical(
        &self
    ) -> Result<(DBSimpleSudokuCanonical, Vec<DBSimpleSudokuCanonicalSquares>), SudokuError> {
        if !self.is_canonical {
            return Err(
                SudokuError::WrongFunction(
                    format!("db_to_canonical() when this sudoku isn't canonical: {self}")
                )
            );
        }

        let board: Vec<u8> = self.board
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
                    square_canonical_board_hash: self.canonical_board_hash,
                    square_id: square_id as u8,
                    square_hash: hasher.finish(),
                });
            }
        }
        Ok((simple_sudoku_canonical, simple_sudoku_canonical_squares))
    }

    #[cfg(feature = "database")]
    pub fn db_to_randomized(&self) -> Result<DBNewSimpleSudokuGame, SudokuError> {
        if self.is_canonical {
            return Err(
                SudokuError::WrongFunction(
                    format!("db_to_randomized() when this sudoku isn't randomized: {self}")
                )
            );
        }

        let board: Vec<u8> = self.board
            .iter()
            .flat_map(|line| line.iter().map(|cell| *cell as u8))
            .collect();
        Ok(DBNewSimpleSudokuGame {
            game_canonical_board_hash: self.canonical_board_hash,
            game_n: self.n as u8,
            game_board: board,
            game_difficulty: self.difficulty as u8,
            game_filled_cells: self.filled_cells as u16,
        })
    }

    #[cfg(feature = "database")]
    pub fn load_canonical_from_db(database: &mut Database, n: usize) -> Self {
        database.get_random_simple_sudoku_canonical(n as u8).unwrap()
    }

    #[cfg(feature = "database")]
    pub fn load_game_from_db(
        database: &mut Database,
        n: usize,
        difficulty: SudokuDifficulty
    ) -> Self {
        database.get_random_simple_sudoku_game(n as u8, difficulty).unwrap()
    }
}

const BASE_64: [char; 65] = [
    '·',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'G',
    'H',
    'I',
    'J',
    'K',
    'L',
    'M',
    'N',
    'O',
    'P',
    'Q',
    'R',
    'S',
    'T',
    'U',
    'V',
    'W',
    'X',
    'Y',
    'Z',
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
    'g',
    'h',
    'i',
    'j',
    'k',
    'l',
    'm',
    'n',
    'o',
    'p',
    'q',
    'r',
    's',
    't',
    'u',
    'v',
    'w',
    'x',
    'y',
    'z',
    'α',
    'β',
    'δ',
];

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lines().join("\n"))
    }
}

impl PartialEq for Sudoku {
    fn eq(&self, other: &Self) -> bool {
        if self.n != other.n {
            return false;
        }

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if
                    self.board[y][x].ne(&other.board[y][x]) ||
                    self.possibility_board[y][x].ne(&other.possibility_board[y][x])
                {
                    return false;
                }
            }
        }

        true
    }
}

impl Eq for Sudoku {}
