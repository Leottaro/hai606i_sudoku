use std::{
    fs,
    io::{stdin},
};

use diesel::{
    connection::DefaultLoadingMode, dsl::min, BoolExpressionMethods, Connection, ExpressionMethods,
    JoinOnDsl, PgConnection, PgExpressionMethods, QueryDsl, RunQueryDsl,
};

use crate::{carpet_sudoku::CarpetSudoku, simple_sudoku::Sudoku as SimpleSudoku};

use super::{
    schema::{
        canonical_carpet_games::dsl::*, canonical_carpet_sudokus::dsl::*,
        canonical_carpets::dsl::*, canonical_sudoku_games::dsl::*,
        canonical_sudoku_squares::dsl::*, canonical_sudokus::dsl::*,
    },
    *,
};

define_sql_function! {
    fn random() -> Text;
}

impl Database {
    pub fn connect() -> Option<Self> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
            eprintln!("Couldn't get DATABASE_URL environment variable: {}", err);
			
			println!("Please enter the mandatory DATABASE_URL (postgresql://<USER>:<USER_PASSWORD>@<DB_IP>/database) :");
			let mut database_url = String::new();
			stdin().read_line(&mut database_url).unwrap();
			database_url = database_url.trim().to_string();

			fs::write(".env", format!("DATABASE_URL={database_url}")).unwrap();
			database_url
        });

        let connection = PgConnection::establish(&database_url);
        if let Err(error) = connection {
            eprintln!("Error connecting to db at {database_url}: {}", error);
            None
        } else {
            Some(Self {
                connection: connection.unwrap(),
            })
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   GET N   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_n_canonical_sudokus(
        &mut self,
        n: i64,
    ) -> Result<Vec<DBCanonicalSudoku>, diesel::result::Error> {
        canonical_sudokus
            .order(random())
            .limit(n)
            .get_results::<DBCanonicalSudoku>(&mut self.connection)
    }

    pub fn get_n_canonical_sudoku_games(
        &mut self,
        n: i64,
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        canonical_sudoku_games
            .order(random())
            .limit(n)
            .get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    pub fn get_n_canonical_carpets(
        &mut self,
        canonical_number: i64,
        n: i16,
        (pattern, pattern_size): (i16, Option<i16>),
    ) -> Result<Vec<DBFilledCarpetData>, diesel::result::Error> {
        let mut db_carpets = canonical_carpets
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size)),
            )
            .order(random())
            .limit(canonical_number)
            .get_results::<DBCanonicalCarpet>(&mut self.connection)?;
        db_carpets.sort_by(|a, b| a.carpet_filled_board_hash.cmp(&b.carpet_filled_board_hash));

        let filled_board_hashes = db_carpets
            .iter()
            .map(|db_carpet| db_carpet.carpet_filled_board_hash)
            .collect::<Vec<_>>();

        let db_carpets_data = canonical_carpet_sudokus
            .left_join(canonical_sudokus.on(filled_board_hash.eq(carpet_sudoku_filled_board_hash)))
            .filter(carpet_sudoku_carpet_filled_board_hash.eq_any(filled_board_hashes))
            .order_by((carpet_sudoku_carpet_filled_board_hash, carpet_sudoku_i))
            .load_iter::<(DBCanonicalCarpetSudoku, Option<DBCanonicalSudoku>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((db_carpet_sudoku, Some(db_sudoku))) = a {
                    Some((db_carpet_sudoku, db_sudoku))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if (db_carpets_data.len() as i16)
            != db_carpets
                .iter()
                .map(|db_carpet| db_carpet.carpet_sudoku_number)
                .sum()
        {
            Err(diesel::result::Error::NotFound)
        } else {
            let mut db_carpets_data = db_carpets_data.into_iter();
            Ok(db_carpets
                .into_iter()
                .map(|db_carpet| {
                    let (db_carpet_sudokus, db_sudokus): (Vec<_>, Vec<_>) = (0..db_carpet
                        .carpet_sudoku_number)
                        .map(|_| db_carpets_data.next().unwrap())
                        .unzip();
                    (db_carpet, db_carpet_sudokus, db_sudokus)
                })
                .collect())
        }
    }

    pub fn get_n_canonical_carpet_games(
        &mut self,
        canonical_number: i64,
        n: i16,
        (pattern, pattern_size): (i16, Option<i16>),
        difficulty: i16,
    ) -> Result<Vec<DBGameCarpetData>, diesel::result::Error> {
        let mut db_carpet_games = canonical_carpet_games
            .left_join(
                canonical_carpets
                    .on(carpet_game_carpet_filled_board_hash.eq(carpet_filled_board_hash)),
            )
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_game_difficulty.eq(difficulty))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size)),
            )
            .order(random())
            .limit(canonical_number)
            .load_iter::<(DBCanonicalCarpetGame, Option<DBCanonicalCarpet>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((a, Some(b))) = a {
                    Some((b, a))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        db_carpet_games.sort_by(|a, b| {
            a.0.carpet_filled_board_hash
                .cmp(&b.0.carpet_filled_board_hash)
        });

        let filled_board_hashes = db_carpet_games
            .iter()
            .map(|(db_carpet, _game)| db_carpet.carpet_filled_board_hash)
            .collect::<Vec<_>>();

        let db_carpets_data = canonical_carpet_sudokus
            .left_join(canonical_sudokus.on(filled_board_hash.eq(carpet_sudoku_filled_board_hash)))
            .filter(carpet_sudoku_carpet_filled_board_hash.eq_any(filled_board_hashes))
            .order_by((carpet_sudoku_carpet_filled_board_hash, carpet_sudoku_i))
            .load_iter::<(DBCanonicalCarpetSudoku, Option<DBCanonicalSudoku>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((db_carpet_sudoku, Some(db_sudoku))) = a {
                    Some((db_carpet_sudoku, db_sudoku))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if (db_carpets_data.len() as i16)
            != db_carpet_games
                .iter()
                .map(|(db_carpet, _db_game)| db_carpet.carpet_sudoku_number)
                .sum()
        {
            Err(diesel::result::Error::NotFound)
        } else {
            let mut db_carpets_data = db_carpets_data.into_iter();
            Ok(db_carpet_games
                .into_iter()
                .map(|(db_carpet, db_game)| {
                    let (db_carpet_sudokus, db_sudokus): (Vec<_>, Vec<_>) = (0..db_carpet
                        .carpet_sudoku_number)
                        .map(|_| db_carpets_data.next().unwrap())
                        .unzip();
                    (db_game.into(), db_carpet, db_carpet_sudokus, db_sudokus)
                })
                .collect())
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   INSERT   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_canonical_sudoku(
        &mut self,
        ignore: bool,
        sudoku: DBCanonicalSudoku,
        squares: Vec<DBCanonicalSudokuSquare>,
    ) -> Result<usize, diesel::result::Error> {
        let query1 = diesel::insert_into(canonical_sudokus).values(&sudoku);
        let query2 = diesel::insert_into(canonical_sudoku_squares).values(&squares);
        if ignore {
            let query1 = query1.on_conflict_do_nothing();
            let query2 = query2.on_conflict_do_nothing();
            Ok(query1.execute(&mut self.connection)? + query2.execute(&mut self.connection)?)
        } else {
            Ok(query1.execute(&mut self.connection)? + query2.execute(&mut self.connection)?)
        }
    }

    pub fn insert_canonical_sudoku_game(
        &mut self,
        ignore: bool,
        sudoku: DBNewCanonicalSudokuGame,
    ) -> Result<DBCanonicalSudokuGame, diesel::result::Error> {
        let query = diesel::insert_into(canonical_sudoku_games).values(&sudoku);
        if ignore {
            query
                .on_conflict_do_nothing()
                .get_result(&mut self.connection)
        } else {
            query.get_result(&mut self.connection)
        }
    }

    pub fn insert_canonical_carpet(
        &mut self,
        ignore: bool,
        carpet: DBCanonicalCarpet,
        carpet_sudokus: Vec<DBCanonicalCarpetSudoku>,
    ) -> Result<usize, diesel::result::Error> {
        let query1 = diesel::insert_into(canonical_carpets).values(&carpet);
        if ignore {
            query1
                .on_conflict_do_nothing()
                .execute(&mut self.connection)?;
        } else {
            query1.execute(&mut self.connection)?;
        }

        let query2 = diesel::insert_into(canonical_carpet_sudokus).values(&carpet_sudokus);
        let inserted_sudokus_count = if ignore {
            query2
                .on_conflict_do_nothing()
                .execute(&mut self.connection)?
        } else {
            query2.execute(&mut self.connection)?
        };
        Ok(1 + inserted_sudokus_count)
    }

    pub fn insert_canonical_carpet_game(
        &mut self,
        game: DBNewCanonicalCarpetGame,
    ) -> Result<DBCanonicalCarpetGame, diesel::result::Error> {
        diesel::insert_into(canonical_carpet_games)
            .values(&game)
            .get_result::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////   INSERT MULTIPLE   //////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_multiple_canonical_sudokus(
        &mut self,
        sudokus: Vec<DBCanonicalSudoku>,
        squares: Vec<DBCanonicalSudokuSquare>,
    ) -> Result<(usize, usize), diesel::result::Error> {
        let inserted_sudokus = diesel::copy_from(canonical_sudokus)
            .from_insertable(&sudokus)
            .execute(&mut self.connection)?;
        let inserted_squares = diesel::copy_from(canonical_sudoku_squares)
            .from_insertable(&squares)
            .execute(&mut self.connection)?;
        Ok((inserted_sudokus, inserted_squares))
    }

    pub fn insert_multiple_canonical_sudoku_game(
        &mut self,
        sudokus: Vec<DBNewCanonicalSudokuGame>,
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        diesel::insert_into(canonical_sudoku_games)
            .values(sudokus)
            .get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    pub fn insert_multiple_canonical_carpets(
        &mut self,
        carpets: Vec<DBCanonicalCarpet>,
        carpets_sudokus: Vec<DBCanonicalCarpetSudoku>,
    ) -> Result<(usize, usize), diesel::result::Error> {
        let inserted_carpets = diesel::copy_from(canonical_carpets)
            .from_insertable(&carpets)
            .execute(&mut self.connection)?;
        let inserted_carpet_sudokus = diesel::copy_from(canonical_carpet_sudokus)
            .from_insertable(&carpets_sudokus)
            .execute(&mut self.connection)?;
        Ok((inserted_carpets, inserted_carpet_sudokus))
    }

    pub fn insert_multiple_canonical_carpet_games(
        &mut self,
        games: Vec<DBNewCanonicalCarpetGame>,
    ) -> Result<Vec<DBCanonicalCarpetGame>, diesel::result::Error> {
        diesel::insert_into(canonical_carpet_games)
            .values(games)
            .get_results::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////   INSERT IGNORE MULTIPLE   ///////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn insert_ignore_multiple_canonical_sudokus(
        &mut self,
        sudokus: Vec<DBCanonicalSudoku>,
        squares: Vec<DBCanonicalSudokuSquare>,
    ) -> Result<(usize, usize), diesel::result::Error> {
        let inserted_sudokus = diesel::insert_into(canonical_sudokus)
            .values(&sudokus)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)?;
        let inserted_squares = diesel::insert_into(canonical_sudoku_squares)
            .values(&squares)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)?;
        Ok((inserted_sudokus, inserted_squares))
    }

    pub fn insert_ignore_multiple_canonical_sudoku_game(
        &mut self,
        sudokus: Vec<DBNewCanonicalSudokuGame>,
    ) -> Result<Vec<DBCanonicalSudokuGame>, diesel::result::Error> {
        diesel::insert_into(canonical_sudoku_games)
            .values(sudokus)
            .on_conflict_do_nothing()
            .get_results::<DBCanonicalSudokuGame>(&mut self.connection)
    }

    pub fn insert_ignore_multiple_canonical_carpets(
        &mut self,
        carpets: Vec<DBCanonicalCarpet>,
        carpets_sudokus: Vec<DBCanonicalCarpetSudoku>,
    ) -> Result<(usize, usize), diesel::result::Error> {
        let inserted_carpets = diesel::insert_into(canonical_carpets)
            .values(&carpets)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)?;
        let inserted_carpet_sudokus = diesel::insert_into(canonical_carpet_sudokus)
            .values(&carpets_sudokus)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)?;
        Ok((inserted_carpets, inserted_carpet_sudokus))
    }

    pub fn insert_ignore_multiple_canonical_carpet_games(
        &mut self,
        games: Vec<DBNewCanonicalCarpetGame>,
    ) -> Result<Vec<DBCanonicalCarpetGame>, diesel::result::Error> {
        diesel::insert_into(canonical_carpet_games)
            .values(games)
            .on_conflict_do_nothing()
            .get_results::<DBCanonicalCarpetGame>(&mut self.connection)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   GET RANDOM   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_random_canonical_sudokus(
        &mut self,
        n: u8,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        canonical_sudokus
            .filter(sudoku_n.eq(n as i16))
            .order(random())
            .limit(1)
            .get_result::<DBCanonicalSudoku>(&mut self.connection)
            .map(SimpleSudoku::db_from_filled)
    }

    pub fn get_random_canonical_sudoku_game(
        &mut self,
        n: i16,
        difficulty: i16,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let (game_info, filled_info) = canonical_sudoku_games
            .left_join(canonical_sudokus.on(filled_board_hash.eq(sudoku_game_filled_board_hash)))
            .filter(sudoku_n.eq(n).and(sudoku_game_difficulty.eq(difficulty)))
            .order(random())
            .limit(1)
            .get_result::<(DBCanonicalSudokuGame, Option<DBCanonicalSudoku>)>(
                &mut self.connection,
            )?;
        Ok(SimpleSudoku::db_from_game(game_info, filled_info.unwrap()))
    }

    pub fn get_random_canonical_carpet(
        &mut self,
        n: i16,
        (pattern, pattern_size): (i16, Option<i16>),
    ) -> Result<CarpetSudoku, diesel::result::Error> {
        let db_carpet = canonical_carpets
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size)),
            )
            .order(random())
            .limit(1)
            .get_result::<DBCanonicalCarpet>(&mut self.connection)?;

        let (db_carpet_sudokus, db_sudokus): (Vec<_>, Vec<_>) = canonical_carpet_sudokus
            .left_join(canonical_sudokus.on(filled_board_hash.eq(carpet_sudoku_filled_board_hash)))
            .filter(carpet_sudoku_carpet_filled_board_hash.eq(db_carpet.carpet_filled_board_hash))
            .order_by(carpet_sudoku_i)
            .load_iter::<(DBCanonicalCarpetSudoku, Option<DBCanonicalSudoku>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((db_carpet_sudoku, Some(db_sudoku))) = a {
                    Some((db_carpet_sudoku, db_sudoku))
                } else {
                    None
                }
            })
            .unzip();

        if db_carpet_sudokus.len() != db_sudokus.len()
            || db_sudokus.len() != (db_carpet.carpet_sudoku_number as usize)
        {
            Err(diesel::result::Error::NotFound)
        } else {
            Ok(CarpetSudoku::db_from_filled(
                db_carpet,
                db_carpet_sudokus,
                db_sudokus,
            ))
        }
    }

    pub fn get_random_canonical_carpet_game(
        &mut self,
        n: i16,
        (pattern, pattern_size): (i16, Option<i16>),
        difficulty: i16,
    ) -> Result<CarpetSudoku, diesel::result::Error> {
        let (game_info, db_carpet) = canonical_carpet_games
            .left_join(
                canonical_carpets
                    .on(carpet_game_carpet_filled_board_hash.eq(carpet_filled_board_hash)),
            )
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_game_difficulty.eq(difficulty))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size)),
            )
            .order(random())
            .get_result::<(DBCanonicalCarpetGame, Option<DBCanonicalCarpet>)>(
                &mut self.connection,
            )?;
        let db_carpet = db_carpet.unwrap();

        let (db_carpet_sudokus, db_sudokus): (Vec<_>, Vec<_>) = canonical_carpet_sudokus
            .left_join(canonical_sudokus.on(filled_board_hash.eq(carpet_sudoku_filled_board_hash)))
            .filter(carpet_sudoku_carpet_filled_board_hash.eq(db_carpet.carpet_filled_board_hash))
            .order_by(carpet_sudoku_i)
            .load_iter::<(DBCanonicalCarpetSudoku, Option<DBCanonicalSudoku>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((db_carpet_sudoku, Some(db_sudoku))) = a {
                    Some((db_carpet_sudoku, db_sudoku))
                } else {
                    None
                }
            })
            .unzip();

        if db_carpet_sudokus.len() != db_sudokus.len()
            || db_sudokus.len() != (db_carpet.carpet_sudoku_number as usize)
        {
            Err(diesel::result::Error::NotFound)
        } else {
            Ok(CarpetSudoku::db_from_game(
                game_info,
                db_carpet,
                db_carpet_sudokus,
                db_sudokus,
            ))
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   GET MINIMAL   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn get_random_minimal_canonical_sudoku_game(
        &mut self,
        n: i16,
        difficulty: i16,
    ) -> Result<SimpleSudoku, diesel::result::Error> {
        let min_filled_cells_count = canonical_sudoku_games
            .left_join(canonical_sudokus.on(filled_board_hash.eq(sudoku_game_filled_board_hash)))
            .filter(sudoku_n.eq(n).and(sudoku_game_difficulty.eq(difficulty)))
            .select(min(sudoku_game_filled_cells_count))
            .get_result::<Option<i16>>(&mut self.connection)?;
        if min_filled_cells_count.is_none() {
            return Err(diesel::result::Error::NotFound);
        }
        let min_filled_cells_count = min_filled_cells_count.unwrap();

        let (game_info, filled_info) = canonical_sudoku_games
            .left_join(canonical_sudokus.on(filled_board_hash.eq(sudoku_game_filled_board_hash)))
            .filter(
                sudoku_n
                    .eq(n)
                    .and(sudoku_game_difficulty.eq(difficulty))
                    .and(sudoku_game_filled_cells_count.eq(min_filled_cells_count)),
            )
            .order(random())
            .limit(1)
            .get_result::<(DBCanonicalSudokuGame, Option<DBCanonicalSudoku>)>(
                &mut self.connection,
            )?;
        Ok(SimpleSudoku::db_from_game(game_info, filled_info.unwrap()))
    }

    pub fn get_random_minimal_canonical_carpet_game(
        &mut self,
        n: i16,
        (pattern, pattern_size): (i16, Option<i16>),
        difficulty: i16,
    ) -> Result<CarpetSudoku, diesel::result::Error> {
        let min_filled_cells_count = canonical_carpet_games
            .left_join(
                canonical_carpets
                    .on(carpet_game_carpet_filled_board_hash.eq(carpet_filled_board_hash)),
            )
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_game_difficulty.eq(difficulty))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size)),
            )
            .select(min(carpet_game_filled_cells_count))
            .get_result::<Option<i16>>(&mut self.connection)?;
        if min_filled_cells_count.is_none() {
            return Err(diesel::result::Error::NotFound);
        }
        let min_filled_cells_count = min_filled_cells_count.unwrap();

        let (game_info, db_carpet) = canonical_carpet_games
            .left_join(
                canonical_carpets
                    .on(carpet_game_carpet_filled_board_hash.eq(carpet_filled_board_hash)),
            )
            .filter(
                carpet_n
                    .eq(n)
                    .and(carpet_pattern.eq(pattern))
                    .and(carpet_game_difficulty.eq(difficulty))
                    .and(carpet_pattern_size.is_not_distinct_from(pattern_size))
                    .and(carpet_game_filled_cells_count.eq(min_filled_cells_count)),
            )
            .order(random())
            .get_result::<(DBCanonicalCarpetGame, Option<DBCanonicalCarpet>)>(
                &mut self.connection,
            )?;
        let db_carpet = db_carpet.unwrap();

        let (db_carpet_sudokus, db_sudokus): (Vec<_>, Vec<_>) = canonical_carpet_sudokus
            .left_join(canonical_sudokus.on(filled_board_hash.eq(carpet_sudoku_filled_board_hash)))
            .filter(carpet_sudoku_carpet_filled_board_hash.eq(db_carpet.carpet_filled_board_hash))
            .order_by(carpet_sudoku_i)
            .load_iter::<(DBCanonicalCarpetSudoku, Option<DBCanonicalSudoku>), DefaultLoadingMode>(
                &mut self.connection,
            )?
            .filter_map(|a| {
                if let Ok((db_carpet_sudoku, Some(db_sudoku))) = a {
                    Some((db_carpet_sudoku, db_sudoku))
                } else {
                    None
                }
            })
            .unzip();

        if db_carpet_sudokus.len() != db_sudokus.len()
            || db_sudokus.len() != (db_carpet.carpet_sudoku_number as usize)
        {
            Err(diesel::result::Error::NotFound)
        } else {
            Ok(CarpetSudoku::db_from_game(
                game_info,
                db_carpet,
                db_carpet_sudokus,
                db_sudokus,
            ))
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////   OTHER   ///////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // TODO:
    // pub fn construct_canonical_carpet(
    //     &mut self,
    //     n: i16,
    //     pattern: CarpetPattern,
    // ) -> Result<DBFilledCarpetData, SudokuError> {
    //     let n_sudokus = pattern.get_n_sudokus();
    //     let mut truc = canonical_sudokus
    //         .left_join(canonical_sudoku_squares.on(square_filled_board_hash.eq(filled_board_hash)));
    //     let raw_links = pattern.get_raw_links(n as usize);
    //     todo!()
    // }
}
