use super::{CarpetPattern, CarpetSudoku};
use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty, SudokuError, SudokuGroups};
use log::warn;
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
    sync::{mpsc, Arc, Mutex},
    thread::{available_parallelism, JoinHandle},
};

type RawLink = ((usize, usize), (usize, usize));
impl CarpetSudoku {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////   GETTERS / SETTERS   /////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_n(&self) -> usize {
        self.n
    }

    pub fn get_n2(&self) -> usize {
        self.n2
    }

    pub fn get_pattern(&self) -> CarpetPattern {
        self.pattern
    }

    pub fn get_sudokus(&self) -> Vec<Sudoku> {
        self.sudokus.clone()
    }

    pub fn get_n_sudokus(&self) -> usize {
        self.sudokus.len()
    }

    pub fn get_links(&self) -> HashMap<usize, HashSet<(usize, usize, usize)>> {
        self.links.clone()
    }

    pub fn get_difficulty(&self) -> SudokuDifficulty {
        self.difficulty
    }

    pub fn get_cell_value(&self, sudoku_id: usize, x: usize, y: usize) -> usize {
        self.sudokus[sudoku_id].get_cell_value(x, y)
    }

    pub fn get_cell_possibilities(&self, sudoku_id: usize, x: usize, y: usize) -> HashSet<usize> {
        self.sudokus[sudoku_id].get_cell_possibilities(x, y).clone()
    }

    pub fn get_cell_possibilities_mut(
        &mut self,
        sudoku_id: usize,
        x: usize,
        y: usize,
    ) -> &mut HashSet<usize> {
        self.sudokus[sudoku_id].get_cell_possibilities_mut(x, y)
    }

    pub fn get_cell_group(
        &self,
        sudoku_id: usize,
        x: usize,
        y: usize,
        groups: SudokuGroups,
    ) -> HashSet<Coords> {
        self.sudokus[sudoku_id].get_cell_group(x, y, groups)
    }

    pub fn get_golbal_cell_group(
        &self,
        sudoku_id: usize,
        x: usize,
        y: usize,
        group: SudokuGroups,
    ) -> HashSet<(usize, usize, usize)> {
        let dx = x % self.n;
        let dy = y % self.n;
        let x0 = x - dx;
        let y0 = y - dy;
        let square_id = y0 + x0 / self.n;
        let mut cell_group: HashSet<(usize, usize, usize)> = self.sudokus[sudoku_id].get_cell_group(x, y, group).into_iter().map(|(x,y)| (sudoku_id, x, y)).collect();

        for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
            if square_id != square1 {
                continue;
            }

            let y2 = (square2 / self.n) * self.n;
            let x2 = (square2 % self.n) * self.n;
            cell_group.extend(
                self.sudokus[sudoku2]
                    .get_cell_group(x2 + dx, y2 + dy, group)
                    .into_iter()
                    .map(|(x,y)| (sudoku2, x, y))
            );
        }
        cell_group
    }

    pub fn get_filled_cells(&self) -> usize {
        self.sudokus
            .iter()
            .map(|sudoku| sudoku.get_filled_cells())
            .sum()
    }

    pub fn get_possibility_board(&self) -> Vec<Vec<Vec<HashSet<usize>>>> {
        self.sudokus
            .iter()
            .map(|sudoku| sudoku.get_possibility_board().clone())
            .collect()
    }

    pub fn get_sudoku_possibility_board(&self, sudoku_i: usize) -> Vec<Vec<HashSet<usize>>> {
        self.sudokus[sudoku_i].get_possibility_board().clone()
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   CREATION   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn new(n: usize, pattern: CarpetPattern) -> Self {
        let (sudokus, raw_links) = match pattern {
            CarpetPattern::Double => Self::new_diagonal(n, 2),
            CarpetPattern::Diagonal(n_sudokus) => Self::new_diagonal(n, n_sudokus),
            CarpetPattern::Samurai => Self::new_samurai(n),
        };

        let mut links: HashMap<usize, HashSet<(usize, usize, usize)>> = HashMap::new();
        for sudoku_id in 0..sudokus.len() {
            links.insert(sudoku_id, HashSet::new());
        }

        for ((sudoku1, square1), (sudoku2, square2)) in raw_links {
            let sudoku1_links = links.get_mut(&sudoku1).unwrap();
            sudoku1_links.insert((square1, sudoku2, square2));
            let sudoku2_links = links.get_mut(&sudoku2).unwrap();
            sudoku2_links.insert((square2, sudoku1, square1));
        }

        Self {
            n,
            n2: n * n,
            difficulty: SudokuDifficulty::Unknown,
            sudokus,
            links,
            pattern,
        }
    }

    fn new_diagonal(n: usize, n_sudokus: usize) -> (Vec<Sudoku>, Vec<RawLink>) {
        let sudokus = vec![Sudoku::new(n); n_sudokus];
        let links = (1..n_sudokus)
            .map(|i| ((i - 1, n - 1), (i, n * (n - 1))))
            .collect();
        (sudokus, links)
    }

    fn new_samurai(n: usize) -> (Vec<Sudoku>, Vec<RawLink>) {
        let sudokus = vec![
            Sudoku::new(n), // center sudoku
            Sudoku::new(n), // top left sudoku
            Sudoku::new(n), // top right sudoku
            Sudoku::new(n), // bottom left sudoku
            Sudoku::new(n), // bottom right sudoku
        ];
        let links = vec![
            ((0, 0), (1, n * n - 1)),
            ((0, n - 1), (2, 2 * n)),
            ((0, 2 * n), (3, n - 1)),
            ((0, n * n - 1), (4, 0)),
        ];
        (sudokus, links)
    }

    pub fn generate_full(n: usize, pattern: CarpetPattern) -> Self {
        let mut carpet = Self::new(n, pattern);
        carpet.backtrack_solve(0, 0, 0);
        carpet
    }

    pub fn generate_new(n: usize, pattern: CarpetPattern, difficulty: SudokuDifficulty) -> Self {
        Self::generate_full(n, pattern).into_generate_from(difficulty)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   MODIFICATION   ///////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn update_link(&mut self) -> Result<(), SudokuError> {
        for (&sudoku1, links) in self.links.to_owned().iter() {
            for &(square1, sudoku2, square2) in links.iter() {
                let y1 = (square1 / self.n) * self.n;
                let x1 = (square1 % self.n) * self.n;

                let y2 = (square2 / self.n) * self.n;
                let x2 = (square2 % self.n) * self.n;

                for dy in 0..self.n {
                    for dx in 0..self.n {
                        let value1 = self.sudokus[sudoku1].get_cell_value(x1 + dx, y1 + dy);
                        let value2 = self.sudokus[sudoku2].get_cell_value(x2 + dx, y2 + dy);
                        if value1 != value2 {
                            if value1 != 0 {
                                self.sudokus[sudoku2].set_value(x2 + dx, y2 + dy, value1)?;
                                continue;
                            } else if value2 != 0 {
                                self.sudokus[sudoku1].set_value(x1 + dx, y1 + dy, value2)?;
                                continue;
                            } else {
                                panic!("ALORS LÀ J'AI PAS COMPRIS");
                            }
                        }

                        if value1 != 0 && value2 != 0 {
                            self.sudokus[sudoku1].clear_possibilities(x1 + dx, y1 + dy);
                            self.sudokus[sudoku2].clear_possibilities(x2 + dx, y2 + dy);
                            continue;
                        }

                        let possibilities1 = self.sudokus[sudoku1]
                            .get_cell_possibilities(x1 + dx, y1 + dy)
                            .clone();
                        let possibilities2 = self.sudokus[sudoku2]
                            .get_cell_possibilities(x2 + dx, y2 + dy)
                            .clone();

                        for p in possibilities1.iter() {
                            if possibilities2.contains(p) {
                                continue;
                            }
                            self.sudokus[sudoku1].remove_possibility(x1 + dx, y1 + dy, *p)?;
                        }
                        for p in possibilities2.iter() {
                            if possibilities1.contains(p) {
                                continue;
                            }
                            self.sudokus[sudoku2].remove_possibility(x2 + dx, y2 + dy, *p)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_value(
        &mut self,
        sudoku_id: usize,
        x: usize,
        y: usize,
        value: usize,
    ) -> Result<(), SudokuError> {
        let dx = x % self.n;
        let dy = y % self.n;
        let x0 = x - dx;
        let y0 = y - dy;
        let square_id = y0 + x0 / self.n;
        let mut is_in_link = false;
        self.sudokus[sudoku_id].set_value(x, y, value)?;

        for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
            if square_id != square1 {
                continue;
            }
            is_in_link = true;

            let y2 = (square2 / self.n) * self.n;
            let x2 = (square2 % self.n) * self.n;
            self.sudokus[sudoku2].set_value(x2 + dx, y2 + dy, value)?;
        }

        if is_in_link {
            return Ok(());
        }

        for (x, y) in self.sudokus[sudoku_id].get_cell_group(x, y, SudokuGroups::All) {
            let dx = x % self.n;
            let dy = y % self.n;
            let x0 = x - dx;
            let y0 = y - dy;
            let square_id = y0 + x0 / self.n;

            for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
                if square_id != square1 {
                    continue;
                }

                let y2 = (square2 / self.n) * self.n;
                let x2 = (square2 % self.n) * self.n;
                self.sudokus[sudoku2]
                    .get_cell_possibilities_mut(x2 + dx, y2 + dy)
                    .remove(&value);
            }
        }

        Ok(())
    }

    pub fn remove_value(
        &mut self,
        sudoku_id: usize,
        x: usize,
        y: usize,
    ) -> Result<usize, SudokuError> {
        let dx = x % self.n;
        let dy = y % self.n;
        let x0 = x - dx;
        let y0 = y - dy;
        let square_id = y0 + x0 / self.n;
        let value = self.sudokus[sudoku_id].remove_value(x, y)?;
        let mut is_in_link = false;

        for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
            if square_id != square1 {
                continue;
            }
            is_in_link = true;

            let y2 = (square2 / self.n) * self.n;
            let x2 = (square2 % self.n) * self.n;
            self.sudokus[sudoku2].remove_value(x2 + dx, y2 + dy)?;
        }

        if is_in_link {
            return Ok(value);
        }

        for (x, y) in self.sudokus[sudoku_id].get_cell_group(x, y, SudokuGroups::All) {
            let dx = x % self.n;
            let dy = y % self.n;
            let x0 = x - dx;
            let y0 = y - dy;
            let square_id = y0 + x0 / self.n;

            for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
                if square_id != square1 {
                    continue;
                }

                let y2 = (square2 / self.n) * self.n;
                let x2 = (square2 % self.n) * self.n;

                if self.sudokus[sudoku2]
                    .get_cell_group(x2 + dx, y2 + dy, SudokuGroups::All)
                    .into_iter()
                    .any(|(x3, y3)| self.sudokus[sudoku2].get_cell_value(x3, y3) == value)
                {
                    self.sudokus[sudoku_id]
                        .get_cell_possibilities_mut(x, y)
                        .remove(&value);
                } else {
                    self.sudokus[sudoku2]
                        .get_cell_possibilities_mut(x2 + dx, y2 + dy)
                        .insert(value);
                }
            }
        }

        Ok(value)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   SOLVING   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn rule_solve(
        &mut self,
        max_difficulty: Option<SudokuDifficulty>,
    ) -> Result<(bool, bool), SudokuError> {
        let mut modified_possibility = false;
        let mut modified_value = false;
        for sudoku in self.sudokus.iter_mut() {
            match sudoku.rule_solve(None, max_difficulty) {
                Ok(Some(0 | 1)) => {
                    modified_value = true;
                    modified_possibility = true;
                }
                Ok(Some(_)) => {
                    modified_possibility = true;
                }
                Ok(None) => (),
                Err(err) => {
                    self.update_link()?;
                    return Err(err);
                }
            }
            self.difficulty = self.difficulty.max(sudoku.get_difficulty());
        }
        self.update_link()
            .map(|_| (modified_possibility, modified_value))
    }

    pub fn backtrack_solve(&mut self, mut sudoku_id: usize, mut x: usize, mut y: usize) -> bool {
        loop {
            if sudoku_id == self.sudokus.len() - 1 && y == self.n2 - 1 && x == self.n2 {
                return true;
            }

            if x == self.n2 {
                if y == self.n2 - 1 {
                    sudoku_id += 1;
                    y = 0;
                    x = 0;
                } else {
                    y += 1;
                    x = 0;
                }
            }

            if self.sudokus[sudoku_id].get_cell_value(x, y) == 0 {
                break;
            }
            x += 1;
        }

        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut thread_rng());
        for value in possibilities {
            match self.set_value(sudoku_id, x, y, value) {
                Ok(()) => (),
                Err(SudokuError::NoPossibilityCell((errx, erry))) => {
                    if let Err(err) = self.remove_value(sudoku_id, x, y) {
                        warn!(
                            "ERRROR AFTER set_value({sudoku_id}, {x}, {y}, {value}) MADE {errx},{erry} EMPTY: {err}\nFOR CARPET:{self}"
                        );
                    }
                    continue;
                }
                Err(err) => warn!("{err}"),
            }

            if self.backtrack_solve(sudoku_id, x + 1, y) {
                return true;
            }

            if let Err(err) = self.remove_value(sudoku_id, x, y) {
                warn!(
                    "ERRROR AFTER self.remove_value({sudoku_id}, {x}, {y}): {err}\nFOR CARPET:{self}"
                );
            }
        }

        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////   GENERATION   ////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Self {
        self.clone().into_generate_from(aimed_difficulty)
    }

    pub fn into_generate_from(self, aimed_difficulty: SudokuDifficulty) -> Self {
        let n2 = self.n2;
        let n_sudokus = self.sudokus.len();
        let (tx, rx) = mpsc::channel::<Self>();
        type SudokuFilledCells = (CarpetSudoku, Vec<bool>);

        let thread_count: usize = available_parallelism().unwrap().get();
        let default: Arc<Mutex<SudokuFilledCells>> = {
            let filled_cells: Vec<bool> = (0..n_sudokus * n2 * n2)
                .map(|i| {
                    let sudoku_id = i / (n2 * n2);
                    let cell_i = i - sudoku_id * n2 * n2;
                    let y = cell_i / n2;
                    let x = cell_i % n2;
                    self.sudokus[sudoku_id].get_cell_value(x, y) != 0
                })
                .collect();
            Arc::new(Mutex::new((self.clone(), filled_cells)))
        };
        let to_explore: Arc<Mutex<Vec<SudokuFilledCells>>> = Arc::new(Default::default());
        let explored_filled_cells: Arc<Mutex<HashSet<Vec<bool>>>> = Arc::new(Default::default());
        let total: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let skipped: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

        let mut threads_infos: Vec<(JoinHandle<()>, mpsc::Sender<()>)> = Vec::new();
        for thread_id in 0..thread_count {
            let thread_default = Arc::clone(&default);
            let thread_to_explore = Arc::clone(&to_explore);
            let thread_explored_filled_cells = Arc::clone(&explored_filled_cells);
            let thread_total = Arc::clone(&total);
            let thread_skipped = Arc::clone(&skipped);
            let thread_tx = tx.clone();
            let (main_tx, thread_rx) = mpsc::channel::<()>();

            let join_handle = std::thread::Builder::new()
                .name(format!("thread {thread_id}"))
                .spawn(move || {
                    let mut rng = rand::thread_rng();
                    while thread_rx.try_recv().is_err() {
                        let (carpet, filled_cells) = thread_to_explore
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
                            let sudoku_id = i / (n2 * n2);
                            let cell_i = i - sudoku_id * n2 * n2;
                            let y = cell_i / n2;
                            let x = cell_i % n2;
                            let mut testing_carpet = carpet.clone();
                            testing_carpet.difficulty = SudokuDifficulty::Unknown;

                            let removed_value =
                                testing_carpet.remove_value(sudoku_id, x, y).unwrap();

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
                                match testing_carpet.rule_solve(Some(aimed_difficulty)) {
                                    Ok((true, true)) => {
                                        if testing_carpet.sudokus[sudoku_id].get_cell_value(x, y)
                                            == removed_value
                                        {
                                            can_solve = true;
                                            break;
                                        }

                                        let testing_filled_cells: Vec<bool> =
                                            (0..n_sudokus * n2 * n2)
                                                .map(|i| {
                                                    let sudoku_id = i / (n2 * n2);
                                                    let cell_i = i - sudoku_id * n2 * n2;
                                                    let y = cell_i / n2;
                                                    let x = cell_i % n2;
                                                    testing_carpet.sudokus[sudoku_id]
                                                        .get_cell_value(x, y)
                                                        != 0
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
                                    Ok((true, false)) => (),
                                    _ => {
                                        let testing_filled_cells: Vec<bool> =
                                            (0..n_sudokus * n2 * n2)
                                                .map(|i| {
                                                    let sudoku_id = i / (n2 * n2);
                                                    let cell_i = i - sudoku_id * n2 * n2;
                                                    let y = cell_i / n2;
                                                    let x = cell_i % n2;
                                                    testing_carpet.sudokus[sudoku_id]
                                                        .get_cell_value(x, y)
                                                        != 0
                                                })
                                                .collect();
                                        thread_explored_filled_cells
                                            .lock()
                                            .unwrap()
                                            .insert(testing_filled_cells);
                                        break;
                                    }
                                }
                            }
                            if !can_solve {
                                continue;
                            }

                            if testing_carpet.get_filled_cells() < n_sudokus * (n2 * 2 - 1) {
                                thread_explored_filled_cells
                                    .lock()
                                    .unwrap()
                                    .insert(filled_cells.clone());
                            } else {
                                // EXPLORATION EN PROFONDEUR
                                let mut passed_carpet = carpet.clone();
                                passed_carpet.remove_value(sudoku_id, x, y).unwrap();
                                passed_carpet.difficulty = testing_carpet.difficulty;

                                let passed_filled_cells: Vec<bool> = (0..n_sudokus * n2 * n2)
                                    .map(|i| {
                                        let sudoku_id = i / (n2 * n2);
                                        let cell_i = i - sudoku_id * n2 * n2;
                                        let y = cell_i / n2;
                                        let x = cell_i % n2;
                                        passed_carpet.sudokus[sudoku_id].get_cell_value(x, y) != 0
                                    })
                                    .collect();

                                thread_to_explore
                                    .lock()
                                    .unwrap()
                                    .push((passed_carpet, passed_filled_cells));

                                working_sub_sudokus += 1;
                            }
                        }

                        if working_sub_sudokus == 0 && carpet.difficulty == aimed_difficulty {
                            let mut returned = carpet.clone();
                            returned.difficulty = SudokuDifficulty::Unknown;
                            thread_tx.send(returned).unwrap();
                        }
                    }
                })
                .unwrap();
            threads_infos.push((join_handle, main_tx));
        }

        loop {
            let mut carpet = rx.recv().unwrap();

            // verify that the carpet is unique
            if !carpet.clone().is_unique() {
                continue;
            }

            // verify that each sudoku isn't solvable alone
            if carpet.get_sudokus().clone().into_iter().any(|mut sudoku| {
                while let Ok(Some(_)) = sudoku.rule_solve(None, Some(aimed_difficulty)) {}
                sudoku.is_filled()
            }) {
                continue;
            }

            // verify the generated carpet
            let mut verify_carpet = carpet.clone();
            while let Ok((true, _)) = verify_carpet.rule_solve(None) {}

            if !verify_carpet.is_filled() {
                continue;
            }

            for (handle, tx) in threads_infos {
                tx.send(()).unwrap();
                handle.join().unwrap();
            }
            carpet.update_link();
            return carpet;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   UTILITY   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn is_filled(&self) -> bool {
        self.sudokus.iter().all(|sudoku| sudoku.is_filled())
    }

    pub fn is_unique(&mut self) -> bool {
        self.count_solutions(0, 0, 0, Some(1)) == 1
    }

    pub fn count_solutions(
        &mut self,
        mut sudoku_id: usize,
        mut x: usize,
        mut y: usize,
        max_solutions: Option<usize>,
    ) -> usize {
        loop {
            if sudoku_id == self.sudokus.len() - 1 && y == self.n2 - 1 && x == self.n2 {
                return 1;
            }

            if x == self.n2 {
                if y == self.n2 - 1 {
                    sudoku_id += 1;
                    y = 0;
                    x = 0;
                } else {
                    y += 1;
                    x = 0;
                }
            }

            if self.sudokus[sudoku_id].get_cell_value(x, y) == 0 {
                break;
            }
            x += 1;
        }

        let mut current_solutions = 0;
        for value in self.sudokus[sudoku_id].get_cell_possibilities(x, y).clone() {
            match self.set_value(sudoku_id, x, y, value) {
                Ok(()) => (),
                Err(SudokuError::NoPossibilityCell((errx, erry))) => {
                    if let Err(err) = self.remove_value(sudoku_id, x, y) {
                        warn!(
                            "ERRROR AFTER set_value({sudoku_id}, {x}, {y}, {value}) MADE {errx},{erry} EMPTY: {err}\nFOR CARPET:{self}"
                        );
                    }
                    continue;
                }
                Err(err) => warn!("{err}"),
            }

            current_solutions += self.count_solutions(sudoku_id, x + 1, y, max_solutions);
            if let Some(max_solutions) = max_solutions {
                if current_solutions >= max_solutions {
                    return current_solutions;
                }
            }

            if let Err(err) = self.remove_value(sudoku_id, x, y) {
                warn!(
                    "ERRROR AFTER self.remove_value({sudoku_id}, {x}, {y}): {err}\nFOR CARPET:{self}"
                );
            }
        }

        current_solutions
    }
}

impl std::fmt::Display for CarpetSudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, sudoku) in self.sudokus.iter().enumerate() {
            writeln!(f, "Sudoku {}:\t{}", i, sudoku)?;
        }
        Ok(())
    }
}

impl PartialEq for CarpetSudoku {
    fn eq(&self, other: &Self) -> bool {
        if self.n != other.n {
            return false;
        }

        if self.difficulty != other.difficulty {
            return false;
        }

        for (sudoku_id, sudoku1) in self.sudokus.iter().enumerate() {
            let sudoku2 = other.sudokus.get(sudoku_id).unwrap();

            for x in 0..self.n2 {
                for y in 0..self.n2 {
                    if sudoku1.get_cell_value(x, y) != sudoku2.get_cell_value(x, y)
                        || sudoku1
                            .get_cell_possibilities(x, y)
                            .ne(sudoku2.get_cell_possibilities(x, y))
                    {
                        return false;
                    }
                }
            }
        }

        for (key1, values1) in self.links.iter() {
            if let Some(values2) = other.links.get(key1) {
                if values1.ne(values2) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}
