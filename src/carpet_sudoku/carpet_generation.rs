use super::{CarpetPattern, CarpetSudoku};
use crate::simple_sudoku::{sudoku_generation::duration_to_string, SudokuDifficulty, SudokuGroups};
use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    io::{stdout, Write},
    sync::{mpsc, Arc, Mutex},
    thread::{self, available_parallelism},
};

struct CarpetGenerationThreadInput {
    pub tx: mpsc::Sender<Option<CarpetSudoku>>,
    pub rng: rand::rngs::ThreadRng,
    pub exploring_filled_cells: Vec<bool>,
    pub cells_to_remove: HashSet<(usize, usize, usize, usize)>,
}
struct CarpetGenerationLogInfos {
    pub start_time: std::time::Instant,
    pub explored_counter: usize,
    pub skipped_counter: usize,
    pub minimal_filled_cells_counter: usize,
    pub non_unique_counter: usize,
    pub can_remove_a_cell_counter: usize,
    pub wrong_difficulty_counter: usize,
    pub solvable_sub_carpet_counter: usize,
}

impl std::fmt::Display for CarpetGenerationLogInfos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
			f,
			"{} explored:{} skipped:{} below_minimal_filled_cells:{} non_unique:{} can_remove_a_cell:{} wrong_difficulty:{} solvable_sub_carpet:{}",
			duration_to_string(self.start_time.elapsed()),
			self.explored_counter,
			self.skipped_counter,
			self.minimal_filled_cells_counter,
			self.non_unique_counter,
			self.can_remove_a_cell_counter,
			self.wrong_difficulty_counter,
			self.solvable_sub_carpet_counter
		)
    }
}

impl CarpetSudoku {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   FILLED   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_full(n: usize, pattern: CarpetPattern) -> Self {
        Self::new(n, pattern).into_generate_full_from()
    }

    pub fn generate_full_from(&self) -> Self {
        self.clone().into_generate_full_from()
    }

    pub fn into_generate_full_from(self) -> Self {
        let mut tries = 0;
        loop {
            tries += 1;
            print!("\r{} generate_full has {tries} tries", self.pattern);
            let mut carpet = self.clone();
            if !carpet._generate_canonical_from(0, 0, 0) {
                panic!("pattern: {} juste pas possible en fait", carpet.pattern);
            }

            for sudoku in carpet.sudokus.iter_mut() {
                sudoku.set_is_canonical(true);
            }
            carpet.is_canonical = true;

            if carpet.backtrack_solve() {
                println!();
                return carpet;
            }
        }
    }

    fn _generate_canonical_from(
        &mut self,
        mut sudoku_id: usize,
        mut x: usize,
        mut y: usize,
    ) -> bool {
        let backup = self.clone();

        while self.sudokus[sudoku_id].get_cell_value(x, y) != 0 {
            if y == 0 {
                x += 1;

                if x == self.n2 {
                    x = 0;
                    sudoku_id += 1;
                }

                if sudoku_id == self.sudokus.len() {
                    sudoku_id = 0;
                    y = 1;
                }
            } else {
                y += 1;

                if y == self.n2 {
                    y = 1;
                    sudoku_id += 1;
                }

                if sudoku_id == self.sudokus.len() {
                    return true;
                }
            }
        }

        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.sort();

        for value in possibilities {
            if self.set_value(sudoku_id, x, y, value).is_ok()
                && self._generate_canonical_from(sudoku_id, x, y)
            {
                return true;
            }

            *self = backup.clone();
        }

        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GAMES   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_new(n: usize, pattern: CarpetPattern, difficulty: SudokuDifficulty) -> Self {
        Self::generate_full(n, pattern)
            .into_generate_from(difficulty)
            .unwrap()
    }

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Option<Self> {
        self.clone().into_generate_from(aimed_difficulty)
    }

    pub fn into_generate_from(mut self, aimed_difficulty: SudokuDifficulty) -> Option<Self> {
        self.difficulty = SudokuDifficulty::Unknown;
        self.difficulty_score = 0;

        let temp = (0..self.sudokus.len() * self.n2 * self.n2)
            .map(|i| {
                let sudoku_id = i / (self.n2 * self.n2);
                let cell_i = i - sudoku_id * self.n2 * self.n2;
                let y = cell_i / self.n2;
                let x = cell_i % self.n2;
                let value = self.sudokus[sudoku_id].get_cell_value(x, y);
                (sudoku_id, x, y, value)
            })
            .collect::<Vec<_>>();
        let original_exploring_filled_cells = temp
            .iter()
            .map(|(_, _, _, value)| *value > 0)
            .collect::<Vec<_>>();
        let original_cells_to_remove = {
            let mut cells_to_remove = HashSet::new();
            for (i, x, y, value) in temp {
                if value == 0 {
                    continue;
                }

                let mut to_insert = true;
                for (i2, x2, y2) in self.get_twin_cells(i, x, y) {
                    if cells_to_remove.contains(&(i2, x2, y2, value)) {
                        to_insert = false;
                        break;
                    }
                }

                if to_insert {
                    cells_to_remove.insert((i, x, y, value));
                }
            }
            cells_to_remove
        };

        let already_explored_filled_cells = Arc::new(Mutex::new(HashSet::new()));
        let log_infos = Arc::new(Mutex::new(CarpetGenerationLogInfos {
            start_time: std::time::Instant::now(),
            explored_counter: 0,
            skipped_counter: 0,
            minimal_filled_cells_counter: 0,
            non_unique_counter: 0,
            can_remove_a_cell_counter: 0,
            wrong_difficulty_counter: 0,
            solvable_sub_carpet_counter: 0,
        }));
        let starting_points = Arc::new(Mutex::new(
            (0..self.sudokus.len() * self.n2 * self.n2)
                .filter_map(|i| {
                    let sudoku_id = i / (self.n2 * self.n2);
                    let cell_i = i - sudoku_id * self.n2 * self.n2;
                    let y = cell_i / self.n2;
                    let x = cell_i % self.n2;
                    let value = self.sudokus[sudoku_id].get_cell_value(x, y);

                    if value == 0 {
                        return None;
                    }

                    let mut starting_carpet = self.clone();
                    let mut starting_exploring_filled_cells =
                        original_exploring_filled_cells.clone();
                    let mut starting_cells_to_remove = original_cells_to_remove.clone();

                    starting_carpet.remove_value(sudoku_id, x, y).unwrap();
                    starting_carpet.difficulty = SudokuDifficulty::Unknown;
                    starting_carpet.difficulty_score = 0;
                    for (sudoku_id, x, y) in self.get_twin_cells(sudoku_id, x, y) {
                        starting_exploring_filled_cells[(sudoku_id * self.n2 + y) * self.n2 + x] =
                            false;
                        starting_cells_to_remove.remove(&(sudoku_id, x, y, value));
                    }

                    Some((
                        starting_carpet,
                        starting_exploring_filled_cells,
                        starting_cells_to_remove,
                    ))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        ));

        let (carpet_tx, carpet_rx) = mpsc::channel();
        let thread_count = available_parallelism().unwrap().get();
        let mut threads_should_stop = Vec::new();
        let mut threads_join_handles = Vec::new();

        for thread_id in 0..thread_count {
            let starting_points = Arc::clone(&starting_points);
            let already_explored_filled_cells = Arc::clone(&already_explored_filled_cells);
            let log_infos = Arc::clone(&log_infos);
            let carpet_tx = carpet_tx.clone();
            let should_stop = Arc::new(Mutex::new(false));

            let thread_should_stop = Arc::clone(&should_stop);
            let join_handle = thread::Builder::new()
                .name(format!("thread-{thread_id}"))
                .spawn(move || {
                    let rng = rand::rng();

                    while let Some((
                        mut starting_carpet,
                        starting_exploring_filled_cells,
                        starting_cells_to_remove,
                    )) = starting_points
                        .lock()
                        .ok()
                        .and_then(|mut owned_starting_points| owned_starting_points.next())
                    {
                        let mut carpet_generation_input = CarpetGenerationThreadInput {
                            tx: carpet_tx.clone(),
                            rng: rng.clone(),
                            exploring_filled_cells: starting_exploring_filled_cells,
                            cells_to_remove: starting_cells_to_remove,
                        };

                        if *thread_should_stop.lock().unwrap() {
                            break;
                        }

                        starting_carpet._generate_from(
                            aimed_difficulty,
                            &mut carpet_generation_input,
                            &thread_should_stop,
                            &already_explored_filled_cells,
                            &log_infos,
                        );

                        let _ = carpet_tx.send(None);
                    }
                })
                .unwrap();
            threads_should_stop.push(should_stop);
            threads_join_handles.push(join_handle);
        }

        while starting_points.lock().unwrap().len() > 0 {
            let carpet = carpet_rx.recv().unwrap();
            if carpet.is_none() {
                continue;
            }
            let mut carpet = carpet.unwrap();

            // if this possibility isn't unique
            if !carpet.is_unique() {
                continue;
            }

            for should_stop in threads_should_stop {
                *should_stop.lock().unwrap() = true;
            }
            for join_handle in threads_join_handles {
                let _ = join_handle.join();
            }

            {
                let mut solved_carpet = carpet.clone();
                solved_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
                carpet.difficulty = solved_carpet.difficulty;
                carpet.difficulty_score = solved_carpet.difficulty_score;
            }

            println!(
                "{} {} score {}: {}",
                carpet.pattern,
                carpet.difficulty,
                carpet.difficulty_score,
                log_infos.lock().unwrap()
            );
            return Some(carpet);
        }
        None
    }

    fn _generate_from(
        &mut self,
        aimed_difficulty: SudokuDifficulty,
        carpet_generation_input: &mut CarpetGenerationThreadInput,
        thread_should_stop: &Arc<Mutex<bool>>,
        already_explored_filled_cells: &Arc<Mutex<HashSet<Vec<bool>>>>,
        log_infos: &Arc<Mutex<CarpetGenerationLogInfos>>,
    ) {
        // stop if a solution was found by another thread
        if *thread_should_stop.lock().unwrap() {
            return;
        }

        // skip if this possibility has already been explored
        if !already_explored_filled_cells
            .lock()
            .unwrap()
            .insert(carpet_generation_input.exploring_filled_cells.clone())
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.skipped_counter += 1;
            print!(
                "{} {}: {}          \r",
                self.pattern, aimed_difficulty, log_infos
            );
            stdout().flush().unwrap();
            return;
        }

        // skip if we are below the minimal filled cells
        if carpet_generation_input.cells_to_remove.len() < (2 * self.n2 - 1) {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.minimal_filled_cells_counter += 1;
            print!(
                "{} {}: {}          \r",
                self.pattern, aimed_difficulty, log_infos
            );
            stdout().flush().unwrap();
            return;
        }

        // printing progress
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.explored_counter += 1;
            print!(
                "{} {}: {}          \r",
                self.pattern, aimed_difficulty, log_infos
            );
            stdout().flush().unwrap();
        }

        // for each cell we can remove (in random order for variety)
        let mut randomized_cells_to_remove = carpet_generation_input
            .cells_to_remove
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        randomized_cells_to_remove.shuffle(&mut carpet_generation_input.rng);
        randomized_cells_to_remove.sort_by_key(|(sudoku_id, x, y, _)| {
            let mut possibilities = (1..=self.n2).collect::<HashSet<_>>();
            for (i, x, y) in self.get_global_cell_group(*sudoku_id, *x, *y, SudokuGroups::All) {
                let cell_value = self.sudokus[i].get_cell_value(x, y);
                if cell_value != 0 {
                    possibilities.remove(&cell_value);
                }
            }
            possibilities.len()
        });

        let mut can_remove_a_cell = false;
        for (sudoku_id, x, y, removed_value) in randomized_cells_to_remove {
            // stop if a solution was found by another thread
            if *thread_should_stop.lock().unwrap() {
                return;
            }

            let twin_cells = self.get_twin_cells(sudoku_id, x, y);
            // remove the cell and its twins
            self.remove_value(sudoku_id, x, y).unwrap();
            for &(i, x, y) in &twin_cells {
                carpet_generation_input.exploring_filled_cells[(i * self.n2 + y) * self.n2 + x] =
                    false;
                carpet_generation_input
                    .cells_to_remove
                    .remove(&(i, x, y, removed_value));
            }

            // if we can still solve the carpet
            let mut carpet = self.clone();
            carpet.rule_solve_until((false, false), Some(aimed_difficulty));
            if carpet.is_filled() {
                can_remove_a_cell = true;
                // recurcively try to remove more cells
                self._generate_from(
                    aimed_difficulty,
                    carpet_generation_input,
                    thread_should_stop,
                    already_explored_filled_cells,
                    log_infos,
                );
            }

            // add back the cell and its twins
            self.set_value(sudoku_id, x, y, removed_value).unwrap();
            for (i, x, y) in twin_cells {
                carpet_generation_input.exploring_filled_cells[(i * self.n2 + y) * self.n2 + x] =
                    true;
                carpet_generation_input
                    .cells_to_remove
                    .insert((i, x, y, removed_value));
            }
        }

        // stop if a solution was found by another thread
        if *thread_should_stop.lock().unwrap() {
            return;
        }

        // if no cell can be removed...
        if can_remove_a_cell {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.can_remove_a_cell_counter += 1;
            print!(
                "{} {}: {}          \r",
                self.pattern, aimed_difficulty, log_infos
            );
            stdout().flush().unwrap();
            return;
        }

        // if we can solve the carpet and its the right difficulty...
        let mut verify_carpet = self.clone();
        verify_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
        if !verify_carpet.is_filled() || verify_carpet.difficulty != aimed_difficulty {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.wrong_difficulty_counter += 1;
            print!(
                "{} {}: {}          \r",
                self.pattern, aimed_difficulty, log_infos
            );
            stdout().flush().unwrap();
            return;
        }

        // and if we can't solve any of the sub carpets...
        for sub_links in self.pattern.get_sub_links(self.n) {
            // stop if a solution was found by another thread
            if *thread_should_stop.lock().unwrap() {
                return;
            }

            let sub_sudokus = self.sudokus.clone();
            let mut sub_carpet = CarpetSudoku::new_custom(self.n, sub_sudokus, sub_links);
            sub_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
            if sub_carpet.is_filled() {
                let mut log_infos = log_infos.lock().unwrap();
                log_infos.solvable_sub_carpet_counter += 1;
                print!(
                    "{} {}: {}          \r",
                    self.pattern, aimed_difficulty, log_infos
                );
                stdout().flush().unwrap();
                return;
            }
        }

        // we just found a solution !
        carpet_generation_input.tx.send(Some(self.clone())).unwrap();
    }
}
