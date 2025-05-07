use std::{
    collections::HashSet,
    io::{stdout, Write},
    sync::{mpsc, Arc, Mutex},
    thread::{self, available_parallelism},
};

use rand::seq::SliceRandom;

use super::{Sudoku, SudokuDifficulty, SudokuGroups};

pub fn duration_to_string(duration: std::time::Duration) -> String {
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

struct SudokuGenerationThreadInput {
    pub tx: mpsc::Sender<Option<Sudoku>>,
    pub rng: rand::rngs::ThreadRng,
    pub exploring_filled_cells: Vec<bool>,
    pub cells_to_remove: HashSet<(usize, usize, usize)>,
}
struct SudokuGenerationLogInfos {
    pub start_time: std::time::Instant,
    pub explored_counter: usize,
    pub skipped_counter: usize,
    pub minimal_filled_cells_counter: usize,
    pub non_unique_counter: usize,
    pub can_remove_a_cell_counter: usize,
    pub wrong_difficulty_counter: usize,
}

impl std::fmt::Display for SudokuGenerationLogInfos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
			f,
			"{}: explored:{} skipped:{} below_minimal_filled_cells:{} non_unique:{} can_remove_a_cell:{} wrong_difficulty:{}",
			duration_to_string(self.start_time.elapsed()),
			self.explored_counter,
			self.skipped_counter,
			self.minimal_filled_cells_counter,
			self.non_unique_counter,
			self.can_remove_a_cell_counter,
			self.wrong_difficulty_counter,
		)
    }
}

impl Sudoku {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   FILLED   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GAMES   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_new(n: usize, aimed_difficulty: SudokuDifficulty) -> Self {
        Sudoku::generate_full(n)
            .into_generate_from(aimed_difficulty)
            .unwrap()
    }

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Option<Self> {
        self.clone().into_generate_from(aimed_difficulty)
    }

    pub fn into_generate_from(mut self, aimed_difficulty: SudokuDifficulty) -> Option<Self> {
        self.difficulty = SudokuDifficulty::Unknown;

        let temp = (0..self.n2 * self.n2)
            .map(|cell_i| {
                let y = cell_i / self.n2;
                let x = cell_i % self.n2;
                let value = self.board[y][x];
                (x, y, value)
            })
            .collect::<Vec<_>>();
        let original_exploring_filled_cells = temp
            .iter()
            .map(|(_, _, value)| *value > 0)
            .collect::<Vec<_>>();
        let original_cells_to_remove = temp
            .into_iter()
            .filter(|(_, _, value)| *value > 0)
            .collect::<HashSet<_>>();

        let already_explored_filled_cells = Arc::new(Mutex::new(HashSet::new()));
        let log_infos = Arc::new(Mutex::new(SudokuGenerationLogInfos {
            start_time: std::time::Instant::now(),
            explored_counter: 0,
            skipped_counter: 0,
            minimal_filled_cells_counter: 0,
            non_unique_counter: 0,
            can_remove_a_cell_counter: 0,
            wrong_difficulty_counter: 0,
        }));
        let starting_points = Arc::new(Mutex::new(
            (0..self.n2 * self.n2)
                .filter_map(|cell_i| {
                    let y = cell_i / self.n2;
                    let x = cell_i % self.n2;
                    let value = self.board[y][x];

                    if value == 0 {
                        return None;
                    }

                    let mut starting_sudoku = self.clone();
                    let mut starting_exploring_filled_cells =
                        original_exploring_filled_cells.clone();
                    let mut starting_cells_to_remove = original_cells_to_remove.clone();

                    starting_sudoku.remove_value(x, y).unwrap();
                    starting_sudoku.difficulty = SudokuDifficulty::Unknown;
                    starting_exploring_filled_cells[y * self.n2 + x] = false;
                    starting_cells_to_remove.remove(&(x, y, value));

                    Some((
                        starting_sudoku,
                        starting_exploring_filled_cells,
                        starting_cells_to_remove,
                    ))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        ));

        let (sudoku_tx, sudoku_rx) = mpsc::channel();
        let thread_count = available_parallelism().unwrap().get();
        let mut threads_should_stop = Vec::new();
        let mut threads_join_handles = Vec::new();

        for thread_id in 0..thread_count {
            let starting_points = Arc::clone(&starting_points);
            let already_explored_filled_cells = Arc::clone(&already_explored_filled_cells);
            let log_infos = Arc::clone(&log_infos);
            let sudoku_tx = sudoku_tx.clone();
            let should_stop = Arc::new(Mutex::new(false));

            let thread_should_stop = Arc::clone(&should_stop);
            let join_handle = thread::Builder::new()
                .name(format!("thread-{thread_id}"))
                .spawn(move || {
                    let rng = rand::rng();

                    while let Some((
                        mut starting_sudoku,
                        starting_exploring_filled_cells,
                        starting_cells_to_remove,
                    )) = starting_points
                        .lock()
                        .ok()
                        .and_then(|mut owned_starting_points| owned_starting_points.next())
                    {
                        let mut sudoku_generation_input = SudokuGenerationThreadInput {
                            tx: sudoku_tx.clone(),
                            rng: rng.clone(),
                            exploring_filled_cells: starting_exploring_filled_cells,
                            cells_to_remove: starting_cells_to_remove,
                        };

                        if *thread_should_stop.lock().unwrap() {
                            break;
                        }

                        starting_sudoku._generate_from(
                            aimed_difficulty,
                            &mut sudoku_generation_input,
                            &thread_should_stop,
                            &already_explored_filled_cells,
                            &log_infos,
                        );

                        let _ = sudoku_tx.send(None);
                    }
                })
                .unwrap();
            threads_should_stop.push(should_stop);
            threads_join_handles.push(join_handle);
        }

        while starting_points.lock().unwrap().len() > 0 {
            let sudoku = sudoku_rx.recv().unwrap();
            if sudoku.is_none() {
                continue;
            }
            let mut sudoku = sudoku.unwrap();

            // if this possibility isn't unique
            if !self.is_unique() {
                continue;
            }

            for should_stop in threads_should_stop {
                *should_stop.lock().unwrap() = true;
            }
            for join_handle in threads_join_handles {
                let _ = join_handle.join();
            }

            println!("{}: {}", aimed_difficulty, log_infos.lock().unwrap());
            sudoku.difficulty = aimed_difficulty;
            return Some(sudoku);
        }
        None
    }

    fn _generate_from(
        &mut self,
        aimed_difficulty: SudokuDifficulty,
        sudoku_generation_input: &mut SudokuGenerationThreadInput,
        thread_should_stop: &Arc<Mutex<bool>>,
        already_explored_filled_cells: &Arc<Mutex<HashSet<Vec<bool>>>>,
        log_infos: &Arc<Mutex<SudokuGenerationLogInfos>>,
    ) {
        // stop if a solution was found by another thread
        if *thread_should_stop.lock().unwrap() {
            return;
        }

        // skip if we are below the minimal filled cells
        if sudoku_generation_input.cells_to_remove.len() < (2 * self.n2 - 1) {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.minimal_filled_cells_counter += 1;
            print!("{log_infos}          \r");
            stdout().flush().unwrap();
            return;
        }

        // skip if this possibility has already been explored
        if !already_explored_filled_cells
            .lock()
            .unwrap()
            .insert(sudoku_generation_input.exploring_filled_cells.clone())
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.skipped_counter += 1;
            print!("{}          \r", log_infos);
            stdout().flush().unwrap();
            return;
        }

        // printing progress
        {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.explored_counter += 1;
            print!("{}          \r", log_infos);
            stdout().flush().unwrap();
        }

        // for each cell we can remove (in random order for variety)
        let mut randomized_cells_to_remove = sudoku_generation_input
            .cells_to_remove
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        randomized_cells_to_remove.shuffle(&mut sudoku_generation_input.rng);
        randomized_cells_to_remove.sort_by_key(|(x, y, _)| {
            let mut possibilities = (1..=self.n2).collect::<HashSet<_>>();
            for (x, y) in self.get_cell_group(*x, *y, SudokuGroups::All) {
                let cell_value = self.board[y][x];
                if cell_value != 0 {
                    possibilities.remove(&cell_value);
                }
            }
            possibilities.len()
        });

        let mut can_remove_a_cell = false;
        for (x, y, removed_value) in randomized_cells_to_remove {
            // stop if a solution was found by another thread
            if *thread_should_stop.lock().unwrap() {
                return;
            }

            // remove the cell and its twins
            self.remove_value(x, y).unwrap();
            sudoku_generation_input.exploring_filled_cells[y * self.n2 + x] = false;
            sudoku_generation_input
                .cells_to_remove
                .remove(&(x, y, removed_value));

            // if we can still solve the sudoku
            let mut sudoku = self.clone();
            sudoku.rule_solve_until(None, None, Some(aimed_difficulty));
            if sudoku.is_filled() {
                can_remove_a_cell = true;
                // recurcively try to remove more cells
                self._generate_from(
                    aimed_difficulty,
                    sudoku_generation_input,
                    thread_should_stop,
                    already_explored_filled_cells,
                    log_infos,
                );
            }

            // add back the cell and its twins
            self.set_value(x, y, removed_value).unwrap();
            sudoku_generation_input.exploring_filled_cells[y * self.n2 + x] = true;
            sudoku_generation_input
                .cells_to_remove
                .insert((x, y, removed_value));
        }

        // stop if a solution was found by another thread
        if *thread_should_stop.lock().unwrap() {
            return;
        }

        // if no cell can be removed...
        if can_remove_a_cell {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.can_remove_a_cell_counter += 1;
            print!("{}          \r", log_infos);
            stdout().flush().unwrap();
            return;
        }

        // and if we can solve the sudoku and its the right difficulty...
        let mut verify_sudoku = self.clone();
        verify_sudoku.rule_solve_until(None, None, Some(aimed_difficulty));
        if !verify_sudoku.is_filled() || verify_sudoku.difficulty != aimed_difficulty {
            let mut log_infos = log_infos.lock().unwrap();
            log_infos.wrong_difficulty_counter += 1;
            print!("{}          \r", log_infos);
            stdout().flush().unwrap();
            return;
        }

        // we just found a solution !
        sudoku_generation_input.tx.send(Some(self.clone())).unwrap();
    }
}
