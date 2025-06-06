use super::{
    CellGroupMap, Coords, GroupMap, Sudoku,
    SudokuDifficulty::{self, *},
    SudokuError,
    SudokuGroups::{self, *},
};
use crate::debug_only;
use rand::{rng, seq::SliceRandom};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
    sync::{LazyLock, RwLock},
};

static GROUPS: LazyLock<RwLock<HashMap<usize, GroupMap>>> = LazyLock::new(Default::default);
static CELL_GROUPS: LazyLock<RwLock<HashMap<usize, CellGroupMap>>> =
    LazyLock::new(Default::default);

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
    pub fn clear_possibilities(&mut self, x: usize, y: usize) {
        self.possibility_board[y][x].clear();
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
        groups: Vec<SudokuGroups>,
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

    pub fn get_canonical_filled_board_hash(&self) -> u64 {
        self.canonical_filled_board_hash
    }

    pub fn get_values_swap(&self) -> HashMap<usize, Coords> {
        self.values_swap.clone()
    }

    pub fn get_rows_swap(&self) -> HashMap<usize, Coords> {
        self.rows_swap.clone()
    }

    pub fn set_is_canonical(&mut self, is_canonical: bool) {
        self.is_canonical = is_canonical;
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: usize) -> Result<(), SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(SudokuError::WrongInput(format!(
                "set_value({x}, {y}, {value}); value should be in [1..{}]",
                self.n2
            )));
        }
        if self.board[y][x] != 0 {
            return Err(SudokuError::InvalidState(format!(
                "set_value({x}, {y}, {value}) when board[y][x] = {}",
                self.board[y][x]
            )));
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

        if res.is_ok()
            && self.is_canonical
            && self.canonical_filled_board_hash == 0
            && self.is_filled()
        {
            self.canonical_filled_board_hash = {
                let mut hasher = DefaultHasher::new();
                for y in 0..self.n2 {
                    for x in 0..self.n2 {
                        self.board[y][x].hash(&mut hasher);
                    }
                }
                hasher.finish()
            };
        }

        res
    }

    pub fn insert_possibility(
        &mut self,
        x: usize,
        y: usize,
        value: usize,
    ) -> Result<bool, SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(SudokuError::WrongInput(format!(
                "set_value({x}, {y}, {value}); value should be in [1..{}]",
                self.n2
            )));
        }
        if self.board[y][x] != 0 {
            return Err(SudokuError::InvalidState(format!(
                "remove_value({x}, {y}) when board[y][x] = {}",
                self.board[y][x]
            )));
        }

        Ok(self.possibility_board[y][x].insert(value))
    }

    pub fn remove_possibility(
        &mut self,
        x: usize,
        y: usize,
        value: usize,
    ) -> Result<bool, SudokuError> {
        if value == 0 || value > self.n2 {
            return Err(SudokuError::WrongInput(format!(
                "remove_possibility({x}, {y}, {value}); value should be in [1..{}]",
                self.n2
            )));
        }
        if self.board[y][x] != 0 {
            return Err(SudokuError::InvalidState(format!(
                "remove_possibility({x}, {y}, {value}) when board[y][x] = {}",
                self.board[y][x]
            )));
        }

        Ok(self.possibility_board[y][x].remove(&value))
    }

    pub fn remove_value(&mut self, x: usize, y: usize) -> Result<usize, SudokuError> {
        if self.board[y][x] == 0 {
            return Err(SudokuError::InvalidState(format!(
                "remove_value({x}, {y}) when board[y][x] = {}",
                self.board[y][x]
            )));
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

            if self
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
        let canonical_filled_board_hash = 0;
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
                canonical_filled_board_hash,
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
            canonical_filled_board_hash,
            values_swap,
            rows_swap,
        }
    }

    pub fn generate_canonical(n: usize) -> Self {
        Self::new(n).into_generate_canonical_from()
    }

    pub fn generate_canonical_from(&self) -> Self {
        self.clone().into_generate_canonical_from()
    }

    pub fn into_generate_canonical_from(mut self) -> Self {
        self._fill_first_line(true, 0);
        self._fill_first_line(false, 0);
        self.is_canonical = true;
        self
    }

    fn _fill_first_line(&mut self, fill_row: bool, mut i: usize) -> bool {
        let (mut x, mut y) = if fill_row { (i, 0) } else { (0, i) };

        while i < self.n2 && self.board[y][x] > 0 {
            i += 1;
            (x, y) = if fill_row { (i, 0) } else { (0, i) };
        }
        if i >= self.n2 {
            return true;
        }

        let mut possibilities = self.possibility_board[y][x]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.sort();
        for value in possibilities {
            match self.set_value(x, y, value) {
                Ok(()) => (),
                Err(_) => {
                    let _ = self.remove_value(x, y);
                    continue;
                }
            }

            if self._fill_first_line(fill_row, i + 1) {
                return true;
            }

            if let Err(err) = self.remove_value(x, y) {
                log::warn!("ERRROR AFTER self.remove_value({x}, {y}): {err}\nFOR SUDOKU:{self}");
            }
        }

        false
    }

    pub fn generate_full(n: usize) -> Self {
        Self::new(n).into_generate_full_from().unwrap()
    }

    pub fn generate_full_from(&self) -> Result<Self, SudokuError> {
        self.clone().into_generate_full_from()
    }

    pub fn into_generate_full_from(self) -> Result<Self, SudokuError> {
        let mut canonical = self.generate_canonical_from();

        // fill the rest of the sudoku
        canonical.backtrack_solve(0, 0);

        Ok(canonical)
    }

    pub fn randomize(
        &mut self,
        rows_swap: Option<HashMap<usize, Coords>>,
        values_swap: Option<HashMap<usize, Coords>>,
        shuffle_floors: bool,
    ) -> Result<(), SudokuError> {
        if !self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "randomize() when this sudoku is already randomized: {self}"
            )));
        }
        let mut rng = rng();

        self.rows_swap = rows_swap.unwrap_or({
            let mut floors = (0..self.n2)
                .collect::<Vec<_>>()
                .chunks(self.n)
                .map(|floor| floor.to_vec())
                .collect::<Vec<_>>();

            // shuffle each floor
            if shuffle_floors {
                floors.shuffle(&mut rng);
            }

            // shuffle each row inside a floor
            for floor in floors.iter_mut() {
                floor.shuffle(&mut rng);
            }

            let shuffled_rows = floors.into_iter().flatten().enumerate().collect::<Vec<_>>();

            let mut rows_swap = HashMap::new();
            for (y, to_y) in shuffled_rows {
                rows_swap
                    .entry(y)
                    .and_modify(|(a, _)| *a = to_y)
                    .or_insert((to_y, 0));
                rows_swap
                    .entry(to_y)
                    .and_modify(|(_, b)| *b = y)
                    .or_insert((0, y));
            }
            rows_swap
        });

        self.values_swap = values_swap.unwrap_or({
            let mut to_values = (1..=self.n2).collect::<Vec<_>>();
            to_values.shuffle(&mut rng);

            let mut values_swap = HashMap::new();
            for (value, to_value) in to_values
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, to_value)| (i + 1, to_value))
            {
                values_swap
                    .entry(value)
                    .and_modify(|(a, _)| *a = to_value)
                    .or_insert((to_value, 0));
                values_swap
                    .entry(to_value)
                    .and_modify(|(_, b)| *b = value)
                    .or_insert((0, value));
            }
            values_swap
        });

        // swap rows randomly following self.rows_swap rules
        let mut new_board = vec![Vec::new(); self.n2];
        let mut new_possibility_board = vec![Vec::new(); self.n2];
        for y in 0..self.n2 {
            let (to_y, _) = self.rows_swap[&y];
            new_board[to_y] = self.board[y].clone();
            new_possibility_board[to_y] = self.possibility_board[y].clone();
        }
        self.board = new_board;
        self.possibility_board = new_possibility_board;
        self.is_canonical = false;

        // swap value randomly following self.values_swap rules
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let value = self.board[y][x];
                if value != 0 {
                    let (to_value, _) = self.values_swap[&value];
                    self.board[y][x] = to_value;
                } else {
                    let possibilities = self.possibility_board[y][x]
                        .iter()
                        .map(|value| self.values_swap[value].0)
                        .collect::<HashSet<_>>();
                    self.possibility_board[y][x] = possibilities;
                }
            }
        }
        Ok(())
    }

    pub fn canonize(&mut self) -> Result<(), SudokuError> {
        if self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "canonize() when this sudoku is already canonized: {self}"
            )));
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
            for y in 0..self.n2 {
                for x in 0..self.n2 {
                    self.board[y][x].hash(&mut hasher);
                }
            }
            hasher.finish()
        };

        if board_hash != self.canonical_filled_board_hash {
            Err(SudokuError::CanonizationMismatch(
                Box::new(self.clone()),
                board_hash,
            ))
        } else {
            self.is_canonical = false;
            self.rows_swap.clear();
            self.values_swap.clear();
            Ok(())
        }
    }

    pub fn parse_file(file_name: &str) -> Result<Self, SudokuError> {
        let mut file_path = current_dir().unwrap();
        file_path.push("res/sudoku_samples/");
        file_path.push(file_name);
        let file_content = std::fs::read_to_string(&file_path).map_err(|error| {
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
        let n = first_line.parse::<usize>().map_err(|error| {
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
                line.iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        lines.join("\n")
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////
    // RULE SOLVING

    pub fn rule_solve(
        &mut self,
        specific_rules: Option<Range<usize>>,
        max_difficulty: Option<SudokuDifficulty>,
    ) -> Result<Option<usize>, SudokuError> {
        let mut used_rule: Option<usize> = None;
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

        // try the rules and set the difficulty in consequence
        for &&(rule_id, difficulty, rule) in rules.iter() {
            // if the rule can't be applied, then pass to the next one
            if !rule(self).unwrap_or(false) {
                continue;
            }
            used_rule = Some(rule_id);
            debug_only!("règle {} appliquée", rule_id);
            debug_only!("Sudoku actuel:\n{}", self);

            self.difficulty = max(self.difficulty, difficulty);
            break;
        }
        Ok(used_rule)
    }

    pub fn rule_solve_until(
        &mut self,
        rule_solve_result: Option<usize>,
        specific_rules: Option<Range<usize>>,
        max_difficulty: Option<SudokuDifficulty>,
    ) -> bool {
        let mut did_anything = false;
        while let Ok(result) = self.rule_solve(specific_rules.clone(), max_difficulty) {
            if result.is_none() || result == rule_solve_result {
                break;
            }
            did_anything = true;
        }
        did_anything
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

        let mut possibilities = self.possibility_board[y][x]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut rng());
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

    pub fn is_empty(&self) -> bool {
        self.filled_cells == 0
    }

    pub fn is_filled(&self) -> bool {
        self.filled_cells == self.n2 * self.n2
    }

    pub fn is_unique(&mut self) -> bool {
        self.count_solutions(Some(2)) == 1
    }

    pub fn count_solutions(&self, max_solutions: Option<usize>) -> usize {
        self.clone()._count_solutions(
            (0..self.n2 * self.n2)
                .filter_map(|cell_i| {
                    let y = cell_i / self.n2;
                    let x = cell_i % self.n2;
                    let value = self.board[y][x];
                    if value == 0 {
                        Some((x, y))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            max_solutions,
        )
    }

    fn _count_solutions(
        &mut self,
        mut empty_cells: Vec<Coords>,
        max_solutions: Option<usize>,
    ) -> usize {
        empty_cells.sort_by_key(|&(x1, y1)| self.possibility_board[y1][x1].len());

        let mut i = 0;
        while i < empty_cells.len() {
            let (x, y) = empty_cells[i];
            if !self.possibility_board[y][x].is_empty() {
                break;
            }
            if self.board[y][x] == 0 {
                return 0;
            }
            i += 1;
        }
        empty_cells.drain(0..i);

        if empty_cells.is_empty() {
            return 1;
        }

        let (x, y) = empty_cells[0];
        let mut possibilities = self.possibility_board[y][x]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut rng());
        let mut sub_solutions = 0;
        for value in possibilities {
            match self.set_value(x, y, value) {
                Ok(()) => (),
                Err(_) => {
                    let _ = self.remove_value(x, y);
                    continue;
                }
            }

            sub_solutions += self._count_solutions(empty_cells.clone(), max_solutions);
            if let Some(max_solutions) = max_solutions {
                if sub_solutions >= max_solutions {
                    return sub_solutions;
                }
            }

            if let Err(err) = self.remove_value(x, y) {
                eprintln!("ERRROR AFTER self.remove_value({x}, {y}): {err}\nFOR SUDOKU:{self}");
            }
        }

        sub_solutions
    }

    pub fn to_string_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if self.is_canonical {
            lines.push(format!(
                "CANONICAL, difficulty:{}, filled_cells:{}, canonical_filled_board_hash:{}",
                self.difficulty, self.filled_cells, self.canonical_filled_board_hash
            ));
        } else {
            lines.push(
                format!(
                    "RANDOMIZED, difficulty:{}, filled_cells:{}, canonical_filled_board_hash:{}, values_swap:{:?}, rows_swap:{:?}",
                    self.difficulty,
                    self.filled_cells,
                    self.canonical_filled_board_hash,
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
        lines
    }
}

#[cfg(feature = "database")]
use crate::database::{
    DBCanonicalSudoku, DBCanonicalSudokuSquare, DBNewCanonicalSudokuGame, Database,
};

#[cfg(feature = "database")]
impl Sudoku {
    pub fn db_from_filled(origin: DBCanonicalSudoku) -> Self {
        let mut sudoku = Sudoku::new(origin.sudoku_n as usize);
        sudoku.is_canonical = true;
        sudoku.canonical_filled_board_hash =
            (origin.filled_board_hash as u64).wrapping_add(u64::MAX / 2 + 1);
        for y in 0..sudoku.n2 {
            for x in 0..sudoku.n2 {
                let value = origin.canonical_board
                    [y * (origin.sudoku_n as usize) * (origin.sudoku_n as usize) + x]
                    as usize;
                if value != 0 {
                    sudoku.set_value(x, y, value).unwrap();
                }
            }
        }
        sudoku
    }

    pub fn db_from_game(
        game_info: impl Into<DBNewCanonicalSudokuGame>,
        filled_info: DBCanonicalSudoku,
    ) -> Self {
        let game_info: DBNewCanonicalSudokuGame = game_info.into();

        let mut sudoku = Sudoku::new(filled_info.sudoku_n as usize);
        sudoku.is_canonical = true;
        sudoku.canonical_filled_board_hash =
            (filled_info.filled_board_hash as u64).wrapping_add(u64::MAX / 2 + 1);
        for y in 0..sudoku.n2 {
            for x in 0..sudoku.n2 {
                let i = y * sudoku.n * sudoku.n + x;
                if game_info.sudoku_game_filled_cells[i] == 1 {
                    sudoku
                        .set_value(x, y, filled_info.canonical_board[i] as usize)
                        .unwrap();
                }
            }
        }
        sudoku.difficulty = SudokuDifficulty::from(game_info.sudoku_game_difficulty);
        sudoku
    }

    pub fn game_to_db(&self) -> Result<DBNewCanonicalSudokuGame, SudokuError> {
        if self.is_filled() {
            return Err(
                SudokuError::WrongFunction(
                    format!(
                        "game_to_db() when the sudoku is filled. Try calling filled_to_db() instead.\n{self}"
                    )
                )
            );
        }
        if self.canonical_filled_board_hash == 0 {
            return Err(SudokuError::InvalidState(format!(
                "game_to_db() when the sudoku has no canonical filled board hash: \n{self}"
            )));
        }

        let filled_cells: Vec<u8> = (0..self.n2 * self.n2)
            .map(|i| (self.board[i / self.n2][i % self.n2] > 0) as u8)
            .collect();
        Ok(DBNewCanonicalSudokuGame {
            sudoku_game_filled_board_hash: self
                .canonical_filled_board_hash
                .wrapping_sub(u64::MAX / 2 + 1) as i64,
            sudoku_game_difficulty: self.difficulty as i16,
            sudoku_game_filled_cells: filled_cells,
            sudoku_game_filled_cells_count: self.filled_cells as i16,
        })
    }

    pub fn filled_to_db(
        &self,
    ) -> Result<(DBCanonicalSudoku, Vec<DBCanonicalSudokuSquare>), SudokuError> {
        if !self.is_filled() {
            return Err(
                SudokuError::WrongFunction(
                    format!(
                        "filled_to_db() when the sudoku isn't filled. Try calling game_to_db() instead.\n{self}"
                    )
                )
            );
        }
        if self.canonical_filled_board_hash == 0 {
            return Err(SudokuError::InvalidState(format!(
                "filled_to_db() when the sudoku has no canonical filled board hash: \n{self}"
            )));
        }

        let board: Vec<u8> = self
            .board
            .iter()
            .flat_map(|line| line.iter().map(|cell| *cell as u8))
            .collect();

        let simple_sudoku_canonical = DBCanonicalSudoku {
            filled_board_hash: self
                .canonical_filled_board_hash
                .wrapping_sub(u64::MAX / 2 + 1) as i64,
            sudoku_n: self.n as i16,
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
                simple_sudoku_canonical_squares.push(DBCanonicalSudokuSquare {
                    square_filled_board_hash: self
                        .canonical_filled_board_hash
                        .wrapping_sub(u64::MAX / 2 + 1)
                        as i64,
                    square_id: square_id as i16,
                    square_hash: hasher.finish().wrapping_sub(u64::MAX / 2 + 1) as i64,
                });
            }
        }
        Ok((simple_sudoku_canonical, simple_sudoku_canonical_squares))
    }

    pub fn load_filled_from_db(database: &mut Database, n: usize) -> Self {
        database.get_random_canonical_sudokus(n as u8).unwrap()
    }

    pub fn load_game_from_db(
        database: &mut Database,
        n: usize,
        difficulty: SudokuDifficulty,
    ) -> Self {
        database
            .get_random_canonical_sudoku_game(n as i16, difficulty as i16)
            .unwrap()
    }

    pub fn load_minimal_game_from_db(
        database: &mut Database,
        n: usize,
        difficulty: SudokuDifficulty,
    ) -> Self {
        database
            .get_random_minimal_canonical_sudoku_game(n as i16, difficulty as i16)
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

impl Eq for Sudoku {}
