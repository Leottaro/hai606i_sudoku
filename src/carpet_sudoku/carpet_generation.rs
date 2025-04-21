use super::{CarpetPattern, CarpetSudoku};
use crate::simple_sudoku::SudokuDifficulty;
use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    io::{stdout, Write},
    sync::{mpsc, Arc, Mutex},
    thread::{self, available_parallelism},
};

fn duration_to_string(duration: std::time::Duration) -> String {
    let milliseconds = duration.as_millis();
    let seconds = milliseconds / 1000;
    let minutes = milliseconds / 60_000;
    let hours = milliseconds / 3_600_000;
    if hours > 0 {
        format!(
            "{}h {}m {}.{}s",
            hours,
            minutes % 60,
            seconds % 60,
            milliseconds % 1000
        )
    } else if minutes > 0 {
        format!(
            "{}m {}.{}s",
            minutes % 60,
            seconds % 60,
            milliseconds % 1000
        )
    } else if seconds > 0 {
        format!("{}.{}s", seconds % 60, milliseconds % 1000)
    } else {
        format!("{}ms", milliseconds % 1000)
    }
}
struct CarpetGenerationThreadInput {
    pub rng: rand::rngs::ThreadRng,
    pub exploring_filled_cells: Vec<bool>,
    pub cells_to_remove: HashSet<(usize, usize, usize, usize)>,
}
struct CarpetGenerationLogInfos {
    pub start_time: std::time::Instant,
    pub explored_counter: usize,
    pub skipped_counter: usize,
    pub non_unique_counter: usize,
    pub can_remove_a_cell_counter: usize,
    pub wrong_difficulty_counter: usize,
    pub solvable_sub_carpet_counter: usize,
}

impl std::fmt::Display for CarpetGenerationLogInfos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
			f,
			"{}: explored:{} skipped:{} non_unique:{} can_remove_a_cell:{} not_right_difficulty:{} solvable_sub_carpet:{}",
			duration_to_string(self.start_time.elapsed()),
			self.explored_counter,
			self.skipped_counter,
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
        loop {
            let mut carpet = Self::new(n, pattern);
            if !carpet._generate_canonical_from(0, 0, 0) {
                panic!("pattern: {pattern} juste pas possible en fait");
            }
            if carpet.count_solutions(Some(1), None) == 0 {
                println!("bloqué ici: {carpet}");
                continue;
            }

            for sudoku in carpet.sudokus.iter_mut() {
                sudoku.set_is_canonical(true);
            }
            carpet.is_canonical = true;

            if carpet.backtrack_solve() {
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
        loop {
            if x == self.n2 {
                y += 1;
                x = 0;
            }
            if y == self.n2 {
                y = 0;
                sudoku_id += 1;
            }
            if sudoku_id == self.sudokus.len() {
                return true;
            }
            if (y == 0 || x == 0) && self.sudokus[sudoku_id].get_cell_value(x, y) == 0 {
                break;
            }
            x += 1;
        }

        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.sort();
        for value in possibilities {
            match self.set_value(sudoku_id, x, y, value) {
                Ok(()) => (),
                Err(_) => {
                    let _ = self.remove_value(sudoku_id, x, y);
                    continue;
                }
            }

            if self.count_solutions(Some(1), None) > 0
                && self._generate_canonical_from(sudoku_id, x + 1, y)
            {
                return true;
            }

            if let Err(err) = self.remove_value(sudoku_id, x, y) {
                eprintln!(
                    "ERRROR AFTER self.remove_value({sudoku_id}, {x}, {y}): {err}\nFOR CARPET:{self}"
                );
            }
        }

        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GAMES   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_new(n: usize, pattern: CarpetPattern, difficulty: SudokuDifficulty) -> Self {
        Self::generate_full(n, pattern).into_generate_from(difficulty)
    }

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Self {
        self.clone().into_generate_from(aimed_difficulty)
    }

    pub fn into_generate_from(mut self, aimed_difficulty: SudokuDifficulty) -> Self {
        self.difficulty = SudokuDifficulty::Unknown;

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
        let original_cells_to_remove = temp
            .into_iter()
            .filter(|(_, _, _, value)| *value > 0)
            .collect::<HashSet<_>>();

        let already_explored_filled_cells = Arc::new(Mutex::new(HashSet::new()));
        let log_infos = Arc::new(Mutex::new(CarpetGenerationLogInfos {
            start_time: std::time::Instant::now(),
            explored_counter: 0,
            skipped_counter: 0,
            non_unique_counter: 0,
            can_remove_a_cell_counter: 0,
            wrong_difficulty_counter: 0,
            solvable_sub_carpet_counter: 0,
        }));
        let starting_points = Arc::new(Mutex::new(
            (0..self.sudokus.len() * self.n2 * self.n2)
                .map(|i| {
                    let sudoku_id = i / (self.n2 * self.n2);
                    let cell_i = i - sudoku_id * self.n2 * self.n2;
                    let y = cell_i / self.n2;
                    let x = cell_i % self.n2;
                    let value = self.sudokus[sudoku_id].get_cell_value(x, y);

                    let mut starting_carpet = self.clone();
                    let mut starting_exploring_filled_cells =
                        original_exploring_filled_cells.clone();
                    let mut starting_cells_to_remove = original_cells_to_remove.clone();

                    starting_carpet.remove_value(sudoku_id, x, y).unwrap();
                    for (sudoku_id, x, y) in self.get_twin_cells(sudoku_id, x, y) {
                        starting_exploring_filled_cells[(sudoku_id * self.n2 + y) * self.n2 + x] =
                            false;
                        starting_cells_to_remove.remove(&(sudoku_id, x, y, value));
                    }

                    (
                        starting_carpet,
                        starting_exploring_filled_cells,
                        starting_cells_to_remove,
                    )
                })
                .collect::<Vec<_>>()
                .into_iter(),
        ));

        let (carpet_tx, carpet_rx) = mpsc::channel();
        let thread_count = available_parallelism().unwrap().get();
        let mut thread_infos = Vec::new();

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
                            rng: rng.clone(),
                            exploring_filled_cells: starting_exploring_filled_cells,
                            cells_to_remove: starting_cells_to_remove,
                        };

                        if *thread_should_stop.lock().unwrap() {
                            break;
                        }

                        starting_carpet.difficulty = SudokuDifficulty::Unknown;
                        if starting_carpet._generate_from(
                            aimed_difficulty,
                            &mut carpet_generation_input,
                            &thread_should_stop,
                            &already_explored_filled_cells,
                            &log_infos,
                        ) {
                            let _ = carpet_tx.send(starting_carpet);
                        }
                    }
                })
                .unwrap();
            thread_infos.push((should_stop, join_handle));
        }

        let carpet = carpet_rx.recv().unwrap();

        for (should_stop, _) in thread_infos.iter() {
            *should_stop.lock().unwrap() = true;
        }
        for (_, join_handle) in thread_infos {
            let _ = join_handle.join();
        }

        println!("{}", log_infos.lock().unwrap());
        carpet
    }

    fn _generate_from(
        &mut self,
        aimed_difficulty: SudokuDifficulty,
        carpet_generation_input: &mut CarpetGenerationThreadInput,
        thread_should_stop: &Arc<Mutex<bool>>,
        already_explored_filled_cells: &Arc<Mutex<HashSet<Vec<bool>>>>,
        log_infos: &Arc<Mutex<CarpetGenerationLogInfos>>,
    ) -> bool {
        // stop if a solution was found by another thread
        if *thread_should_stop.lock().unwrap() {
            return false;
        }

        // skip if this possibility has already been explored
        if !already_explored_filled_cells
            .lock()
            .unwrap()
            .insert(carpet_generation_input.exploring_filled_cells.clone())
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.skipped_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
            return false;
        }

        // skip if this possibility has not a unique solution
        if !self.is_unique(Some(&already_explored_filled_cells.lock().unwrap().clone())) {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.non_unique_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
            return false;
        }

        // printing progress
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.explored_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
        }

        // for each cell we can remove (in random order for variety)
        let mut randomized_cells_to_remove = carpet_generation_input
            .cells_to_remove
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        randomized_cells_to_remove.shuffle(&mut carpet_generation_input.rng);
        let mut can_remove_a_cell = false;
        for (sudoku_id, x, y, removed_value) in randomized_cells_to_remove {
            // stop if a solution was found by another thread
            if *thread_should_stop.lock().unwrap() {
                return false;
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
                if self._generate_from(
                    aimed_difficulty,
                    carpet_generation_input,
                    thread_should_stop,
                    already_explored_filled_cells,
                    log_infos,
                ) {
                    // if a solution was found, stop everything
                    return true;
                }
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
            return false;
        }

        // if no cell can be removed...
        if can_remove_a_cell {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.can_remove_a_cell_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
            return false;
        }

        // if we can solve the carpet and its the right difficulty...
        let mut verify_carpet = self.clone();
        verify_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
        if !verify_carpet.is_filled() || verify_carpet.difficulty != aimed_difficulty {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.wrong_difficulty_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
            return false;
        }

        // and if we can't solve any of the sub carpets...
        for sub_links in self.pattern.get_sub_links(self.n) {
            // stop if a solution was found by another thread
            if *thread_should_stop.lock().unwrap() {
                return false;
            }

            let sub_sudokus = self.sudokus.clone();
            let mut sub_carpet = CarpetSudoku::new_custom(self.n, sub_sudokus, sub_links);
            sub_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
            if sub_carpet.is_filled() {
                let mut log_infos = log_infos.lock().unwrap();
                log_infos.solvable_sub_carpet_counter += 1;
                print!("{log_infos}          \r");
                stdout().flush().unwrap();
                return false;
            }
        }

        // we just found a solution !
        self.difficulty = aimed_difficulty;
        true
    }
}
