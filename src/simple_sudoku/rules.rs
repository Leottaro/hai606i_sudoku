use log::warn;
use std::collections::HashSet;

use super::{Sudoku, SudokuGroups::*};

macro_rules! debug_only {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            log::debug!($($arg)*);
        }
    };
}

impl Sudoku {
    // RULES SOLVING
    // CHECK https://www.taupierbw.be/SudokuCoach
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT CHANGED SOMETHING

    // règle 1: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    pub(super) fn naked_singles(&mut self) -> bool {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() == 1 {
                    let &value = self.possibility_board[y][x].iter().next().unwrap();
                    self.fix_value(x, y, value);
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return true;
                }
            }
        }
        false
    }

    // règle 2: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    pub(super) fn hidden_singles(&mut self) -> bool {
        for group in self.groups.get(&ALL).unwrap() {
            for value in 1..=self.n2 {
                let cells_with_value: Vec<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if cells_with_value.len() == 1 {
                    let &&(x, y) = cells_with_value.first().unwrap();
                    self.fix_value(x, y, value);
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return true;
                }
            }
        }
        false
    }

    // règle 3: http://www.taupierbw.be/SudokuCoach/SC_NakedPairs.shtml
    pub(super) fn naked_pairs(&mut self) -> bool {
        let mut modified = false;
        for group in self.groups.get(&ALL).unwrap() {
            let pairs: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].len() == 2)
                .collect();

            for i in 0..pairs.len() {
                for j in (i + 1)..pairs.len() {
                    let &(x1, y1) = pairs[i];
                    let &(x2, y2) = pairs[j];
                    if self.possibility_board[y1][x1] == self.possibility_board[y2][x2] {
                        for &(x, y) in group.iter() {
                            if (x, y) == *pairs[i] || (x, y) == *pairs[j] {
                                continue;
                            }
                            for value in self.possibility_board[y1][x1].clone() {
                                if self.possibility_board[y][x].remove(&value) {
                                    debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                                    modified = true;
                                }
                            }
                        }
                        if modified {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// règle 4: http://www.taupierbw.be/SudokuCoach/SC_NakedTriples.shtml
    pub(super) fn naked_triples(&mut self) -> bool {
        let mut modified = false;
        for group in self.groups.get(&ALL).unwrap() {
            let pairs_or_triples: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| {
                    self.possibility_board[y][x].len() == 2
                        || self.possibility_board[y][x].len() == 3
                })
                .collect();

            for i in 0..pairs_or_triples.len() {
                for j in (i + 1)..pairs_or_triples.len() {
                    for k in (j + 1)..pairs_or_triples.len() {
                        let &(x1, y1) = pairs_or_triples[i];
                        let &(x2, y2) = pairs_or_triples[j];
                        let &(x3, y3) = pairs_or_triples[k];
                        let common_possibilities: HashSet<usize> = self.possibility_board[y1][x1]
                            .union(&self.possibility_board[y2][x2])
                            .chain(&self.possibility_board[y3][x3])
                            .cloned()
                            .collect();
                        if common_possibilities.len() == 3 {
                            for &(x, y) in group.iter() {
                                if (x, y) == *pairs_or_triples[i]
                                    || (x, y) == *pairs_or_triples[j]
                                    || (x, y) == *pairs_or_triples[k]
                                {
                                    continue;
                                }
                                for &value in common_possibilities.iter() {
                                    if self.possibility_board[y][x].remove(&value) {
                                        #[cfg(debug_assertions)]
                                        debug_only!(
                                            "possibilitée {value} supprimée de x: {x}, y: {y}"
                                        );
                                        modified = true;
                                    }
                                }
                            }
                            if modified {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// règle 5: http://www.taupierbw.be/SudokuCoach/SC_HiddenPairs.shtml
    pub(super) fn hidden_pairs(&mut self) -> bool {
        for group in self.groups.get(&ALL).unwrap() {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.len() != 2 {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&(usize, usize)> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value1 != occurences_value2 {
                        continue;
                    }
                    let mut modified = false;
                    for &&(x, y) in occurences_value1.iter() {
                        for value in 1..=self.n2 {
                            if value != value1
                                && value != value2
                                && self.possibility_board[y][x].remove(&value)
                            {
                                debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                                modified = true;
                            }
                        }
                    }
                    if modified {
                        return true;
                    }
                }
            }
        }
        false
    }

    // règle 6: http://www.taupierbw.be/SudokuCoach/SC_HiddenTriples.shtml
    pub(super) fn hidden_triples(&mut self) -> bool {
        for group in self.groups.get(&ALL).unwrap() {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&(usize, usize)> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in (value2 + 1)..=self.n2 {
                        let occurences_value3: HashSet<&(usize, usize)> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();

                        if occurences_value3.is_empty() {
                            continue;
                        }

                        let common_occurences: HashSet<&&(usize, usize)> = occurences_value1
                            .union(&occurences_value2)
                            .chain(&occurences_value3)
                            .collect();

                        if common_occurences.len() != 3 {
                            continue;
                        }
                        let mut modified = false;
                        for &&(x, y) in common_occurences.into_iter() {
                            for value in 1..=self.n2 {
                                if value != value1
                                    && value != value2
                                    && value != value3
                                    && self.possibility_board[y][x].remove(&value)
                                {
                                    debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                                    modified = true;
                                }
                            }
                        }
                        if modified {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // règle 7: http://www.taupierbw.be/SudokuCoach/SC_NakedQuads.shtml
    pub(super) fn naked_quads(&mut self) -> bool {
        let mut modified = false;
        for group in self.groups.get(&ALL).unwrap() {
            let pairs_or_triples_or_quads: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| {
                    self.possibility_board[y][x].len() >= 2
                        && self.possibility_board[y][x].len() <= 4
                })
                .collect();

            for i in 0..pairs_or_triples_or_quads.len() {
                for j in (i + 1)..pairs_or_triples_or_quads.len() {
                    for k in (j + 1)..pairs_or_triples_or_quads.len() {
                        for l in (k + 1)..pairs_or_triples_or_quads.len() {
                            let &(x1, y1) = pairs_or_triples_or_quads[i];
                            let &(x2, y2) = pairs_or_triples_or_quads[j];
                            let &(x3, y3) = pairs_or_triples_or_quads[k];
                            let &(x4, y4) = pairs_or_triples_or_quads[l];
                            let common_possibilities: HashSet<usize> = self.possibility_board[y1]
                                [x1]
                                .union(&self.possibility_board[y2][x2])
                                .chain(
                                    self.possibility_board[y3][x3]
                                        .union(&self.possibility_board[y4][x4]),
                                )
                                .cloned()
                                .collect();
                            if common_possibilities.len() == 4 {
                                for &(x, y) in group.iter() {
                                    if (x, y) == *pairs_or_triples_or_quads[i]
                                        || (x, y) == *pairs_or_triples_or_quads[j]
                                        || (x, y) == *pairs_or_triples_or_quads[k]
                                        || (x, y) == *pairs_or_triples_or_quads[l]
                                    {
                                        continue;
                                    }

                                    for &value in common_possibilities.iter() {
                                        if self.possibility_board[y][x].remove(&value) {
                                            debug_only!(
                                                "possibilitée {value} supprimée de x: {x}, y: {y}"
                                            );
                                            modified = true;
                                        }
                                    }
                                }
                                if modified {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // règle 8: http://www.taupierbw.be/SudokuCoach/SC_HiddenQuads.shtml
    pub(super) fn hidden_quads(&mut self) -> bool {
        for group in self.groups.get(&ALL).unwrap() {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&(usize, usize)> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in (value2 + 1)..=self.n2 {
                        let occurences_value3: HashSet<&(usize, usize)> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();
                        if occurences_value3.is_empty() {
                            continue;
                        }
                        for value4 in (value3 + 1)..=self.n2 {
                            let occurences_value4: HashSet<&(usize, usize)> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value4))
                                .collect();
                            if occurences_value4.is_empty() {
                                continue;
                            }

                            let common_occurences: HashSet<&&(usize, usize)> = occurences_value1
                                .union(&occurences_value2)
                                .chain(occurences_value3.union(&occurences_value4))
                                .collect();

                            if common_occurences.len() != 4 {
                                continue;
                            }
                            let mut modified = false;
                            for &&(x, y) in common_occurences.into_iter() {
                                for value in 1..=self.n2 {
                                    if value != value1
                                        && value != value2
                                        && value != value3
                                        && value != value4
                                        && self.possibility_board[y][x].remove(&value)
                                    {
                                        debug_only!(
                                            "possibilitée {value} supprimée de x: {x}, y: {y}"
                                        );
                                        modified = true;
                                    }
                                }
                            }
                            if modified {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // règle 9: http://www.taupierbw.be/SudokuCoach/SC_PointingPair.shtml
    pub(super) fn pointing_pair(&mut self) -> bool {
        for square in self.groups.get(&SQUARE).unwrap() {
            for value in 1..=self.n2 {
                let occurences: Vec<&(usize, usize)> = square
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 {
                    continue;
                }
                let &(x1, y1) = occurences[0];
                let &(x2, y2) = occurences[1];
                let mut modified = false;
                if x1 == x2 {
                    for y in 0..self.n2 {
                        if y == y1 || y == y2 {
                            continue;
                        }
                        if self.possibility_board[y][x1].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x1}, y: {y}");
                            modified = true;
                        }
                    }
                } else if y1 == y2 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x}, y: {y1}");
                            modified = true;
                        }
                    }
                } else {
                    continue;
                }
                if modified {
                    return true;
                }
            }
        }
        false
    }

    // règle 10: http://www.taupierbw.be/SudokuCoach/SC_PointingTriple.shtml
    pub(super) fn pointing_triple(&mut self) -> bool {
        for square in self.groups.get(&SQUARE).unwrap() {
            for value in 1..=self.n2 {
                let occurences: Vec<&(usize, usize)> = square
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences[0];
                let &(x2, y2) = occurences[1];
                let &(x3, y3) = occurences[2];
                let mut modified = false;
                if x1 == x2 && x2 == x3 {
                    for y in 0..self.n2 {
                        if y == y1 || y == y2 || y == y3 {
                            continue;
                        }
                        if self.possibility_board[y][x1].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x1}, y: {y}");
                            modified = true;
                        }
                    }
                } else if y1 == y2 && y2 == y3 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 || x == x3 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x}, y: {y1}");
                            modified = true;
                        }
                    }
                } else {
                    continue;
                }
                if modified {
                    return true;
                }
            }
        }
        false
    }

    // règle 11: http://www.taupierbw.be/SudokuCoach/SC_BoxReduction.shtml
    pub(super) fn box_reduction(&mut self) -> bool {
        let mut modified = false;
        for rows in self.groups.get(&ROW).unwrap() {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&(usize, usize)> = rows
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 && occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(x, _)| x / self.n == x1 / self.n) {
                    for &(x, y) in self.cell_groups.get(&(x1, y1, SQUARE)).unwrap() {
                        if y == y1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                            modified = true;
                        }
                    }
                    if modified {
                        return true;
                    }
                }
            }
        }

        for cols in self.groups.get(&COLUMN).unwrap() {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&(usize, usize)> = cols
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 && occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(_, y)| y / self.n == y1 / self.n) {
                    for &(x, y) in self.cell_groups.get(&(x1, y1, SQUARE)).unwrap() {
                        if x == x1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                            modified = true;
                        }
                    }
                    if modified {
                        return true;
                    }
                }
            }
        }

        false
    }

    // règle 12: http://www.taupierbw.be/SudokuCoach/SC_XWing.shtml
    pub(super) fn x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..(self.n2 - 1) {
                for i2 in (i1 + 1)..self.n2 {
                    let mut picked_cells: Vec<(&str, (usize, usize))> = Vec::new();

                    let row1_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i1][*x].contains(&value))
                        .collect();
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();

                    if row1_positions.len() == 2 && row1_positions == row2_positions {
                        let row1_vec: Vec<usize> = row1_positions.into_iter().collect();
                        let (x1, x2) = (row1_vec[0], row1_vec[1]);

                        let col1 = self.cell_groups.get(&(x1, i1, COLUMN)).unwrap();
                        let col2 = self.cell_groups.get(&(x2, i1, COLUMN)).unwrap();
                        for &(x, y) in col1.union(col2) {
                            if y == i1 || y == i2 {
                                continue;
                            }
                            picked_cells.push(("rows", (x, y)));
                        }
                    }

                    let col1_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i1].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();

                    if col1_positions.len() == 2 && col1_positions == col2_positions {
                        let col1_vec: Vec<usize> = col1_positions.into_iter().collect();
                        let (y1, y2) = (col1_vec[0], col1_vec[1]);

                        let row1 = self.cell_groups.get(&(i1, y1, ROW)).unwrap();
                        let row2 = self.cell_groups.get(&(i1, y2, ROW)).unwrap();
                        for &(x, y) in row1.union(row2) {
                            if x == i1 || x == i2 {
                                continue;
                            }
                            picked_cells.push(("cols", (x, y)));
                        }
                    }

                    for (_is_row, (x, y)) in picked_cells {
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("{_is_row} {i1} and {i2}: possibilitée {value} supprimée de x: {x}, y: {y}");
                            modified = true;
                        }
                    }

                    if modified {
                        return true;
                    }
                }
            }
        }
        false
    }

    // règle 13: http://www.taupierbw.be/SudokuCoach/SC_FinnedXWing.shtml
    pub(super) fn finned_x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..(self.n2 - 1) {
                for i2 in (i1 + 1)..self.n2 {
                    if i1 / self.n == i2 / self.n {
                        continue;
                    }
                    let mut picked_cells: Vec<(&str, (usize, usize), (usize, usize))> = Vec::new();

                    let row1_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i1][*x].contains(&value))
                        .collect();
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();

                    let (smaller_row, larger_row, is_row2_larger) =
                        if row1_positions.len() < row2_positions.len() {
                            (&row1_positions, &row2_positions, true)
                        } else {
                            (&row2_positions, &row1_positions, false)
                        };

                    if smaller_row.len() == 2
                        && larger_row.len() == 3
                        && smaller_row.is_subset(larger_row)
                    {
                        let fin = larger_row.difference(smaller_row).next().unwrap();
                        let fin_i = if is_row2_larger { i2 } else { i1 };

                        let smaller_vec: Vec<usize> = smaller_row.iter().cloned().collect();
                        let (x1, x2) = (smaller_vec[0], smaller_vec[1]);
                        if fin / self.n == x1 / self.n {
                            picked_cells.push(("rows", (x1, fin_i), (*fin, fin_i)));
                        } else if fin / self.n == x2 / self.n {
                            picked_cells.push(("rows", (x2, fin_i), (*fin, fin_i)));
                        }
                    }

                    let col1_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i1].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();

                    let (smaller_col, larger_col, is_col2_larger) =
                        if col1_positions.len() < col2_positions.len() {
                            (&col1_positions, &col2_positions, true)
                        } else {
                            (&col2_positions, &col1_positions, false)
                        };

                    if smaller_col.len() == 2
                        && larger_col.len() == 3
                        && smaller_col.is_subset(larger_col)
                    {
                        let fin = larger_col.difference(smaller_col).next().unwrap();
                        let fin_i = if is_col2_larger { i2 } else { i1 };

                        let smaller_vec: Vec<usize> = smaller_col.iter().cloned().collect();
                        let (y1, y2) = (smaller_vec[0], smaller_vec[1]);
                        if fin / self.n == y1 / self.n {
                            picked_cells.push(("cols", (fin_i, y1), (fin_i, *fin)));
                        } else if fin / self.n == y2 / self.n {
                            picked_cells.push(("cols", (fin_i, y2), (fin_i, *fin)));
                        }
                    }

                    for (_is_row, (x1, y1), (fin_x, fin_y)) in picked_cells {
                        let removed_cells: Vec<&(usize, usize)> = self
                            .cell_groups
                            .get(&(fin_x, fin_y, SQUARE))
                            .unwrap()
                            .into_iter()
                            .filter(|(x, y)| {
                                (y1 == fin_y && *x == x1 && *y != y1)
                                    || (x1 == fin_x && *x != x1 && *y == y1)
                            })
                            .collect();
                        for &(x, y) in removed_cells {
                            if self.possibility_board[y][x].remove(&value) {
                                debug_only!("{_is_row} {i1} and {i2}: possibilitée {value} supprimée de x: {x}, y: {y}");
                                modified = true;
                            }
                        }
                        if modified {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // règle 14: http://www.taupierbw.be/SudokuCoach/SC_FrankenXWing.shtml
    pub(super) fn franken_x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            for line in self.groups.get(&LINES).unwrap() {
                let occurences: Vec<&(usize, usize)> = line
                    .iter()
                    .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                    .collect();
                if occurences.len() != 2
                    || occurences[0].0 / self.n != occurences[1].0 / self.n
                    || occurences[0].1 / self.n != occurences[1].1 / self.n
                {
                    continue;
                }
                let (&(x1, y1), &(x2, y2)) = (occurences[0], occurences[1]);

                for square in self.groups.get(&SQUARE).unwrap() {
                    if !line.is_disjoint(&square) {
                        continue;
                    }

                    let (mut yellow_cells1, mut yellow_cells2): (
                        HashSet<(usize, usize)>,
                        HashSet<(usize, usize)>,
                    ) = if y1 == y2 {
                        (
                            self.cell_groups.get(&(x1, y1, COLUMN)).unwrap().clone(),
                            self.cell_groups.get(&(x2, y2, COLUMN)).unwrap().clone(),
                        )
                    } else {
                        (
                            self.cell_groups.get(&(x1, y1, ROW)).unwrap().clone(),
                            self.cell_groups.get(&(x2, y2, ROW)).unwrap().clone(),
                        )
                    };
                    yellow_cells1.remove(&(x1, y1));
                    yellow_cells2.remove(&(x2, y2));

                    let red_cells_1_value_count = yellow_cells1
                        .intersection(&square)
                        .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                        .count();
                    let red_cells_2_value_count = yellow_cells2
                        .intersection(&square)
                        .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                        .count();

                    let square_cells_value_count = square
                        .iter()
                        .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                        .count();

                    if red_cells_1_value_count == 0
                        || red_cells_2_value_count == 0
                        || square_cells_value_count
                            != red_cells_1_value_count + red_cells_2_value_count
                    {
                        continue;
                    }

                    for &(x, y) in yellow_cells1
                        .difference(&square)
                        .into_iter()
                        .chain(yellow_cells2.difference(&square).into_iter())
                    {
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x}, y: {y}");
                            modified = true;
                        }
                    }

                    if modified {
                        return true;
                    }
                }
            }
        }
        false
    }

    // règle 15: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedXWing.shtml
    pub(super) fn finned_mutant_x_xing(&mut self) -> bool {
        warn!("sashimi_finned_x_wing isn't implemented yet");
        false
    }

    // règle 16: http://www.taupierbw.be/SudokuCoach/SC_Skyscraper.shtml
    pub(super) fn skyscraper(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..(self.n2 - 1) {
                for i2 in (i1 + 1)..self.n2 {
                    // i1 and i2 represents rows or columns
                    let mut picked_cells: Vec<(&str, (usize, usize), (usize, usize))> = Vec::new();

                    let row1_positions: Vec<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i1][*x].contains(&value))
                        .collect();
                    let row2_positions: Vec<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    if row1_positions.len() == 2 && row2_positions.len() == 2 {
                        let x11 = row1_positions[0];
                        let x12 = row1_positions[1];
                        let x21 = row2_positions[0];
                        let x22 = row2_positions[1];
                        if x11 == x21 || x12 == x22 {
                            let (x1, x2) = if x11 == x21 { (x12, x22) } else { (x11, x21) };
                            picked_cells.push(("rows", (x1, i1), (x2, i2)));
                        }
                    }

                    let col1_positions: Vec<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i1].contains(&value))
                        .collect();
                    let col2_positions: Vec<usize> = (0..self.n2)
                        .into_iter()
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    if col1_positions.len() == 2 && col2_positions.len() == 2 {
                        let y11 = col1_positions[0];
                        let y12 = col1_positions[1];
                        let y21 = col2_positions[0];
                        let y22 = col2_positions[1];
                        if y11 == y21 || y12 == y22 {
                            let (y1, y2) = if y11 == y21 { (y12, y22) } else { (y11, y21) };
                            picked_cells.push(("cols", (i1, y1), (i2, y2)));
                        }
                    }

                    for (_is_row, (x1, y1), (x2, y2)) in picked_cells {
                        let cell_group1: &HashSet<(usize, usize)> =
                            self.cell_groups.get(&(x1, y1, ALL)).unwrap();
                        let cell_group2: &HashSet<(usize, usize)> =
                            self.cell_groups.get(&(x2, y2, ALL)).unwrap();
                        let common_cells: HashSet<&(usize, usize)> =
                            cell_group1.intersection(cell_group2).collect();

                        for &(x, y) in common_cells {
                            if (x == x1 && y == y1) || (x == x2 && y == y2) {
                                continue;
                            }

                            if self.possibility_board[y][x].remove(&value) {
                                debug_only!("{_is_row} {i1} and {i2}: {x1},{y1} et {x2},{y2}: possibilitée {value} supprimée de x: {x}, y: {y}");
                                modified = true;
                            }
                        }

                        if modified {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // règle 17: http://www.taupierbw.be/SudokuCoach/SC_YWing.shtml
    pub(super) fn y_wing(&mut self) -> bool {
        let mut modified = false;
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() != 2 {
                    continue;
                }
                let (value1, value2) = {
                    let temp = self.possibility_board[y][x].iter().collect::<Vec<_>>();
                    (temp[0].clone(), temp[1].clone())
                };
                let cell_groups: &HashSet<(usize, usize)> =
                    self.cell_groups.get(&(x, y, ALL)).unwrap();

                let b1_values = cell_groups.iter().filter(|(x1, y1)| {
                    let possibilities = &self.possibility_board[*y1][*x1];
                    possibilities.len() == 2
                        && possibilities.contains(&value1)
                        && !possibilities.contains(&value2)
                });

                let b2_values: Vec<&(usize, usize)> = cell_groups
                    .iter()
                    .filter(|(x2, y2)| {
                        let possibilities = &self.possibility_board[*y2][*x2];
                        possibilities.len() == 2
                            && possibilities.contains(&value2)
                            && !possibilities.contains(&value1)
                    })
                    .collect();

                let mut bi_values: Vec<(usize, (usize, usize), (usize, usize))> = Vec::new();
                for (x1, y1) in b1_values {
                    for (x2, y2) in b2_values.iter() {
                        let possible_value3: Option<&usize> = self.possibility_board[*y1][*x1]
                            .intersection(&self.possibility_board[*y2][*x2])
                            .into_iter()
                            .next();
                        if let Some(value3) = possible_value3 {
                            bi_values.push((*value3, (*x1, *y1), (*x2, *y2)));
                        }
                    }
                }

                for (value, (x1, y1), (x2, y2)) in bi_values {
                    let cell_group1: &HashSet<(usize, usize)> =
                        self.cell_groups.get(&(x1, y1, ALL)).unwrap();
                    let cell_group2: &HashSet<(usize, usize)> =
                        self.cell_groups.get(&(x2, y2, ALL)).unwrap();
                    let common_cells: HashSet<&(usize, usize)> =
                        cell_group1.intersection(&cell_group2).collect();
                    for &(x3, y3) in common_cells {
                        if (x3 == x1 && y3 == y1) || (x3 == x2 && y3 == y2) {
                            continue;
                        }
                        if self.possibility_board[y3][x3].remove(&value) {
                            debug_only!("possibilitée {value} supprimée de x: {x3}, y: {y3}");
                            modified = true;
                        }
                    }
                }

                if modified {
                    return true;
                }
            }
        }
        false
    }

    // règle 18: http://www.taupierbw.be/SudokuCoach/SC_WWing.shtml
    pub(super) fn w_wing(&mut self) -> bool {
        let mut modified = false;
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() != 2 {
                    continue;
                }
                let cell_groups: &HashSet<(usize, usize)> =
                    self.cell_groups.get(&(x, y, ALL)).unwrap();

                let possible_values: Vec<(usize, usize)> = {
                    let possible_values: Vec<usize> =
                        self.possibility_board[y][x].clone().into_iter().collect();
                    vec![
                        (possible_values[0], possible_values[1]),
                        (possible_values[1], possible_values[0]),
                    ]
                };

                for (value1, value2) in possible_values {
                    for &(x1, y1) in cell_groups.iter() {
                        if !self.possibility_board[y1][x1].contains(&value1) {
                            continue;
                        }

                        let cell1_groups = vec![
                            self.cell_groups.get(&(x1, y1, ROW)).unwrap(),
                            self.cell_groups.get(&(x1, y1, COLUMN)).unwrap(),
                            self.cell_groups.get(&(x1, y1, SQUARE)).unwrap(),
                        ];
                        for group in cell1_groups {
                            if group.contains(&(x, y)) {
                                continue;
                            }
                            let strong_link: HashSet<&(usize, usize)> = group
                                .into_iter()
                                .filter(|&&(x2, y2)| {
                                    (x2 != x1 || y2 != y1)
                                        && self.possibility_board[y2][x2].contains(&value1)
                                })
                                .collect();
                            if strong_link.len() != 1 {
                                continue;
                            }
                            let &(x2, y2) = strong_link.into_iter().next().unwrap();

                            let mut picked_cells: Vec<(usize, usize)> = Vec::new();

                            let cell2_groups = vec![
                                self.cell_groups.get(&(x2, y2, ROW)).unwrap(),
                                self.cell_groups.get(&(x2, y2, COLUMN)).unwrap(),
                                self.cell_groups.get(&(x2, y2, SQUARE)).unwrap(),
                            ];
                            for group in cell2_groups {
                                if group.contains(&(x1, y1)) {
                                    continue;
                                }

                                for &(x3, y3) in group {
                                    if (x3 == x || y3 == y)
                                        || (x3 == x2 && y3 == y2)
                                        || self.possibility_board[y3][x3]
                                            != self.possibility_board[y][x]
                                    {
                                        continue;
                                    }
                                    picked_cells.push((x, y3));
                                    picked_cells.push((x3, y));
                                }
                            }

                            for (x3, y3) in picked_cells {
                                if self.possibility_board[y3][x3].remove(&value2) {
                                    debug_only!(
                                        "possibilitée {value2} supprimée de x: {x3}, y: {y3}"
                                    );
                                    modified = true;
                                }
                            }

                            if modified {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // règle 19: http://www.taupierbw.be/SudokuCoach/SC_Swordfish.shtml
    pub(super) fn swordfish(&mut self) -> bool {
        warn!("swordfish isn't implemented yet");
        false
    }

    // règle 20: http://www.taupierbw.be/SudokuCoach/SC_FinnedSwordfish.shtml
    pub(super) fn finned_swordfish(&mut self) -> bool {
        warn!("finned_swordfish isn't implemented yet");
        false
    }

    // règle 21: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedSwordfish.shtml
    pub(super) fn sashimi_finned_swordfish(&mut self) -> bool {
        warn!("sashimi_finned_swordfish isn't implemented yet");
        false
    }

    // règle 22: http://www.taupierbw.be/SudokuCoach/SC_XYZWing.shtml
    pub(super) fn xyz_wing(&mut self) -> bool {
        warn!("xyz_wing isn't implemented yet");

        false
    }

    // règle 23: http://www.taupierbw.be/SudokuCoach/SC_BUG.shtml
    pub(super) fn bi_value_universal_grave(&mut self) -> bool {
        let mut unique_triple: Option<(usize, usize)> = None;
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let possibilities_number = self.possibility_board[y][x].len();
                if possibilities_number == 0 || possibilities_number == 2 {
                    continue;
                }
                if possibilities_number != 3 {
                    return false;
                }
                if unique_triple.is_some() {
                    return false;
                }
                unique_triple = Some((x, y));
            }
        }

        if unique_triple.is_none() {
            return false;
        }

        let (x0, y0) = unique_triple.unwrap();
        let mut appearing_value: Vec<usize> = vec![0; self.n2];
        for &(x, y) in self.cell_groups.get(&(x0, y0, ALL)).unwrap() {
            for value in self.possibility_board[y][x].iter() {
                appearing_value[*value - 1] += 1;
            }
        }

        let max_appearing_value = appearing_value
            .iter()
            .enumerate()
            .max_by_key(|&(_, count)| count)
            .map(|(index, _)| index + 1)
            .unwrap();

        self.fix_value(x0, y0, max_appearing_value);
        debug_only!(
            "valeur {} fixée en x: {}, y: {}",
            max_appearing_value,
            x0,
            y0
        );
        return true;
    }

    // règle 24: http://www.taupierbw.be/SudokuCoach/SC_XYChain.shtml
    pub(super) fn xy_chain(&mut self) -> bool {
        warn!("xy_chain isn't implemented yet");

        false
    }

    // règle 25: http://www.taupierbw.be/SudokuCoach/SC_Jellyfish.shtml
    pub(super) fn jellyfish(&mut self) -> bool {
        warn!("jellyfish isn't implemented yet");

        false
    }

    // règle 26: http://www.taupierbw.be/SudokuCoach/SC_FinnedJellyfish.shtml
    pub(super) fn finned_jellyfish(&mut self) -> bool {
        warn!("finned_jellyfish isn't implemented yet");

        false
    }

    // règle 27: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedJellyfish.shtml
    pub(super) fn sashimi_finned_jellyfish(&mut self) -> bool {
        warn!("sashimi_finned_jellyfish isn't implemented yet");

        false
    }

    // règle 28: http://www.taupierbw.be/SudokuCoach/SC_WXYZWing.shtml
    pub(super) fn wxyz_wing(&mut self) -> bool {
        warn!("wxyz_wing isn't implemented yet");

        false
    }

    // règle 29: http://www.taupierbw.be/SudokuCoach/SC_APE.shtml
    pub(super) fn subset_exclusion(&mut self) -> bool {
        warn!("subset_exclusion isn't implemented yet");

        false
    }

    // règle 30: http://www.taupierbw.be/SudokuCoach/SC_EmptyRectangle.shtml
    pub(super) fn empty_rectangle(&mut self) -> bool {
        warn!("empty_rectangle isn't implemented yet");

        false
    }

    // règle 31: http://www.taupierbw.be/SudokuCoach/SC_ALSchain.shtml
    pub(super) fn almost_locked_set_forcing_chain(&mut self) -> bool {
        warn!("almost_locked_set_forcing_chain isn't implemented yet");

        false
    }

    // règle 32: http://www.taupierbw.be/SudokuCoach/SC_DeathBlossom.shtml
    pub(super) fn death_blossom(&mut self) -> bool {
        warn!("death_blossom isn't implemented yet");

        false
    }

    // règle 33: http://www.taupierbw.be/SudokuCoach/SC_PatternOverlay.shtml
    pub(super) fn pattern_overlay(&mut self) -> bool {
        warn!("pattern_overlay isn't implemented yet");

        false
    }

    // règle 34: http://www.taupierbw.be/SudokuCoach/SC_BowmanBingo.shtml
    pub(super) fn bowmans_bingo(&mut self) -> bool {
        warn!("bowmans_bingo isn't implemented yet");

        false
    }
}
