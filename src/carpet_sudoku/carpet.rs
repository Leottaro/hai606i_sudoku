use crate::simple_sudoku::{Coords, Sudoku, SudokuDifficulty, SudokuError, SudokuGroups};

use super::{CarpetPattern, CarpetSudoku};
use rand::{rng, seq::SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

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

        for &(square1, sudoku2, square2) in self.links.to_owned().get(&sudoku_id).unwrap() {
            if square_id != square1 {
                continue;
            }
            let y2 = (square2 / self.n) * self.n;
            let x2 = (square2 % self.n) * self.n;
            twins.push((sudoku2, x2 + dx, y2 + dy));
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
            pattern => pattern,
        };
        let n_sudokus = pattern.get_n_sudokus();

        let sudokus = (0..n_sudokus).map(|_| Sudoku::new(n)).collect();

        let mut links: HashMap<usize, HashSet<(usize, usize, usize)>> = HashMap::new();
        for sudoku_id in 0..n_sudokus {
            links.insert(sudoku_id, HashSet::new());
        }
        for ((sudoku1, square1), (sudoku2, square2)) in pattern.get_raw_links(n) {
            let sudoku1_links = links.get_mut(&sudoku1).unwrap();
            sudoku1_links.insert((square1, sudoku2, square2));
            let sudoku2_links = links.get_mut(&sudoku2).unwrap();
            sudoku2_links.insert((square2, sudoku1, square1));
        }

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

    pub fn generate_full(n: usize, pattern: CarpetPattern) -> Self {
        loop {
            let mut carpet = Self::new(n, pattern);
            if !carpet._generate_canonical_from(0, 0, 0) {
                panic!("pattern: {pattern} juste pas possible en fait");
            }
            if carpet.count_solutions(Some(1)) == 0 {
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

            if self.count_solutions(Some(1)) > 0
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

    pub fn generate_new(n: usize, pattern: CarpetPattern, difficulty: SudokuDifficulty) -> Self {
        let time = std::time::Instant::now();
        let generated_carpet = Self::generate_full(n, pattern).generate_from(difficulty);
        println!("called generate_new({n}, {pattern}, {difficulty}) gave this in {}ms => {generated_carpet}", time.elapsed().as_millis());
        generated_carpet
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
    ////////////////////////////////////////////////////////   GENERATION   ////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn generate_from(&self, aimed_difficulty: SudokuDifficulty) -> Self {
        let mut carpet = self.clone();
        println!("generate_from({aimed_difficulty}): ");
        let start_time = std::time::Instant::now();
        let mut explored_possibilities = 0;
        let mut skipped_possibilities = 0;
        let mut already_explored_filled_cells = HashSet::new();
        carpet._generate_from(
            aimed_difficulty,
            &start_time,
            &mut explored_possibilities,
            &mut skipped_possibilities,
            &mut already_explored_filled_cells,
        );
        println!(
            "Skipped {skipped_possibilities}/{explored_possibilities} possibilities in {}ms          ",
            start_time.elapsed().as_millis()
        );
        carpet
    }

    fn _generate_from(
        &mut self,
        aimed_difficulty: SudokuDifficulty,
        start_time: &std::time::Instant,
        explored_possibilities: &mut usize,
        skipped_possibilities: &mut usize,
        already_explored_filled_cells: &mut HashSet<Vec<bool>>,
    ) -> bool {
        self.difficulty = SudokuDifficulty::Unknown;

        let mut rng = rand::rng();
        let (mut exploring_cells, mut exploring_filled_cells) = {
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

            let exploring_filled_cells = temp
                .iter()
                .map(|(_, _, _, value)| *value > 0)
                .collect::<Vec<_>>();

            let exploring_cells = temp
                .into_iter()
                .filter(|(_, _, _, value)| *value > 0)
                .collect::<Vec<_>>();

            (exploring_cells, exploring_filled_cells)
        };
        exploring_cells.shuffle(&mut rng);

        if already_explored_filled_cells.contains(&exploring_filled_cells) {
            return false;
        }
        already_explored_filled_cells.insert(exploring_filled_cells.clone());

        let mut did_anything = false;
        for (sudoku_id, x, y, removed_value) in exploring_cells {
            let twin_cells = self.get_twin_cells(sudoku_id, x, y);
            self.remove_value(sudoku_id, x, y).unwrap();
            for (i, x, y) in &twin_cells {
                exploring_filled_cells[(*i * self.n2 + *y) * self.n2 + *x] = false;
            }

            if already_explored_filled_cells.contains(&exploring_filled_cells) {
                *skipped_possibilities += 1;
                print!("Skipped {skipped_possibilities}/{explored_possibilities} possibilities in {}ms          \r", start_time.elapsed().as_millis());
            } else {
                *explored_possibilities += 1;
                print!("Skipped {skipped_possibilities}/{explored_possibilities} possibilities in {}ms          \r", start_time.elapsed().as_millis());
                let mut carpet = self.clone();
                carpet.rule_solve_until((false, false), Some(aimed_difficulty));
                if carpet.is_filled() {
                    did_anything = true;
                    // if carpet.difficulty == aimed_difficulty {
                    //     println!(
                    //         "Found a solution with {aimed_difficulty} difficulty and {} filled cells at {}ms",
                    // 		exploring_filled_cells.iter().filter(|&&x| x).count(),
                    //         start_time.elapsed().as_millis()
                    //     );
                    // }
                    if self._generate_from(
                        aimed_difficulty,
                        start_time,
                        explored_possibilities,
                        skipped_possibilities,
                        already_explored_filled_cells,
                    ) {
                        return true;
                    }
                }
            }

            self.set_value(sudoku_id, x, y, removed_value).unwrap();
            for (i, x, y) in &twin_cells {
                exploring_filled_cells[(*i * self.n2 + *y) * self.n2 + *x] = true;
            }
        }

        if !did_anything {
            let mut verify_carpet = self.clone();
            verify_carpet.rule_solve_until((false, false), Some(aimed_difficulty));
            if verify_carpet.is_filled()
                && verify_carpet.difficulty == aimed_difficulty
                && self.count_solutions(Some(2)) == 1
            {
                self.difficulty = aimed_difficulty;
                return true;
            }
        }

        false
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////   CANONIZATION   ///////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn randomize(&mut self) -> Result<(), SudokuError> {
        if !self.is_filled() {
            return Err(SudokuError::InvalidState(format!(
                "randomize() when this carpet isn't filled: {self}"
            )));
        }
        if !self.is_canonical {
            return Err(SudokuError::InvalidState(format!(
                "randomize() when this carpet is already randomized: {self}"
            )));
        }

        self.sudokus[0].randomize(None, None)?;
        let rows_swap = self.sudokus[0].get_rows_swap();
        let values_swap = self.sudokus[0].get_values_swap();
        for sudoku in self.sudokus.iter_mut().skip(1) {
            sudoku.randomize(Some(rows_swap.clone()), Some(values_swap.clone()))?;
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
        if !self.is_canonical {
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
        self.count_solutions(Some(1)) == 1
    }

    pub fn count_solutions(&self, max_solutions: Option<usize>) -> usize {
        self.clone()._count_solutions(
            max_solutions,
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

    fn _count_solutions(
        &mut self,
        max_solutions: Option<usize>,
        mut empty_cells: Vec<(usize, usize, usize)>,
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

            sub_solutions += self._count_solutions(max_solutions, empty_cells.clone());
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

    pub fn db_sudokus_to_filled(&self) -> Vec<(DBCanonicalSudoku, Vec<DBCanonicalSudokuSquare>)> {
        self.sudokus
            .iter()
            .map(|sudoku| sudoku.filled_to_db().unwrap())
            .collect()
    }

    pub fn db_to_game(&self) -> DBNewCanonicalCarpetGame {
        let filled_cells: Vec<u8> = (0..self.sudokus.len() * self.n2 * self.n2)
            .map(|i| {
                let sudoku_id = i / (self.n2 * self.n2);
                let cell_i = i - sudoku_id * self.n2 * self.n2;
                let y = cell_i / self.n2;
                let x = cell_i % self.n2;
                (self.sudokus[sudoku_id].get_cell_value(x, y) > 0) as u8
            })
            .collect();
        DBNewCanonicalCarpetGame {
            carpet_game_carpet_filled_board_hash: self
                .filled_board_hash
                .wrapping_sub(u64::MAX / 2 + 1)
                as i64,
            carpet_game_difficulty: self.difficulty as i16,
            carpet_game_filled_cells: filled_cells.clone(),
            carpet_game_filled_cells_count: filled_cells.len() as i16,
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
            if is_filled == 0 {
                let sudoku_id = i / (carpet.n2 * carpet.n2);
                let cell_i = i - sudoku_id * carpet.n2 * carpet.n2;
                let y = cell_i / carpet.n2;
                let x = cell_i % carpet.n2;
                carpet.remove_value(sudoku_id, x, y).unwrap();
            }
        }

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
