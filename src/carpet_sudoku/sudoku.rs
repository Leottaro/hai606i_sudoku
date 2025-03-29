use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
    sync::{mpsc, Arc, Mutex},
    thread::{available_parallelism, JoinHandle},
};

use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::simple_sudoku::{Sudoku, SudokuDifficulty, SudokuError, SudokuGroups};

use super::{CarpetPattern, CarpetSudoku};

type RawLink = ((usize, usize), (usize, usize));
impl CarpetSudoku {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   CREATION   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn new(n: usize, pattern: CarpetPattern) -> Self {
        let (sudokus, links) = match pattern {
            CarpetPattern::Double => Self::new_diagonal(n, 2),
            CarpetPattern::Diagonal(n_sudokus) => Self::new_diagonal(n, n_sudokus),
            CarpetPattern::Samurai => Self::new_samurai(n),
        };

        let mut hashmap: HashMap<usize, Vec<(usize, usize, usize)>> = HashMap::new();
        for ((sudoku1, square1), (sudoku2, square2)) in links {
            if let Some(sudoku_links) = hashmap.get_mut(&sudoku1) {
                sudoku_links.push((square1, sudoku2, square2));
            } else {
                hashmap.insert(sudoku1, vec![(square1, sudoku2, square2)]);
            }
            if let Some(sudoku_links) = hashmap.get_mut(&sudoku2) {
                sudoku_links.push((square2, sudoku1, square1));
            } else {
                hashmap.insert(sudoku2, vec![(square2, sudoku1, square1)]);
            }
        }

        Self {
            n,
            n2: n * n,
            filled_cells: 0,
            difficulty: SudokuDifficulty::Unknown,
            sudokus,
            links: hashmap,
        }
    }

    fn new_diagonal(n: usize, n_sudokus: usize) -> (Vec<Sudoku>, Vec<RawLink>) {
        let sudokus = vec![Sudoku::new(n); n_sudokus];
        let links = (1..n_sudokus)
            .map(|i| ((i - 1, n - 1), (i, 2 * n)))
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

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   MODIFICATION   ///////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn update_link(&mut self) -> Result<(), SudokuError> {
        for (&sudoku1, links) in self.links.iter() {
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
                                panic!("AIE AIE AIIIIIEEEE");
                            }
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
        self.filled_cells += 1;
        self.sudokus[sudoku_id].set_value(x, y, value)?;
        self.update_link()
    }

    pub fn remove_value(
        &mut self,
        sudoku_id: usize,
        x: usize,
        y: usize,
    ) -> Result<usize, SudokuError> {
        self.filled_cells -= 1;
        let dx = x % self.n;
        let dy = y % self.n;
        let x0 = (x / self.n) * self.n;
        let y0 = (y / self.n) * self.n;
        let square_id = y0 * self.n + x0 / self.n;
        let value = self.sudokus[sudoku_id].remove_value(x, y)?;
        let mut is_in_link = false;

        for &(square1, sudoku2, square2) in self.links.get(&sudoku_id).unwrap() {
            if square_id != square1 {
                continue;
            }
            is_in_link = true;

            let y2 = (square2 / self.n) * self.n;
            let x2 = (square2 % self.n) * self.n;
            let _ = self.sudokus[sudoku2].remove_value(x2 + dx, y2 + dy);
        }

        if is_in_link {
            self.update_link()?;
            return Ok(value);
        }

        let modified_posibilities = self.sudokus[sudoku_id].get_cell_group(x, y, SudokuGroups::All);
        for (x, y) in modified_posibilities {
            let dx = x % self.n;
            let dy = y % self.n;
            let x0 = (x / self.n) * self.n;
            let y0 = (y / self.n) * self.n;
            let square_id = y0 * self.n + x0 / self.n;

            for &(square1, sudoku2, square2) in self.links.get(&sudoku_id).unwrap() {
                if square_id != square1 {
                    continue;
                }

                let y2 = (square2 / self.n) * self.n + dx;
                let x2 = (square2 % self.n) * self.n + dy;
                let mut can_insert_value = true;

                for (x3, y3) in self.sudokus[sudoku2].get_cell_group(x2, y2, SudokuGroups::All) {
                    if self.sudokus[sudoku2].get_cell_value(x3, y3) == value {
                        can_insert_value = false;
                        break;
                    }
                }

                if can_insert_value {
                    self.sudokus[sudoku2]
                        .get_cell_possibilities_mut(x2, y2)
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
                Ok(Some(1 | 2)) => {
                    modified_value = true;
                    modified_possibility = true;
                }
                Ok(Some(_)) => modified_possibility = true,
                _ => (),
            }
            self.difficulty = self.difficulty.max(sudoku.get_difficulty());
        }
        self.update_link()?;
        Ok((modified_possibility, modified_value))
    }

    pub fn generate_full(n: usize, pattern: CarpetPattern) -> Self {
        let mut carpet = Self::new(n, pattern);
        carpet.backtrack_solve(0, 0, 0);
        carpet
    }

    pub fn backtrack_solve(&mut self, mut sudoku_id: usize, mut x: usize, mut y: usize) -> bool {
        // println!("{self}");
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
            if self.set_value(sudoku_id, x, y, value).is_err() {
                if self.sudokus[sudoku_id].get_cell_value(x, y) != 0 {
                    let _ = self.remove_value(sudoku_id, x, y);
                }
                continue;
            }

            if self.backtrack_solve(sudoku_id, x + 1, y) {
                return true;
            }

            let _ = self.remove_value(sudoku_id, x, y);
        }

        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////   GENERATION   ////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Self {
        let n2 = self.n2;
        let n_sudokus = self.sudokus.len();

        let (tx, rx) = mpsc::channel::<Self>();
        type SudokuFilledCells = (CarpetSudoku, Vec<bool>);

        loop {
            let thread_count: usize = available_parallelism().unwrap().get();
            let default = {
                let filled_cells: Vec<bool> = (0..n_sudokus * n2 * n2)
                    .map(|i| {
                        let sudoku_id = i / (n2 * n2);
                        let cell_i = i - sudoku_id * n2 * n2;
                        let y = cell_i / n2;
                        let x = cell_i % n2;
                        self.sudokus[sudoku_id].get_cell_value(x, y).ne(&0)
                    })
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
                let (main_tx, thread_rx) = mpsc::channel::<()>();

                let join_handle = std::thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    while thread_rx.try_recv().is_err() {
                        let (mut carpet, filled_cells) = thread_to_explore
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
                                    Ok((_, true)) => {
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
                                    Ok((true, false)) => (),
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            if !can_solve {
                                continue;
                            }

                            if testing_carpet.filled_cells < n_sudokus * (n2 * 2 - 1) {
                                thread_explored_filled_cells
                                    .lock()
                                    .unwrap()
                                    .insert(filled_cells.clone());
                            } else {
                                // EXPLORATION EN PROFONDEUR
                                let mut passed_sudoku = carpet.clone();
                                passed_sudoku.remove_value(sudoku_id, x, y).unwrap();
                                passed_sudoku.difficulty = testing_carpet.difficulty;

                                let mut passed_filled_cells = filled_cells.clone();
                                passed_filled_cells[i] = false;

                                thread_to_explore
                                    .lock()
                                    .unwrap()
                                    .push((passed_sudoku, passed_filled_cells));

                                working_sub_sudokus += 1;
                            }
                        }

                        if working_sub_sudokus == 0 && carpet.difficulty == aimed_difficulty {
                            carpet.difficulty = SudokuDifficulty::Unknown;
                            let _ = thread_tx.send(carpet);
                            break;
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
                let carpet = rx.recv().unwrap();

                // verify that the sudoku is unique
                if !carpet.is_unique() {
                    continue;
                }

                // panic if generated a wrong sudoku
                let mut verify_carpet = carpet.clone();
                loop {
                    match verify_carpet.rule_solve(None) {
                        Ok((false, false)) => {
                            if !verify_carpet.is_filled() {
                                panic!("ERROR IN SUDOKU SOLVING: Couldn't solve generated sudoku: \nORIGINAL SUDOKU:\n{carpet}\nFINISHED SUDOKU: \n{verify_carpet}");
                            }
                            break;
                        }
                        Ok(_) => (),
                        Err(err) => {
                            panic!("ERROR IN SUDOKU: {err}: \nORIGINAL SUDOKU: {carpet}\nLAST SUDOKU: {verify_carpet}");
                        }
                    }
                }

                for (handle, tx) in threads_infos {
                    let _ = tx.send(());
                    handle.join().unwrap();
                }

                return carpet;
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   UTILITY   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn is_filled(&self) -> bool {
        self.filled_cells
            == self.sudokus.len() * self.n2 * self.n2 - self.links.keys().len() * self.n2
    }

    pub fn is_unique(&self) -> bool {
        self.clone().count_solutions(0, 0, 0, Some(1)) <= 1
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

        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut thread_rng());
        let mut sub_solutions = 0;
        for value in possibilities {
            if self.set_value(sudoku_id, x, y, value).is_err() {
                if self.sudokus[sudoku_id].get_cell_value(x, y) != 0 {
                    let _ = self.remove_value(sudoku_id, x, y);
                }
                continue;
            }

            sub_solutions += self.count_solutions(sudoku_id, x + 1, y, max_solutions);
            if let Some(max_solutions) = max_solutions {
                if sub_solutions >= max_solutions {
                    return sub_solutions;
                }
            }

            let _ = self.remove_value(sudoku_id, x, y);
        }

        sub_solutions
    }
}

impl std::fmt::Display for CarpetSudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, sudoku) in self.sudokus.iter().enumerate() {
            writeln!(f, "Sudoku {}:\n{}", i, sudoku)?;
        }
        Ok(())
    }
}
