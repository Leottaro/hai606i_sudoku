use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty, SudokuError, SudokuGroups};

use super::{CarpetLinks, CarpetPattern, CarpetSudoku};
use rand::{rng, seq::SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

impl CarpetSudoku {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   GETTERS   //////////////////////////////////////////////////////////
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
        group: SudokuGroups,
    ) -> HashSet<Coords> {
        self.sudokus[sudoku_id].get_cell_group(x, y, group)
    }

    pub fn get_global_cell_group(
        &self,
        sudoku_id: usize,
        x: usize,
        y: usize,
        group: SudokuGroups,
    ) -> HashSet<(usize, usize, usize)> {
        self.get_twin_cells(sudoku_id, x, y)
            .into_iter()
            .flat_map(|(i, x, y)| {
                self.sudokus[i]
                    .get_cell_group(x, y, group)
                    .into_iter()
                    .map(move |(x, y)| (i, x, y))
            })
            .collect()
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

    pub fn get_twin_cells(
        &self,
        sudoku_id: usize,
        x: usize,
        y: usize,
    ) -> Vec<(usize, usize, usize)> {
        let dx = x % self.n;
        let dy = y % self.n;
        let x0 = x - dx;
        let y0: usize = y - dy;
        let square_id = y0 + x0 / self.n;
        let mut twins = vec![(sudoku_id, x, y)];

        if self.links.contains_key(&sudoku_id) {
            for &(square1, sudoku2, square2) in self.links.get(&sudoku_id).unwrap() {
                if square_id != square1 {
                    continue;
                }
                let y2 = (square2 / self.n) * self.n;
                let x2 = (square2 % self.n) * self.n;
                twins.push((sudoku2, x2 + dx, y2 + dy));
            }
        }

        twins
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   CREATION   /////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn new(n: usize, pattern: CarpetPattern) -> Self {
        let pattern = match pattern {
            CarpetPattern::Diagonal(1) | CarpetPattern::Carpet(1) => CarpetPattern::Simple,
            CarpetPattern::Diagonal(2) => CarpetPattern::Double,
			CarpetPattern::Custom(_) => panic!("Can't call CarpetSudoku::new() with a CarpetPattern::Custom pattern ! Try using CarpetSudoku::new_custom() instead."),
            pattern => pattern,
        };
        let n_sudokus = pattern.get_n_sudokus();
        let sudokus = (0..n_sudokus).map(|_| Sudoku::new(n)).collect();
        let links: CarpetLinks = pattern.get_carpet_links(n);

        Self {
            n,
            n2: n * n,
            pattern,
            difficulty: SudokuDifficulty::Unknown,
            sudokus,
            links,
            filled_board_hash: 0,
            is_canonical: false,
        }
    }

    pub fn new_custom(n: usize, sudokus: Vec<Sudoku>, links: CarpetLinks) -> Self {
        Self {
            n,
            n2: n * n,
            pattern: CarpetPattern::Custom(links.len()),
            difficulty: SudokuDifficulty::Unknown,
            sudokus,
            links,
            filled_board_hash: 0,
            is_canonical: false,
        }
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
        let twin_cells = self.get_twin_cells(sudoku_id, x, y);
        let is_in_link = twin_cells.len() > 1;
        for (sudoku2, x2, y2) in twin_cells {
            self.sudokus[sudoku2].set_value(x2, y2, value)?;
        }

        if is_in_link {
            return Ok(());
        }

        for (x, y) in self.sudokus[sudoku_id].get_cell_group(x, y, SudokuGroups::All) {
            for (sudoku2, x2, y2) in self.get_twin_cells(sudoku_id, x, y) {
                self.sudokus[sudoku2]
                    .get_cell_possibilities_mut(x2, y2)
                    .remove(&value);
            }
        }

        if self.is_canonical && self.filled_board_hash == 0 && self.is_filled() {
            self.filled_board_hash = {
                let mut hasher = DefaultHasher::new();
                for sudoku_i in 0..self.sudokus.len() {
                    for y in 0..self.n2 {
                        for x in 0..self.n2 {
                            self.sudokus[sudoku_i]
                                .get_cell_value(x, y)
                                .hash(&mut hasher);
                        }
                    }
                }
                hasher.finish()
            };
        }

        Ok(())
    }

    pub fn remove_value(
        &mut self,
        sudoku_id: usize,
        x: usize,
        y: usize,
    ) -> Result<usize, SudokuError> {
        let value = self.get_cell_value(sudoku_id, x, y);
        let twin_cells = self.get_twin_cells(sudoku_id, x, y);
        let is_in_link = twin_cells.len() > 1;
        for (sudoku2, x2, y2) in twin_cells {
            self.sudokus[sudoku2].remove_value(x2, y2)?;
        }

        if is_in_link {
            return Ok(value);
        }

        for (x, y) in self.sudokus[sudoku_id].get_cell_group(x, y, SudokuGroups::All) {
            for (sudoku2, x2, y2) in self.get_twin_cells(sudoku_id, x, y) {
                if self.sudokus[sudoku2]
                    .get_cell_group(x2, y2, SudokuGroups::All)
                    .into_iter()
                    .any(|(x3, y3)| self.sudokus[sudoku2].get_cell_value(x3, y3) == value)
                {
                    self.sudokus[sudoku_id]
                        .get_cell_possibilities_mut(x, y)
                        .remove(&value);
                } else {
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

    pub fn rule_solve_until(
        &mut self,
        rule_solve_result: (bool, bool),
        max_difficulty: Option<SudokuDifficulty>,
    ) -> bool {
        self.difficulty = SudokuDifficulty::Unknown;
        let mut did_anything = false;
        while let Ok(result) = self.rule_solve(max_difficulty) {
            if result == rule_solve_result || result == (false, false) {
                break;
            }
            did_anything = true;
        }
        did_anything
    }

    pub fn backtrack_solve(&mut self) -> bool {
        self._backtrack_solve(
            (0..self.sudokus.len() * self.n2 * self.n2)
                .map(|i| {
                    let sudoku_id = i / (self.n2 * self.n2);
                    let cell_i = i - sudoku_id * self.n2 * self.n2;
                    let y: usize = cell_i / self.n2;
                    let x: usize = cell_i % self.n2;
                    (sudoku_id, x, y)
                })
                .collect::<Vec<_>>(),
        )
    }

    fn _backtrack_solve(&mut self, mut empty_cells: Vec<(usize, usize, usize)>) -> bool {
        empty_cells.sort_by(|&a, &b| {
            self.sudokus[a.0]
                .get_cell_possibilities(a.1, a.2)
                .len()
                .cmp(&self.sudokus[b.0].get_cell_possibilities(b.1, b.2).len())
        });

        let mut i = 0;
        while i < empty_cells.len() {
            let (sudoku_id, x, y) = empty_cells[i];
            if !self.get_cell_possibilities(sudoku_id, x, y).is_empty() {
                break;
            }
            if self.get_cell_value(sudoku_id, x, y) == 0 {
                return false;
            }
            i += 1;
        }
        empty_cells.drain(0..i);

        if empty_cells.is_empty() {
            return true;
        }

        let (sudoku_id, x, y) = empty_cells[0];
        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut rng());
        for value in possibilities {
            match self.set_value(sudoku_id, x, y, value) {
                Ok(()) => (),
                Err(_) => {
                    let _ = self.remove_value(sudoku_id, x, y);
                    continue;
                }
            }

            if self._backtrack_solve(empty_cells.clone()) {
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
    ///////////////////////////////////////////////////////   CANONIZATION   ///////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn randomize(&mut self) -> Result<(), SudokuError> {
        if !self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "randomize() when this carpet is already randomized: {self}"
            )));
        }

        if self.links.is_empty() {
            for sudoku in self.sudokus.iter_mut() {
                sudoku.randomize(None, None, true)?;
            }
            return Ok(());
        }

        self.sudokus[0].randomize(None, None, false)?;
        let mut rows_swaps = vec![Default::default(); self.sudokus.len()];
        let values_swap = self.sudokus[0].get_values_swap();
        rows_swaps[0] = self.sudokus[0].get_rows_swap();

        // fill the rows_swaps linked
        for &(square1, sudoku2, square2) in self.links.get(&0).unwrap() {
            let y1 = square1 - square1 % self.n;
            let y2 = square2 - square2 % self.n;
            for dy in 0..self.n {
                let (to_y1, from_y1) = rows_swaps[0][&(y1 + dy)];
                rows_swaps[sudoku2].insert(y2 + dy, (y2 + to_y1 % self.n, y2 + from_y1 % self.n));
            }
        }

        for (sudoku1, sudoku) in self.sudokus.iter_mut().enumerate().skip(1) {
            // complete the rows_swap
            for y0 in (0..self.n2).step_by(self.n) {
                if rows_swaps[sudoku1].contains_key(&y0) {
                    continue;
                }
                let mut to_ys = {
                    let mut dxs = (0..self.n).collect::<Vec<_>>();
                    dxs.shuffle(&mut rng());
                    dxs.into_iter()
                };
                for y in y0..y0 + self.n {
                    let to_y = y0 + to_ys.next().unwrap();
                    rows_swaps[sudoku1]
                        .entry(y)
                        .and_modify(|(a, _)| *a = to_y)
                        .or_insert((to_y, 0));

                    rows_swaps[sudoku1]
                        .entry(to_y)
                        .and_modify(|(_, b)| *b = y)
                        .or_insert((0, y));
                }
            }

            // fill the rows_swaps linked
            for &(square1, sudoku2, square2) in self.links.get(&sudoku1).unwrap() {
                let y1 = square1 - square1 % self.n;
                let y2 = square2 - square2 % self.n;
                for dy in 0..self.n {
                    let (to_y1, from_y1) = rows_swaps[sudoku1][&(y1 + dy)];
                    rows_swaps[sudoku2]
                        .insert(y2 + dy, (y2 + to_y1 % self.n, y2 + from_y1 % self.n));
                }
            }

            sudoku.randomize(
                Some(rows_swaps[sudoku1].clone()),
                Some(values_swap.clone()),
                false,
            )?;
        }

        self.is_canonical = false;
        Ok(())
    }

    pub fn canonize(&mut self) -> Result<(), SudokuError> {
        if !self.is_filled() {
            return Err(SudokuError::InvalidState(format!(
                "canonize() when this carpet isn't filled: {self}"
            )));
        }
        if self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "canonize() when this carpet is already canonical: {self}"
            )));
        }

        for sudoku in self.sudokus.iter_mut() {
            sudoku.canonize()?;
        }

        self.is_canonical = true;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////   UTILITY   //////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn is_filled(&self) -> bool {
        self.sudokus.iter().all(|sudoku| sudoku.is_filled())
    }

    pub fn is_unique(&mut self) -> bool {
        self.count_solutions(Some(2)) == 1
    }

    pub fn count_solutions(&self, max_solutions: Option<usize>) -> usize {
        self.clone()._count_solutions(
            (0..self.sudokus.len() * self.n2 * self.n2)
                .filter_map(|i| {
                    let sudoku_id = i / (self.n2 * self.n2);
                    let cell_i = i - sudoku_id * self.n2 * self.n2;
                    let y: usize = cell_i / self.n2;
                    let x: usize = cell_i % self.n2;
                    if self.sudokus[sudoku_id].get_cell_value(x, y) == 0 {
                        Some((sudoku_id, x, y))
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
        mut empty_cells: Vec<(usize, usize, usize)>,
        max_solutions: Option<usize>,
    ) -> usize {
        empty_cells.sort_by(|&a, &b| {
            self.sudokus[a.0]
                .get_cell_possibilities(a.1, a.2)
                .len()
                .cmp(&self.sudokus[b.0].get_cell_possibilities(b.1, b.2).len())
        });

        let mut i = 0;
        while i < empty_cells.len() {
            let (sudoku_id, x, y) = empty_cells[i];
            if !self.get_cell_possibilities(sudoku_id, x, y).is_empty() {
                break;
            }
            if self.get_cell_value(sudoku_id, x, y) == 0 {
                return 0;
            }
            i += 1;
        }
        empty_cells.drain(0..i);

        if empty_cells.is_empty() {
            return 1;
        }

        let (sudoku_id, x, y) = empty_cells[0];
        let mut possibilities = self.sudokus[sudoku_id]
            .get_cell_possibilities(x, y)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        possibilities.shuffle(&mut rng());
        let mut sub_solutions = 0;
        for value in possibilities {
            match self.set_value(sudoku_id, x, y, value) {
                Ok(()) => (),
                Err(_) => {
                    let _ = self.remove_value(sudoku_id, x, y);
                    continue;
                }
            }

            sub_solutions += self._count_solutions(empty_cells.clone(), max_solutions);
            if let Some(max_solutions) = max_solutions {
                if sub_solutions >= max_solutions {
                    return sub_solutions;
                }
            }

            if let Err(err) = self.remove_value(sudoku_id, x, y) {
                eprintln!(
                    "ERRROR AFTER self.remove_value({sudoku_id}, {x}, {y}): {err}\nFOR CARPET:{self}"
                );
            }
        }

        sub_solutions
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////   DATABASE   //////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "database")]
use crate::database::{
    DBCanonicalCarpet, DBCanonicalCarpetSudoku, DBCanonicalSudoku, DBCanonicalSudokuSquare,
    DBNewCanonicalCarpetGame, Database,
};

#[cfg(feature = "database")]
impl CarpetSudoku {
    pub fn db_to_filled(
        &self,
    ) -> Result<(DBCanonicalCarpet, Vec<DBCanonicalCarpetSudoku>), SudokuError> {
        if !self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "db_to_filled() when this carpet isn't canonical: {self}"
            )));
        }
        if !self.is_filled() {
            return Err(
                SudokuError::WrongFunction(
                    format!(
                        "db_to_filled() when the carpet isn't filled. Try calling game_to_db() instead.\n{self}"
                    )
                )
            );
        }

        let (pattern, pattern_size) = self.pattern.to_db();
        let db_carpet = DBCanonicalCarpet {
            carpet_filled_board_hash: self.filled_board_hash.wrapping_sub(u64::MAX / 2 + 1) as i64,
            carpet_n: self.n as i16,
            carpet_sudoku_number: self.sudokus.len() as i16,
            carpet_pattern: pattern,
            carpet_pattern_size: pattern_size,
        };

        let db_carpet_sudokus = self
            .sudokus
            .iter()
            .enumerate()
            .map(|(i, sudoku)| DBCanonicalCarpetSudoku {
                carpet_sudoku_carpet_filled_board_hash: self
                    .filled_board_hash
                    .wrapping_sub(u64::MAX / 2 + 1)
                    as i64,
                carpet_sudoku_i: i as i16,
                carpet_sudoku_filled_board_hash: sudoku
                    .get_canonical_filled_board_hash()
                    .wrapping_sub(u64::MAX / 2 + 1)
                    as i64,
            })
            .collect::<Vec<_>>();

        Ok((db_carpet, db_carpet_sudokus))
    }

    pub fn db_sudokus_to_filled(
        &self,
    ) -> Result<Vec<(DBCanonicalSudoku, Vec<DBCanonicalSudokuSquare>)>, SudokuError> {
        let mut sudokus = Vec::new();
        for sudoku in self.sudokus.iter() {
            sudokus.push(sudoku.filled_to_db()?);
        }
        Ok(sudokus)
    }

    pub fn db_to_game(&self) -> DBNewCanonicalCarpetGame {
        let (filled_cells_count, filled_cells): (i16, Vec<u8>) = {
            let temp = (0..self.sudokus.len() * self.n2 * self.n2)
                .map(|i| {
                    let sudoku_id = i / (self.n2 * self.n2);
                    let cell_i = i - sudoku_id * self.n2 * self.n2;
                    let y = cell_i / self.n2;
                    let x = cell_i % self.n2;
                    self.sudokus[sudoku_id].get_cell_value(x, y) > 0
                })
                .collect::<Vec<_>>();
            (
                temp.iter().filter(|is_filled| **is_filled).count() as i16,
                temp.into_iter()
                    .map(|is_filled| is_filled as u8)
                    .collect::<Vec<_>>(),
            )
        };
        DBNewCanonicalCarpetGame {
            carpet_game_carpet_filled_board_hash: self
                .filled_board_hash
                .wrapping_sub(u64::MAX / 2 + 1)
                as i64,
            carpet_game_difficulty: self.difficulty as i16,
            carpet_game_filled_cells: filled_cells,
            carpet_game_filled_cells_count: filled_cells_count,
        }
    }

    pub fn db_from_filled(
        db_carpet: DBCanonicalCarpet,
        db_carpet_sudokus: Vec<DBCanonicalCarpetSudoku>,
        db_sudokus: Vec<DBCanonicalSudoku>,
    ) -> Self {
        let mut carpet = Self::new(
            db_carpet.carpet_n as usize,
            CarpetPattern::from_db(db_carpet.carpet_pattern, db_carpet.carpet_pattern_size),
        );
        carpet.filled_board_hash =
            (db_carpet.carpet_filled_board_hash as u64).wrapping_add(u64::MAX / 2 + 1);
        carpet.is_canonical = true;

        for carpet_sudoku in db_carpet_sudokus {
            let sudoku = db_sudokus
                .iter()
                .find(|sudoku| {
                    sudoku.filled_board_hash == carpet_sudoku.carpet_sudoku_filled_board_hash
                })
                .expect("Sudoku not found in db_sudokus");
            carpet.sudokus[carpet_sudoku.carpet_sudoku_i as usize] =
                Sudoku::db_from_filled(sudoku.clone());
        }

        carpet
    }

    pub fn db_from_game(
        game_info: impl Into<DBNewCanonicalCarpetGame>,
        db_carpet: DBCanonicalCarpet,
        db_carpet_sudokus: Vec<DBCanonicalCarpetSudoku>,
        db_sudokus: Vec<DBCanonicalSudoku>,
    ) -> Self {
        let game_info = game_info.into();
        let mut carpet = Self::db_from_filled(db_carpet, db_carpet_sudokus, db_sudokus);
        carpet.difficulty = SudokuDifficulty::from(game_info.carpet_game_difficulty);

        for (i, is_filled) in game_info.carpet_game_filled_cells.into_iter().enumerate() {
            if is_filled != 0 {
                continue;
            }
            let sudoku_id = i / (carpet.n2 * carpet.n2);
            let cell_i = i - sudoku_id * carpet.n2 * carpet.n2;
            let y = cell_i / carpet.n2;
            let x = cell_i % carpet.n2;
            carpet.sudokus[sudoku_id].remove_value(x, y).unwrap();
        }

        carpet.update_link().unwrap();
        carpet
    }

    pub fn load_filled_from_db(database: &mut Database, n: usize, pattern: CarpetPattern) -> Self {
        database
            .get_random_canonical_carpet(n as i16, pattern.to_db())
            .unwrap()
    }

    pub fn load_game_from_db(
        database: &mut Database,
        n: usize,
        pattern: CarpetPattern,
        difficulty: SudokuDifficulty,
    ) -> Self {
        database
            .get_random_canonical_carpet_game(n as i16, pattern.to_db(), difficulty as i16)
            .unwrap()
    }

    // TODO:
    // pub fn generate_filled_from_db(
    //     database: &mut Database,
    //     n: usize,
    //     pattern: CarpetPattern,
    // ) -> Self {
    //     let (db_carpet, db_carpet_sudokus, db_sudokus) = database
    //         .construct_canonical_carpet(n as i16, pattern)
    //         .unwrap();
    //     Self::db_from_filled(db_carpet, db_carpet_sudokus, db_sudokus)
    // }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////   IMPLEMENTATIONS   //////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for CarpetSudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CARPET: {}, pattern: {}, difficulty: {}, filled_board_hash: {}, filled_cells: {}",
            if self.is_canonical {
                "CANONICAL"
            } else {
                "RANDOMIZED"
            },
            self.pattern,
            self.difficulty,
            self.filled_board_hash,
            self.get_filled_cells()
        )?;

        for (i, sudoku) in self.sudokus.iter().enumerate() {
            writeln!(f, "Sudoku {}:\t{}", i, sudoku)?;
        }
        Ok(())
    }
}

impl PartialEq for CarpetSudoku {
    fn eq(&self, other: &Self) -> bool {
        if self.n.ne(&other.n)
            || self.pattern.ne(&other.pattern)
            || self.difficulty.ne(&other.difficulty)
            || self.is_canonical.ne(&other.is_canonical)
            || self.filled_board_hash.ne(&other.filled_board_hash)
        {
            return false;
        }

        for (sudoku_id, sudoku1) in self.sudokus.iter().enumerate() {
            let sudoku2 = other.sudokus.get(sudoku_id).unwrap();

            for x in 0..self.n2 {
                for y in 0..self.n2 {
                    let value1 = sudoku1.get_cell_value(x, y);
                    let value2 = sudoku2.get_cell_value(x, y);
                    if value1 != value2 {
                        return false;
                    }

                    let possibilities1 = sudoku1.get_cell_possibilities(x, y);
                    let possibilities2 = sudoku2.get_cell_possibilities(x, y);
                    if possibilities1.len() != possibilities2.len() {
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
