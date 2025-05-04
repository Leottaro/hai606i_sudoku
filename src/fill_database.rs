use std::{
    io::{stdout, Write},
    ops::SubAssign,
    sync::{mpsc, Arc, Mutex},
    thread::{self, available_parallelism},
};

use hai606i_sudoku::{
    carpet_sudoku::{CarpetPattern, CarpetSudoku},
    database::Database,
    simple_sudoku::{Sudoku, SudokuDifficulty},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Wrong usage: needed 4 args, got {}", args.len());
        eprintln!(
            "Usage: {} <sudoku|carpet> <filled|games> <max_number>",
            args[0]
        );
        return;
    }
    let max_number = args[3].parse::<usize>().unwrap();

    if args[1].eq("sudoku") {
        if args[2].eq("filled") {
            sudoku_filled(max_number);
            return;
        } else if args[2].eq("games") {
            sudoku_games(max_number);
            return;
        }
    } else if args[1].eq("carpet") {
        if args[2].eq("filled") {
            carpet_filled(max_number);
            return;
        } else if args[2].eq("games") {
            carpet_games(max_number);
            return;
        }
    }
    eprintln!(
        "Usage: {} <sudoku|carpet> <filled|games> <max_number>",
        args[0]
    );
}

fn sudoku_filled(max_number: usize) {
    let mut database = Database::connect().unwrap();
    let mut overlap_count: u128 = 0;
    let mut total_count: u128 = 0;

    let remaining_number = Arc::new(Mutex::new(max_number));
    let data = Arc::new(Mutex::new(Vec::new()));

    let threads_number = available_parallelism().unwrap();
    let mut thread_handles = Vec::new();
    for thread_id in 0..threads_number.into() {
        let remaining_number = Arc::clone(&remaining_number);
        let thread_data = Arc::clone(&data);
        let handle = thread::Builder::new()
            .name(thread_id.to_string())
            .spawn(move || {
                while *remaining_number.lock().unwrap() > 0 {
                    let sudoku_base = Sudoku::generate_full(3);
                    if let Ok(inserted_data) = sudoku_base.filled_to_db() {
                        thread_data.lock().unwrap().push(inserted_data);
                    }
                }
            })
            .unwrap();
        thread_handles.push(handle);
    }

    while *remaining_number.lock().unwrap() > 0 {
        print!(
            " {} filled sudoku remaining: {overlap_count}/{total_count} rows overlapped\r",
            remaining_number.lock().unwrap()
        );
        stdout().flush().unwrap();

        let (passed_sudokus, passed_squares) = {
            let mut locked = data.lock().unwrap();
            let drain_number = remaining_number.lock().unwrap().min(locked.len());
            let (sudokus, squares): (Vec<_>, Vec<_>) = locked.drain(0..drain_number).unzip();
            (sudokus, squares.into_iter().flatten().collect::<Vec<_>>())
        };

        let total_rows = passed_sudokus.len() + passed_squares.len();
        let (just_inserted_sudokus, just_inserted_squares) = database
            .insert_multiple_canonical_sudokus(passed_sudokus, passed_squares)
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT SUDOKU FILLED IN DATABSE: {err}"));

        remaining_number
            .lock()
            .unwrap()
            .sub_assign(just_inserted_sudokus);
        total_count += total_rows as u128;
        overlap_count += (total_rows - just_inserted_sudokus - just_inserted_squares) as u128;
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}

fn sudoku_games(max_number: usize) {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));

    let (mut remaining_canonicals, canonicals) = {
        let temp = database
            .lock()
            .unwrap()
            .get_n_canonical_sudokus(max_number as i64)
            .unwrap();
        (temp.len(), temp.into_iter())
    };

    for canonical in canonicals {
        remaining_canonicals.sub_assign(1);
        let sudoku = Sudoku::db_from_filled(canonical);
        let mut passed_games = Vec::new();
        println!("{remaining_canonicals} filled sudokus{}", " ".repeat(50));

        for difficulty in SudokuDifficulty::iter() {
            println!("{difficulty}:{}", " ".repeat(50));

            let game = sudoku.generate_from(difficulty).unwrap();
            let mut game_db = game.game_to_db().unwrap();
            game_db.sudoku_game_difficulty = difficulty as i16;
            passed_games.push(game_db);
        }

        let thread_database = Arc::clone(&database);
        thread::spawn(move || {
            thread_database
                .lock()
                .unwrap()
                .insert_multiple_canonical_sudoku_game(passed_games)
                .unwrap_or_else(|err| {
                    panic!("ERROR COULDN'T INSERT SUDOKU GAME IN DATABSE: {err}")
                });
        });
    }
}

fn carpet_filled(max_number: usize) {
    let mut database = Database::connect().unwrap();
    let mut overlap_count: u128 = 0;
    let mut total_count: u128 = 0;

    let remaining_number = Arc::new(Mutex::new(max_number * CarpetPattern::iter().count()));
    let (sender, receiver) = mpsc::channel();
    let patterns_loop = Arc::new(Mutex::new(
        CarpetPattern::iter()
            .collect::<Vec<_>>()
            .into_iter()
            .cycle(),
    ));

    let threads_number = available_parallelism().unwrap();
    let mut thread_handles = Vec::new();
    for thread_id in 0..threads_number.into() {
        let thread_remaining_number = Arc::clone(&remaining_number);
        let thread_sender = sender.clone();
        let thread_patterns_loop = Arc::clone(&patterns_loop);
        let handle = thread::Builder::new()
            .name(thread_id.to_string())
            .spawn(move || {
                while *thread_remaining_number.lock().unwrap() > 0 {
                    let mut next_pattern = thread_patterns_loop.lock().unwrap().next();
                    while next_pattern.is_none() {
                        next_pattern = thread_patterns_loop.lock().unwrap().next();
                    }

                    let carpet_base = CarpetSudoku::generate_full(3, next_pattern.unwrap());
                    let (db_carpet, db_carpet_sudokus) = carpet_base.db_to_filled().unwrap();
                    let (sudokus_data, sudokus_data_square): (Vec<_>, Vec<_>) = carpet_base
                        .db_sudokus_to_filled()
                        .unwrap()
                        .into_iter()
                        .unzip();
                    thread_sender
                        .send((
                            (db_carpet, db_carpet_sudokus),
                            (
                                sudokus_data,
                                sudokus_data_square
                                    .into_iter()
                                    .flatten()
                                    .collect::<Vec<_>>(),
                            ),
                        ))
                        .unwrap();

                    let mut remaining = thread_remaining_number.lock().unwrap();
                    if *remaining > 0 {
                        remaining.sub_assign(1);
                    }
                }
            })
            .unwrap();
        thread_handles.push(handle);
    }

    let mut finished = false;
    while !finished {
        if *remaining_number.lock().unwrap() == 0 {
            finished = true;
        }

        print!(
            "{} carpets remaining: {overlap_count}/{total_count} rows overlapped{}\r",
            remaining_number.lock().unwrap(),
            " ".repeat(50)
        );

        let mut data = vec![receiver.recv().unwrap()];
        while let Ok(temp) = receiver.try_recv() {
            data.push(temp);
        }

        let (
            (passed_carpets, passed_carpets_sudokus),
            (passed_carpets_sudokus_data, passed_carpets_sudokus_data_squares),
        ) = {
            let (
                (carpets, carpets_sudokus),
                (carpets_sudokus_data, carpets_sudokus_data_squares),
            ): ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)) = data.into_iter().unzip();
            (
                (
                    carpets,
                    carpets_sudokus.into_iter().flatten().collect::<Vec<_>>(),
                ),
                (
                    carpets_sudokus_data
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                    carpets_sudokus_data_squares
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                ),
            )
        };

        let total_rows = passed_carpets.len()
            + passed_carpets_sudokus.len()
            + passed_carpets_sudokus_data.len()
            + passed_carpets_sudokus_data_squares.len();

        let (just_inserted_sudokus, just_inserted_squares) = database
            .insert_ignore_multiple_canonical_sudokus(
                passed_carpets_sudokus_data,
                passed_carpets_sudokus_data_squares,
            )
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT FILLED CARPET IN DATABSE: {err}"));
        let (just_inserted_carpets, just_inserted_carpets_sudokus) = database
            .insert_ignore_multiple_canonical_carpets(passed_carpets, passed_carpets_sudokus)
            .unwrap_or_else(|err| panic!("ERROR COULDN'T INSERT FILLED CARPET IN DATABSE: {err}"));

        total_count += total_rows as u128;
        overlap_count += (total_rows
            - just_inserted_sudokus
            - just_inserted_squares
            - just_inserted_carpets
            - just_inserted_carpets_sudokus) as u128;
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}

fn carpet_games(max_number: usize) {
    let database = Arc::new(Mutex::new(Database::connect().unwrap()));
    let mut join_handle: Option<thread::JoinHandle<bool>> = None;

    let mut remaining_number = max_number;
    while remaining_number > 0 {
        println!("\n\n\n{remaining_number} filled carpets remaining");

        let mut patterns = CarpetPattern::iter().collect::<Vec<_>>();
        patterns.sort_by(|a, b| {
            a.get_n_sudokus().cmp(&b.get_n_sudokus()).then(
                a.get_carpet_links(3)
                    .len()
                    .cmp(&b.get_carpet_links(3).len()),
            )
        });
        for pattern in patterns.into_iter().cycle() {
            println!("\n{}: ", pattern);

            let filled = CarpetSudoku::generate_full(3, pattern);

            if let Some(my_join_handle) = join_handle {
                let no_problem = my_join_handle.join().unwrap();
                if !no_problem {
                    return;
                }
            }

            let thread_database = database.clone();
            let thread_filled = filled.clone();
            join_handle = Some(thread::spawn(move || {
                let (db_filled, db_filled_sudokus) = thread_filled.db_to_filled().unwrap();
                let (db_sudokus_data, db_sudokus_data_square): (Vec<_>, Vec<_>) = thread_filled
                    .db_sudokus_to_filled()
                    .unwrap()
                    .into_iter()
                    .unzip();

                if thread_filled.get_pattern() == CarpetPattern::Simple {
                    if let Ok((db_sudoku, db_squares)) =
                        thread_filled.get_sudokus()[0].filled_to_db()
                    {
                        let _ = thread_database
                            .lock()
                            .unwrap()
                            .insert_canonical_sudoku(true, db_sudoku, db_squares);
                    }
                }

                if let Err(err) = thread_database
                    .lock()
                    .unwrap()
                    .insert_ignore_multiple_canonical_sudokus(
                        db_sudokus_data,
                        db_sudokus_data_square
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                    )
                {
                    eprintln!("\nERROR :\n{thread_filled}\nCOULDN'T INSERT THIS FILLED CARPET'S SUDOKUS: {err}\n");
                    return false;
                }

                if let Err(err) = thread_database.lock().unwrap().insert_canonical_carpet(
                    true,
                    db_filled,
                    db_filled_sudokus,
                ) {
                    eprintln!(
                        "\nERROR :\n{thread_filled}\nCOULDN'T INSERT THIS FILLED CARPET: {err}\n"
                    );
                    return false;
                }

                true
            }));

            for difficulty in SudokuDifficulty::iter() {
                print!("{}: \r", difficulty);
                stdout().flush().unwrap();

                let game = filled.generate_from(difficulty).unwrap();

                if let Some(my_join_handle) = join_handle {
                    let no_problem = my_join_handle.join().unwrap();
                    if !no_problem {
                        return;
                    }
                }

                let database = Arc::clone(&database);
                join_handle = Some(thread::spawn(move || {
                    if game.get_pattern() == CarpetPattern::Simple {
                        if let Ok(sudoku_game_db) = game.get_sudokus()[0].game_to_db() {
                            let _ = database
                                .lock()
                                .unwrap()
                                .insert_canonical_sudoku_game(true, sudoku_game_db);
                        }
                    }

                    if let Err(err) = database
                        .lock()
                        .unwrap()
                        .insert_canonical_carpet_game(game.db_to_game())
                    {
                        eprintln!("\nERROR :\n{game}\nCOULDN'T INSERT THIS CARPET GAME: {err}\n");
                        return false;
                    }

                    true
                }));
            }
        }
        remaining_number -= 1;
    }
}
