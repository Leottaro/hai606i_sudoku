use graph::prelude::{Graph, GraphBuilder, UndirectedCsrGraph, UndirectedNeighbors};
use log::warn;
use std::collections::HashSet;

use super::{
    Coords, Sudoku,
    SudokuDifficulty::{self, *},
    SudokuGroups::*,
    SudokuRule,
};

macro_rules! debug_only {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            log::debug!($($arg)*);
        }
    };
}

/*
rules info:
rule  0: used in 0561120 sudokus (0.031ms avg)
rule  1: used in 0192951 sudokus (0.191ms avg)
rule  8: used in 0095349 sudokus (90.673ms avg)
rule  2: used in 0065314 sudokus (0.107ms avg)

rule  3: used in 0031991 sudokus (0.152ms avg)
rule  9: used in 0025122 sudokus (96.584ms avg)
rule 10: used in 0021918 sudokus (66.023ms avg)
rule  4: used in 0011633 sudokus (1.098ms avg)

rule 12: used in 0009708 sudokus (57.038ms avg)
rule 29: used in 0009017 sudokus (13.371ms avg)
rule 18: used in 0004359 sudokus (42.829ms avg)
rule 17: used in 0003112 sudokus (42.702ms avg)

rule 15: used in 0002514 sudokus (45.785ms avg)
rule 16: used in 0001804 sudokus (80.014ms avg)
rule  5: used in 0001477 sudokus (6.415ms avg)
rule 20: used in 0001435 sudokus (102.028ms avg)

rule 11: used in 0001084 sudokus (27.369ms avg)
rule  6: used in 0000173 sudokus (12.480ms avg)
rule 19: used in 0000087 sudokus (68.448ms avg)
rule  7: used in 0000014 sudokus (40.929ms avg)
rule 13: used in 0000000 sudokus (0ms avg)
...
*/

impl Sudoku {
    pub const RULES: &'static [(usize, SudokuDifficulty, SudokuRule)] = &[
        (29, Mandatory, Sudoku::bi_value_universal_grave),
        (35, Mandatory, Sudoku::avoidable_rectangle),
        (36, Unimplemented, Sudoku::unique_rectangle),
        (37, Unimplemented, Sudoku::hidden_unique_rectangle),
        (44, Unimplemented, Sudoku::exocet),
        (0, Easy, Sudoku::naked_singles),
        (1, Easy, Sudoku::hidden_singles),
        (8, Easy, Sudoku::pointing_pair),
        (2, Easy, Sudoku::naked_pairs),
        (3, Medium, Sudoku::naked_triples),
        (9, Medium, Sudoku::pointing_triple),
        (10, Medium, Sudoku::box_reduction),
        (4, Medium, Sudoku::hidden_pairs),
        (12, Hard, Sudoku::finned_x_wing),
        (18, Hard, Sudoku::w_wing),
        (17, Hard, Sudoku::y_wing),
        (15, Master, Sudoku::skyscraper),
        (16, Master, Sudoku::simple_coloring),
        (5, Master, Sudoku::hidden_triples),
        (20, Master, Sudoku::finned_swordfish),
        (11, Extreme, Sudoku::x_wing),
        (6, Extreme, Sudoku::naked_quads),
        (19, Extreme, Sudoku::swordfish),
        (7, Extreme, Sudoku::hidden_quads),
        (13, Extreme, Sudoku::franken_x_wing),
        // unimplemented rules
        (14, Unimplemented, Sudoku::finned_mutant_x_wing),
        (21, Unimplemented, Sudoku::sashimi_finned_swordfish),
        (22, Unimplemented, Sudoku::franken_swordfish),
        (23, Unimplemented, Sudoku::mutant_swordfish),
        (24, Unimplemented, Sudoku::finned_mutant_swordfish),
        (25, Unimplemented, Sudoku::sashimi_finned_mutant_swordfish),
        (26, Unimplemented, Sudoku::sue_de_coq),
        (27, Unimplemented, Sudoku::xyz_wing),
        (28, Unimplemented, Sudoku::x_cycle),
        (30, Unimplemented, Sudoku::xy_chain),
        (31, Unimplemented, Sudoku::three_d_medusa),
        (32, Unimplemented, Sudoku::jellyfish),
        (33, Unimplemented, Sudoku::finned_jellyfish),
        (34, Unimplemented, Sudoku::sashimi_finned_jellyfish),
        (38, Unimplemented, Sudoku::wxyz_wing),
        (39, Unimplemented, Sudoku::firework),
        (40, Unimplemented, Sudoku::subset_exclusion),
        (41, Unimplemented, Sudoku::empty_rectangle),
        (42, Unimplemented, Sudoku::sue_de_coq_extended),
        (43, Unimplemented, Sudoku::sk_loop),
        (45, Unimplemented, Sudoku::almost_locked_sets),
        (46, Unimplemented, Sudoku::alternating_inference_chain),
        (47, Unimplemented, Sudoku::digit_forcing_chains),
        (48, Unimplemented, Sudoku::nishio_forcing_chains),
        (49, Unimplemented, Sudoku::cell_forcing_chains),
        (50, Unimplemented, Sudoku::unit_forcing_chains),
        (51, Unimplemented, Sudoku::almost_locked_set_forcing_chain),
        (52, Unimplemented, Sudoku::death_blossom),
        (53, Unimplemented, Sudoku::pattern_overlay),
        (54, Unimplemented, Sudoku::bowmans_bingo),
    ];

    // RULES SOLVING
    // CHECK https://www.taupierbw.be/SudokuCoach
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT CHANGED SOMETHING

    // règle 0: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    pub(super) fn naked_singles(&mut self) -> bool {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() == 1 {
                    let &value = self.possibility_board[y][x].iter().next().unwrap();
                    self.set_value(x, y, value);
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return true;
                }
            }
        }
        false
    }

    // règle 1: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    pub(super) fn hidden_singles(&mut self) -> bool {
        for group in self.get_group(All) {
            for value in 1..=self.n2 {
                let cells_with_value: Vec<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if cells_with_value.len() == 1 {
                    let &&(x, y) = cells_with_value.first().unwrap();
                    self.set_value(x, y, value);
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return true;
                }
            }
        }
        false
    }

    // règle 2: http://www.taupierbw.be/SudokuCoach/SC_NakedPairs.shtml
    pub(super) fn naked_pairs(&mut self) -> bool {
        let mut modified = false;
        for group in self.get_group(All) {
            let pairs: Vec<&Coords> = group
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

    // règle 3: http://www.taupierbw.be/SudokuCoach/SC_NakedTriples.shtml
    pub(super) fn naked_triples(&mut self) -> bool {
        let mut modified = false;
        for group in self.get_group(All) {
            let pairs_or_triples: Vec<&Coords> = group
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

    // règle 4: http://www.taupierbw.be/SudokuCoach/SC_HiddenPairs.shtml
    pub(super) fn hidden_pairs(&mut self) -> bool {
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.len() != 2 {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
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

    // règle 5: http://www.taupierbw.be/SudokuCoach/SC_HiddenTriples.shtml
    pub(super) fn hidden_triples(&mut self) -> bool {
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in (value2 + 1)..=self.n2 {
                        let occurences_value3: HashSet<&Coords> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();

                        if occurences_value3.is_empty() {
                            continue;
                        }

                        let common_occurences: HashSet<&&Coords> = occurences_value1
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

    // règle 6: http://www.taupierbw.be/SudokuCoach/SC_NakedQuads.shtml
    pub(super) fn naked_quads(&mut self) -> bool {
        let mut modified = false;
        for group in self.get_group(All) {
            let pairs_or_triples_or_quads: Vec<&Coords> = group
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

    // règle 7: http://www.taupierbw.be/SudokuCoach/SC_HiddenQuads.shtml
    pub(super) fn hidden_quads(&mut self) -> bool {
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in (value2 + 1)..=self.n2 {
                        let occurences_value3: HashSet<&Coords> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();
                        if occurences_value3.is_empty() {
                            continue;
                        }
                        for value4 in (value3 + 1)..=self.n2 {
                            let occurences_value4: HashSet<&Coords> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value4))
                                .collect();
                            if occurences_value4.is_empty() {
                                continue;
                            }

                            let common_occurences: HashSet<&&Coords> = occurences_value1
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

    // règle 8: http://www.taupierbw.be/SudokuCoach/SC_PointingPair.shtml
    pub(super) fn pointing_pair(&mut self) -> bool {
        for square in self.get_group(Square) {
            for value in 1..=self.n2 {
                let occurences: Vec<&Coords> = square
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

    // règle 9: http://www.taupierbw.be/SudokuCoach/SC_PointingTriple.shtml
    pub(super) fn pointing_triple(&mut self) -> bool {
        for square in self.get_group(Square) {
            for value in 1..=self.n2 {
                let occurences: Vec<&Coords> = square
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

    // règle 10: http://www.taupierbw.be/SudokuCoach/SC_BoxReduction.shtml
    pub(super) fn box_reduction(&mut self) -> bool {
        let mut modified = false;
        for rows in self.get_group(Row) {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&Coords> = rows
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 && occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(x, _)| x / self.n == x1 / self.n) {
                    for (x, y) in self.get_cell_group(x1, y1, Square) {
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

        for cols in self.get_group(Column) {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&Coords> = cols
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 && occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(_, y)| y / self.n == y1 / self.n) {
                    for (x, y) in self.get_cell_group(x1, y1, Square) {
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

    // règle 11: http://www.taupierbw.be/SudokuCoach/SC_XWing.shtml
    pub(super) fn x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..(self.n2 - 1) {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let row1_pos = if row1_positions.len() == 2 {
                    let row1_vec: Vec<&usize> = row1_positions.iter().collect();
                    Some((row1_vec[0], row1_vec[1]))
                } else {
                    None
                };

                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                let col1_pos: Option<Coords> = if col1_positions.len() == 2 {
                    let col1_vec: Vec<&usize> = col1_positions.iter().collect();
                    Some((*col1_vec[0], *col1_vec[1]))
                } else {
                    None
                };

                for i2 in (i1 + 1)..self.n2 {
                    let mut picked_cells: Vec<(bool, Coords)> = Vec::new();

                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();

                    if row1_pos.is_some() && row2_positions == row1_positions {
                        let (x1, x2) = row1_pos.unwrap();

                        let col1 = self.get_cell_group(*x1, i1, Column);
                        let col2 = self.get_cell_group(*x2, i1, Column);
                        for &(x, y) in col1.union(&col2) {
                            if y == i1 || y == i2 {
                                continue;
                            }
                            picked_cells.push((true, (x, y)));
                        }
                    }

                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();

                    if col1_pos.is_some() && col1_positions == col2_positions {
                        let (y1, y2) = col1_pos.unwrap();

                        let row1 = self.get_cell_group(i1, y1, Row);
                        let row2 = self.get_cell_group(i1, y2, Row);
                        for &(x, y) in row1.union(&row2) {
                            if x == i1 || x == i2 {
                                continue;
                            }
                            picked_cells.push((false, (x, y)));
                        }
                    }

                    for (_is_row, (x, y)) in picked_cells {
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("{} {i1} and {i2}: possibilitée {value} supprimée de x: {x}, y: {y}", if _is_row {"rows"} else {"cols"});
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

    // règle 12: http://www.taupierbw.be/SudokuCoach/SC_FinnedXWing.shtml
    pub(super) fn finned_x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..(self.n2 - 1) {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();

                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();

                for i2 in (i1 + 1)..self.n2 {
                    if i1 / self.n == i2 / self.n {
                        continue;
                    }
                    let mut picked_cells: Vec<(bool, Coords, Coords)> = Vec::new();

                    let row2_positions: HashSet<usize> = (0..self.n2)
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
                            picked_cells.push((true, (x1, fin_i), (*fin, fin_i)));
                        } else if fin / self.n == x2 / self.n {
                            picked_cells.push((true, (x2, fin_i), (*fin, fin_i)));
                        }
                    }

                    let col2_positions: HashSet<usize> = (0..self.n2)
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
                            picked_cells.push((false, (fin_i, y1), (fin_i, *fin)));
                        } else if fin / self.n == y2 / self.n {
                            picked_cells.push((false, (fin_i, y2), (fin_i, *fin)));
                        }
                    }

                    for (_is_row, (x1, y1), (fin_x, fin_y)) in picked_cells {
                        let removed_cells: Vec<Coords> = self
                            .get_cell_group(fin_x, fin_y, Square)
                            .into_iter()
                            .filter(|(x, y)| {
                                (y1 == fin_y && *x == x1 && *y != y1)
                                    || (x1 == fin_x && *x != x1 && *y == y1)
                            })
                            .collect();
                        for (x, y) in removed_cells {
                            if self.possibility_board[y][x].remove(&value) {
                                debug_only!("{} {i1} and {i2}: possibilitée {value} supprimée de x: {x}, y: {y}", if _is_row {"rows"} else {"cols"});
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

    // règle 13: http://www.taupierbw.be/SudokuCoach/SC_FrankenXWing.shtml
    pub(super) fn franken_x_wing(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            for line in self.get_group(Lines) {
                let occurences: Vec<&Coords> = line
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

                for square in self.get_group(Square) {
                    if !line.is_disjoint(&square) {
                        continue;
                    }

                    let (mut yellow_cells1, mut yellow_cells2): (HashSet<Coords>, HashSet<Coords>) =
                        if y1 == y2 {
                            (
                                self.get_cell_group(x1, y1, Column).clone(),
                                self.get_cell_group(x2, y2, Column).clone(),
                            )
                        } else {
                            (
                                self.get_cell_group(x1, y1, Row).clone(),
                                self.get_cell_group(x2, y2, Row).clone(),
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
                        .chain(yellow_cells2.difference(&square))
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

    // règle 14: https://www.taupierbw.be/SudokuCoach/SC_FinnedMutantXWing.shtml
    pub(super) fn finned_mutant_x_wing(&mut self) -> bool {
        warn!("finned_mutant_x_wing not yet implemented");
        false
    }

    // règle 15: http://www.taupierbw.be/SudokuCoach/SC_Skyscraper.shtml
    pub(super) fn skyscraper(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..(self.n2 - 1) {
                let row1_positions: Vec<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let row1_pos = if row1_positions.len() == 2 {
                    Some((row1_positions[0], row1_positions[1]))
                } else {
                    None
                };

                let col1_positions: Vec<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                let col1_pos = if col1_positions.len() == 2 {
                    Some((col1_positions[0], col1_positions[1]))
                } else {
                    None
                };

                for i2 in (i1 + 1)..self.n2 {
                    // i1 and i2 represents rows or columns
                    let mut picked_cells: Vec<(bool, Coords, Coords)> = Vec::new();

                    let row2_positions: Vec<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    if row1_pos.is_some() && row2_positions.len() == 2 {
                        let (x11, x12) = row1_pos.unwrap();
                        let x21 = row2_positions[0];
                        let x22 = row2_positions[1];
                        if x11 == x21 || x12 == x22 {
                            let (x1, x2) = if x11 == x21 { (x12, x22) } else { (x11, x21) };
                            picked_cells.push((true, (x1, i1), (x2, i2)));
                        }
                    }

                    let col2_positions: Vec<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    if col1_pos.is_some() && col2_positions.len() == 2 {
                        let (y11, y12) = col1_pos.unwrap();
                        let y21 = col2_positions[0];
                        let y22 = col2_positions[1];
                        if y11 == y21 || y12 == y22 {
                            let (y1, y2) = if y11 == y21 { (y12, y22) } else { (y11, y21) };
                            picked_cells.push((false, (i1, y1), (i2, y2)));
                        }
                    }

                    for (_is_row, (x1, y1), (x2, y2)) in picked_cells {
                        let cell_group1: HashSet<Coords> = self.get_cell_group(x1, y1, All);
                        let cell_group2: HashSet<Coords> = self.get_cell_group(x2, y2, All);
                        let common_cells: HashSet<&Coords> =
                            cell_group1.intersection(&cell_group2).collect();

                        for &(x, y) in common_cells {
                            if (x == x1 && y == y1) || (x == x2 && y == y2) {
                                continue;
                            }

                            if self.possibility_board[y][x].remove(&value) {
                                debug_only!("{} {i1} and {i2}: {x1},{y1} et {x2},{y2}: possibilitée {value} supprimée de x: {x}, y: {y}", if _is_row {"rows"} else {"cols"});
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

    // règle 16: http://www.taupierbw.be/SudokuCoach/SC_SimpleColoring.shtml
    pub(super) fn simple_coloring(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            let mut chains: Vec<Vec<usize>> = Vec::new(); // ne contient pas les (x,y) mais y*n+x (plus simple a traiter)
            let strong_links = self
                .get_strong_links(value)
                .into_iter()
                .map(|((x1, y1), (x2, y2))| (y1 * self.n2 + x1, y2 * self.n2 + x2))
                .collect::<Vec<_>>();
            let graph: UndirectedCsrGraph<usize> = GraphBuilder::new()
                .csr_layout(graph::prelude::CsrLayout::Unsorted)
                .edges(strong_links)
                .build();

            for node in 0..graph.node_count() {
                let mut visited: HashSet<usize> = HashSet::new();
                let mut stack = vec![(node, vec![])];

                while let Some((current, mut path)) = stack.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    visited.insert(current);
                    path.push(current);

                    for &neighbor in graph.neighbors(current) {
                        if !visited.contains(&neighbor) {
                            stack.push((neighbor, path.clone()));
                        } else if path.contains(&neighbor) {
                            chains.push(path.clone());
                        }
                    }
                }
            }

            let mut chains_hashset: Vec<(Vec<usize>, HashSet<usize>)> = chains
                .into_iter()
                .map(|chain| (chain.clone(), chain.into_iter().collect::<HashSet<usize>>()))
                .collect();
            chains_hashset.sort_by(|(chain1, _), (chain2, _)| chain2.len().cmp(&chain1.len()));

            let mut keeped_hashsets: Vec<&HashSet<usize>> = Vec::new();
            let mut keeped_chains: Vec<Vec<Coords>> = Vec::new();

            for (chain1, hash1) in chains_hashset.iter() {
                if chain1.len() < 3 || keeped_hashsets.contains(&hash1) {
                    continue;
                }
                let mut keep = true;
                for hash2 in keeped_hashsets.iter() {
                    if hash2.is_superset(hash1) {
                        keep = false;
                    }
                }
                if keep {
                    keeped_hashsets.push(hash1);
                    keeped_chains.push(
                        chain1
                            .iter()
                            .map(|cell_id| (cell_id % self.n2, cell_id / self.n2))
                            .collect(),
                    );
                }
            }

            for chain in keeped_chains {
                let chain_len = chain.len();
                let &(x1, y1) = chain.first().unwrap();
                let &(x2, y2) = chain.get(chain_len - 1).unwrap();
                if chain_len % 2 == 0 {
                    let cell_group1: HashSet<Coords> = self.get_cell_group(x1, y1, All);
                    let cell_group2: HashSet<Coords> = self.get_cell_group(x2, y2, All);
                    let common_cells: HashSet<&Coords> =
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
                } else if self.is_same_group(x1, y1, x2, y2) {
                    for &(x, y) in chain.iter().step_by(2) {
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
                    (temp[0], temp[1])
                };
                let cell_groups: HashSet<Coords> = self.get_cell_group(x, y, All);

                let b1_values = cell_groups.iter().filter(|(x1, y1)| {
                    let possibilities = &self.possibility_board[*y1][*x1];
                    possibilities.len() == 2
                        && possibilities.contains(value1)
                        && !possibilities.contains(value2)
                });

                let b2_values: Vec<&Coords> = cell_groups
                    .iter()
                    .filter(|(x2, y2)| {
                        let possibilities = &self.possibility_board[*y2][*x2];
                        possibilities.len() == 2
                            && possibilities.contains(value2)
                            && !possibilities.contains(value1)
                    })
                    .collect();

                let mut bi_values: Vec<(usize, Coords, Coords)> = Vec::new();
                for (x1, y1) in b1_values {
                    for (x2, y2) in b2_values.iter() {
                        let possible_value3: Option<&usize> = self.possibility_board[*y1][*x1]
                            .intersection(&self.possibility_board[*y2][*x2])
                            .next();
                        if let Some(value3) = possible_value3 {
                            bi_values.push((*value3, (*x1, *y1), (*x2, *y2)));
                        }
                    }
                }

                for (value, (x1, y1), (x2, y2)) in bi_values {
                    let cell_group1: HashSet<Coords> = self.get_cell_group(x1, y1, All);
                    let cell_group2: HashSet<Coords> = self.get_cell_group(x2, y2, All);
                    let common_cells: HashSet<&Coords> =
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
                let cell_groups: HashSet<Coords> = self.get_cell_group(x, y, All);

                let possible_values: Vec<Coords> = {
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

                        let cell1_groups = self.get_cell_groups(x1, y1, vec![Row, Column, Square]);
                        for group in cell1_groups {
                            if group.contains(&(x, y)) {
                                continue;
                            }
                            let strong_link: HashSet<Coords> = group
                                .into_iter()
                                .filter(|&(x2, y2)| {
                                    (x2 != x1 || y2 != y1)
                                        && self.possibility_board[y2][x2].contains(&value1)
                                })
                                .collect();
                            if strong_link.len() != 1 {
                                continue;
                            }
                            let (x2, y2) = strong_link.into_iter().next().unwrap();

                            let mut picked_cells: Vec<Coords> = Vec::new();

                            let cell2_groups =
                                self.get_cell_groups(x2, y2, vec![Row, Column, Square]);
                            for group in cell2_groups {
                                if group.contains(&(x1, y1)) {
                                    continue;
                                }

                                for (x3, y3) in group {
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
                                        "({x},{y}) ({x1},{y1}) ({x2},{y2}) ({x3},{y3}) possibilitée {value2} supprimée de x: {x3}, y: {y3}"
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
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..(self.n2 - 1) {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                for i2 in (i1 + 1)..self.n2 {
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    for i3 in (i2 + 1)..self.n2 {
                        let row3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|x| self.possibility_board[i3][*x].contains(&value))
                            .collect();
                        let col3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|y| self.possibility_board[*y][i3].contains(&value))
                            .collect();

                        // i1, i2 and i3 represents rows or columns
                        let mut picked_cells: Vec<(bool, usize, usize, usize)> = Vec::new();

                        if (row1_positions.len() == 3 || row1_positions.len() == 2)
                            && (row2_positions.len() == 3 || row2_positions.len() == 2)
                            && (row3_positions.len() == 3 || row3_positions.len() == 2)
                        {
                            let total_positions: HashSet<usize> = row1_positions
                                .union(&row2_positions)
                                .chain(&row3_positions)
                                .cloned()
                                .collect();
                            if total_positions.len() == 3 {
                                let val: Vec<usize> = total_positions.into_iter().collect();
                                picked_cells.push((true, val[0], val[1], val[2]));
                            }
                        }

                        if (col1_positions.len() == 3 || col1_positions.len() == 2)
                            && (col2_positions.len() == 3 || col2_positions.len() == 2)
                            && (col3_positions.len() == 3 || col3_positions.len() == 2)
                        {
                            let total_positions: HashSet<usize> = col1_positions
                                .union(&col2_positions)
                                .chain(&col3_positions)
                                .cloned()
                                .collect();
                            if total_positions.len() == 3 {
                                let val: Vec<usize> = total_positions.into_iter().collect();
                                picked_cells.push((false, val[0], val[1], val[2]));
                            }
                        }

                        for (_is_row, j1, j2, j3) in picked_cells {
                            let mut common_cells: HashSet<&Coords>;
                            let cell_groupe1: HashSet<Coords>;
                            let cell_groupe2: HashSet<Coords>;
                            let cell_groupe3: HashSet<Coords>;
                            if _is_row {
                                cell_groupe1 = self.get_cell_group(j1, i1, Column);
                                cell_groupe2 = self.get_cell_group(j2, i2, Column);
                                cell_groupe3 = self.get_cell_group(j3, i3, Column);
                                common_cells = cell_groupe1
                                    .union(&cell_groupe2)
                                    .chain(&cell_groupe3)
                                    .collect();
                                common_cells.retain(|&&(_, y)| y != i1 && y != i2 && y != i3);
                            } else {
                                cell_groupe1 = self.get_cell_group(i1, j1, Row);
                                cell_groupe2 = self.get_cell_group(i2, j2, Row);
                                cell_groupe3 = self.get_cell_group(i3, j3, Row);
                                common_cells = cell_groupe1
                                    .union(&cell_groupe2)
                                    .chain(&cell_groupe3)
                                    .collect();
                                common_cells.retain(|&&(x, _)| x != i1 && x != i2 && x != i3);
                            }

                            for &(x, y) in common_cells {
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
            }
        }
        false
    }

    // règle 20: http://www.taupierbw.be/SudokuCoach/SC_FinnedSwordfish.shtml
    pub(super) fn finned_swordfish(&mut self) -> bool {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..(self.n2 - 1) {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                for i2 in (i1 + 1)..self.n2 {
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    for i3 in (i2 + 1)..self.n2 {
                        let row3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|x| self.possibility_board[i3][*x].contains(&value))
                            .collect();
                        let col3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|y| self.possibility_board[*y][i3].contains(&value))
                            .collect();

                        // i1, i2 and i3 represents rows or columns
                        let mut picked_cells: Vec<(bool, Coords)> = Vec::new();

                        if (2 <= row1_positions.len() && row1_positions.len() <= 3)
                            && (2 <= row2_positions.len() && row2_positions.len() <= 3)
                            && (2 <= row3_positions.len() && row3_positions.len() <= 3)
                        {
                            let total_positions: HashSet<usize> = row1_positions
                                .union(&row2_positions)
                                .chain(&row3_positions)
                                .cloned()
                                .collect();
                            if total_positions.len() == 4 {
                                let mut potential_fins = Vec::new();
                                for &x in total_positions.iter() {
                                    let contained_y: Vec<usize> = vec![i1, i2, i3]
                                        .into_iter()
                                        .filter(|&y| self.possibility_board[y][x].contains(&value))
                                        .collect();
                                    if contained_y.len() == 1 {
                                        potential_fins.push((x, contained_y[0]));
                                    }
                                }

                                if potential_fins.len() == 1 {
                                    let (fin_x, fin_y) = potential_fins[0];
                                    for x in total_positions.into_iter() {
                                        if x != fin_x
                                            && x / self.n == fin_x / self.n
                                            && self.possibility_board[fin_y][x].contains(&value)
                                        {
                                            debug_only!("rows i1:{i1}, i2:{i2}, i3:{i3}: fin:{fin_x},{fin_y} picked:{x},{fin_y}");
                                            picked_cells.push((true, (x, fin_y)));
                                        }
                                    }
                                }
                            }
                        }

                        if (2 <= col1_positions.len() && col1_positions.len() <= 3)
                            && (2 <= col2_positions.len() && col2_positions.len() <= 3)
                            && (2 <= col3_positions.len() && col3_positions.len() <= 3)
                        {
                            let total_positions: HashSet<usize> = col1_positions
                                .union(&col2_positions)
                                .chain(&col3_positions)
                                .cloned()
                                .collect();
                            if total_positions.len() == 4 {
                                let mut potential_fins = Vec::new();
                                for &y in total_positions.iter() {
                                    let contained_x: Vec<usize> = vec![i1, i2, i3]
                                        .into_iter()
                                        .filter(|&x| self.possibility_board[y][x].contains(&value))
                                        .collect();
                                    if contained_x.len() == 1 {
                                        potential_fins.push((contained_x[0], y));
                                    }
                                }

                                if potential_fins.len() == 1 {
                                    let (fin_x, fin_y) = potential_fins[0];
                                    for y in total_positions.into_iter() {
                                        if y != fin_y
                                            && y / self.n == fin_y / self.n
                                            && self.possibility_board[y][fin_x].contains(&value)
                                        {
                                            debug_only!("cols i1:{i1}, i2:{i2}, i3:{i3}: fin:{fin_x},{fin_y} picked:{fin_x},{y}");
                                            picked_cells.push((false, (y, fin_x)));
                                        }
                                    }
                                }
                            }
                        }

                        for (is_row, data) in picked_cells {
                            if is_row {
                                let (finned_cell_x, finned_cell_y) = data;
                                let square_y = finned_cell_y - finned_cell_y % self.n;
                                for dy in 0..self.n {
                                    let y = square_y + dy;
                                    if y == i1 || y == i2 || y == i3 {
                                        continue;
                                    }
                                    if self.possibility_board[y][finned_cell_x].remove(&value) {
                                        debug_only!("possibilitée {value} supprimée de x: {finned_cell_x}, y: {y}");
                                        modified = true;
                                    }
                                }
                            } else {
                                let (finned_cell_y, finned_cell_x) = data;
                                let square_x = finned_cell_x - finned_cell_x % self.n;
                                for dx in 0..self.n {
                                    let x = square_x + dx;
                                    if x == i1 || x == i2 || x == i3 {
                                        continue;
                                    }
                                    if self.possibility_board[finned_cell_y][x].remove(&value) {
                                        debug_only!("possibilitée {value} supprimée de x: {x}, y: {finned_cell_y}");
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

    // règle 21: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedSwordfish.shtml
    pub(super) fn sashimi_finned_swordfish(&mut self) -> bool {
        warn!("sashimi_finned_swordfish isn't implemented yet");
        false
    }

    // règle 22: https://www.taupierbw.be/SudokuCoach/SC_FrankenSwordfish.shtml
    pub(super) fn franken_swordfish(&mut self) -> bool {
        warn!("franken_swordfish not yet implemented");
        false
    }

    // règle 23: https://www.taupierbw.be/SudokuCoach/SC_MutantSwordfish.shtml
    pub(super) fn mutant_swordfish(&mut self) -> bool {
        warn!("mutant_swordfish not yet implemented");
        false
    }

    // règle 24: https://www.taupierbw.be/SudokuCoach/SC_FinnedMutantSwordfish.shtml
    pub(super) fn finned_mutant_swordfish(&mut self) -> bool {
        warn!("finned_mutant_swordfish not yet implemented");
        false
    }

    // règle 25: https://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedMutantSwordfish.shtml
    pub(super) fn sashimi_finned_mutant_swordfish(&mut self) -> bool {
        warn!("sashimi_finned_mutant_swordfish not yet implemented");
        false
    }

    // règle 26: https://www.taupierbw.be/SudokuCoach/SC_Suedecoq.shtml
    pub(super) fn sue_de_coq(&mut self) -> bool {
        warn!("sue_de_coq not yet implemented");
        false
    }

    // règle 27: http://www.taupierbw.be/SudokuCoach/SC_XYZWing.shtml
    pub(super) fn xyz_wing(&mut self) -> bool {
        warn!("xyz_wing isn't implemented yet");
        false
    }

    // règle 28: https://www.taupierbw.be/SudokuCoach/SC_XCycle.shtml
    pub(super) fn x_cycle(&mut self) -> bool {
        warn!("x_cycle not yet implemented");
        false
    }

    // règle 29: http://www.taupierbw.be/SudokuCoach/SC_BUG.shtml
    pub(super) fn bi_value_universal_grave(&mut self) -> bool {
        let mut unique_triple: Option<Coords> = None;
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
        for value in self.possibility_board[y0][x0].iter() {
            if self
                .get_cell_groups(x0, y0, vec![Row, Column, Square])
                .iter()
                .all(|group| {
                    group
                        .iter()
                        .filter(|(x, y)| self.possibility_board[*y][*x].contains(value))
                        .count()
                        == 2
                })
            {
                debug_only!("valeur {} fixée en x: {}, y: {}", value, x0, y0);
                self.set_value(x0, y0, *value);
                return true;
            }
        }
        false
    }

    // règle 30: http://www.taupierbw.be/SudokuCoach/SC_XYChain.shtml
    pub(super) fn xy_chain(&mut self) -> bool {
        warn!("xy_chain isn't implemented yet");
        false
    }

    // règle 31: https://www.taupierbw.be/SudokuCoach/SC_Medusa.shtml
    pub(super) fn three_d_medusa(&mut self) -> bool {
        warn!("three_d_medusa not yet implemented");
        false
    }

    // règle 32: http://www.taupierbw.be/SudokuCoach/SC_Jellyfish.shtml
    pub(super) fn jellyfish(&mut self) -> bool {
        warn!("jellyfish isn't implemented yet");
        false
    }

    // règle 33: http://www.taupierbw.be/SudokuCoach/SC_FinnedJellyfish.shtml
    pub(super) fn finned_jellyfish(&mut self) -> bool {
        warn!("finned_jellyfish isn't implemented yet");
        false
    }

    // règle 34: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedJellyfish.shtml
    pub(super) fn sashimi_finned_jellyfish(&mut self) -> bool {
        warn!("sashimi_finned_jellyfish isn't implemented yet");
        false
    }

    // règle 35: https://www.taupierbw.be/SudokuCoach/SC_AvoidableRectangle.shtml
    pub(super) fn avoidable_rectangle(&mut self) -> bool {
        let mut modified = false;
        for y0 in 0..self.n2 {
            for x0 in 0..self.n2 {
                for y1 in (y0 + 1)..self.n2 {
                    for x1 in (x0 + 1)..self.n2 {
                        let rectangle = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];

                        let values = rectangle
                            .iter()
                            .filter_map(|(x, y)| {
                                if self.board[*y][*x] != 0 {
                                    Some(self.board[*y][*x])
                                } else {
                                    None
                                }
                            })
                            .collect::<HashSet<_>>();
                        if values.len() != 2 {
                            continue;
                        }
                        let (val1, val2) = {
                            let mut values_iter = values.into_iter();
                            (values_iter.next().unwrap(), values_iter.next().unwrap())
                        };

                        let mut empty_cells = rectangle
                            .iter()
                            .filter(|&&(x, y)| self.board[y][x] == 0)
                            .collect::<Vec<_>>();

                        if empty_cells.len() == 1 {
                            // TYPE 1
                            let &(x, y) = empty_cells.pop().unwrap();

                            if self.possibility_board[y][x].remove(&val1)
                                || self.possibility_board[y][x].remove(&val2)
                            {
                                return true;
                            }
                        } else if empty_cells.len() == 2 {
                            let &(x2, y2) = empty_cells.pop().unwrap();
                            let &(x3, y3) = empty_cells.pop().unwrap();
                            if x2 != x3 && y2 != y3 {
                                continue;
                            }

                            if self.possibility_board[y2][x2].len() != 2
                                || self.possibility_board[y3][x3].len() != 2
                            {
                                continue;
                            }

                            let mut possibilities = self.possibility_board[y2][x2]
                                .union(&self.possibility_board[y3][x3])
                                .cloned()
                                .collect::<HashSet<_>>();
                            if possibilities.len() != 3 {
                                continue;
                            }

                            possibilities.remove(&val1);
                            possibilities.remove(&val2);

                            if possibilities.len() != 1 {
                                continue;
                            }
                            let common_possibility = possibilities.into_iter().next().unwrap();

                            if x2 == x3 {
                                for y in 0..self.n2 {
                                    if y == y2 || y == y3 {
                                        continue;
                                    }
                                    if self.possibility_board[y][x2].remove(&common_possibility) {
                                        modified = true;
                                    }
                                }
                            } else {
                                for x in 0..self.n2 {
                                    if x == x2 || x == x3 {
                                        continue;
                                    }
                                    if self.possibility_board[y2][x].remove(&common_possibility) {
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

    // règle 36: https://www.taupierbw.be/SudokuCoach/SC_UniqueRectangle.shtml
    pub(super) fn unique_rectangle(&mut self) -> bool {
        let mut modified = false;
        for y0 in 0..self.n2 {
            for x0 in 0..self.n2 {
                for y1 in (y0 + 1)..self.n2 {
                    for x1 in (x0 + 1)..self.n2 {
                        //For every rectangle possible
                        let rectangle = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];

                        //Get the different possibilities of the rectangle corner
                        let possval = rectangle
                            .iter()
                            .filter_map(|(x, y)| {
                                let poss = &self.possibility_board[*y][*x];
                                if poss.len() != 0 {
                                    Some(poss.into_iter().cloned().collect::<Vec<usize>>())
                                } else {
                                    None
                                }
                            })
                            .collect::<HashSet<Vec<usize>>>();

                        if possval.len() == 2 {
                            //Type 1 & 2
                            let (val1, val2) = {
                                let mut values_iter = possval.into_iter();
                                (values_iter.next().unwrap().into_iter().collect::<HashSet<usize>>(), values_iter.next().unwrap().into_iter().collect::<HashSet<usize>>())
                            };
                            //Get the corners with same bi-values
                            let bi_cell = rectangle
                                .iter()
                                .filter(|(x, y)| self.possibility_board[*y][*x].len() == 2)
                                .collect::<HashSet<_>>();
                            if bi_cell.len() == 3 {
                                //Type 1
                                //get the other corner
                                let rectangle_refs: HashSet<_> = rectangle.iter().collect();
                                let mut last_cell = rectangle_refs
                                    .difference(&bi_cell)
                                    .collect::<Vec<&&(usize, usize)>>();
                                let (x2, y2) = last_cell.pop().unwrap();
                                //remove the bi-value possibilities from the other corner
                                if val1.len() == 2 {
                                    for val in val1 {
                                        if self.possibility_board[*y2][*x2].remove(&val) {
                                            debug_only!("possibilitée {val} supprimée de x: {x2}, y: {y2}");
                                            modified = true;
                                        }
                                    }
                                } else if val2.len() == 2 {
                                    for val in val2 {
                                        if self.possibility_board[*y2][*x2].remove(&val) {
                                            debug_only!("possibilitée {val} supprimée de x: {x2}, y: {y2}");
                                            modified = true;
                                        }
                                    }
                                }
                            } else if bi_cell.len() == 2 {
                                //Type 2
                                //get 2 othercell
                                let mut other_cells = rectangle
                                    .iter()
                                    .filter(|(x, y)| self.possibility_board[*y][*x].len() == 3)
                                    .collect::<Vec<_>>();
                                if other_cells.len() != 2 {
                                    continue;
                                }
                                //get the cells that sees both three-value-cells
                                let &(x2, y2) = other_cells.pop().unwrap();
                                let &(x3, y3) = other_cells.pop().unwrap();
                                let group1 = self.get_cell_group(x2, y2, All);
                                let group2 = self.get_cell_group(x3, y3, All);
                                let see_three_val = group1.intersection(&group2).collect::<HashSet<_>>();
                                //get the extra value of the three-value compared to the bi-value
                                let xtraval = if val1.len() == 2 {
                                    val2.difference(&val1).next().unwrap()
                                } else {
                                    val1.difference(&val2).next().unwrap()
                                };
                                //remove the extra value from the cells that sees both three-value-cells
                                for &(x4,y4) in see_three_val{
                                    if (x4 == x2 && y4 == y2) ||(x4 == x3 && y4 == y3) {
                                        continue;
                                    }
                                    else{
                                        if self.possibility_board[y4][x4].remove(xtraval){
                                            debug_only!("possibilitée {xtraval} supprimée de x: {x4}, y: {y4}");
                                            modified = true;
                                        }
                                    }
                                }
                            }
                        } else if possval.len() == 3 {
                            //Type 3
                            continue;
                        } else {
                            continue;
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

    // règle 37: https://www.taupierbw.be/SudokuCoach/SC_HiddenUniqueRectangle.shtml
    pub(super) fn hidden_unique_rectangle(&mut self) -> bool {
        warn!("hidden_unique_rectangle not yet implemented");
        false
    }

    // règle 38: http://www.taupierbw.be/SudokuCoach/SC_WXYZWing.shtml
    pub(super) fn wxyz_wing(&mut self) -> bool {
        warn!("wxyz_wing isn't implemented yet");
        false
    }

    // règle 39: https://www.taupierbw.be/SudokuCoach/SC_Firework.shtml
    pub(super) fn firework(&mut self) -> bool {
        warn!("firework not yet implemented");
        false
    }

    // règle 40: http://www.taupierbw.be/SudokuCoach/SC_APE.shtml
    pub(super) fn subset_exclusion(&mut self) -> bool {
        warn!("subset_exclusion isn't implemented yet");
        false
    }

    // règle 41: http://www.taupierbw.be/SudokuCoach/SC_EmptyRectangle.shtml
    pub(super) fn empty_rectangle(&mut self) -> bool {
        warn!("empty_rectangle isn't implemented yet");
        false
    }

    // règle 42: https://www.taupierbw.be/SudokuCoach/SC_SuedecoqExtended.shtml
    pub(super) fn sue_de_coq_extended(&mut self) -> bool {
        warn!("sue_de_coq_extended not yet implemented");
        false
    }

    // règle 43: https://www.taupierbw.be/SudokuCoach/SC_SKLoop.shtml
    pub(super) fn sk_loop(&mut self) -> bool {
        warn!("sk_loop not yet implemented");
        false
    }

    // règle 44: https://www.taupierbw.be/SudokuCoach/SC_Exocet.shtml
    pub(super) fn exocet(&mut self) -> bool {
        warn!("exocet not yet implemented");
        false
    }

    // règle 45: https://www.taupierbw.be/SudokuCoach/SC_ALS.shtml
    pub(super) fn almost_locked_sets(&mut self) -> bool {
        warn!("almost_locked_sets not yet implemented");
        false
    }

    // règle 46: https://www.taupierbw.be/SudokuCoach/SC_AIC.shtml
    pub(super) fn alternating_inference_chain(&mut self) -> bool {
        warn!("alternating_inference_chain not yet implemented");
        false
    }

    // règle 47: https://www.taupierbw.be/SudokuCoach/SC_DigitForcingChains.shtml
    pub(super) fn digit_forcing_chains(&mut self) -> bool {
        warn!("digit_forcing_chains not yet implemented");
        false
    }

    // règle 48: https://www.taupierbw.be/SudokuCoach/SC_NishioForcingChains.shtml
    pub(super) fn nishio_forcing_chains(&mut self) -> bool {
        warn!("nishio_forcing_chains not yet implemented");
        false
    }

    // règle 49: https://www.taupierbw.be/SudokuCoach/SC_CellForcingChains.shtml
    pub(super) fn cell_forcing_chains(&mut self) -> bool {
        warn!("cell_forcing_chains not yet implemented");
        false
    }

    // règle 50: https://www.taupierbw.be/SudokuCoach/SC_UnitForcingChains.shtml
    pub(super) fn unit_forcing_chains(&mut self) -> bool {
        warn!("unit_forcing_chains not yet implemented");
        false
    }

    // règle 51: http://www.taupierbw.be/SudokuCoach/SC_ALSchain.shtml
    pub(super) fn almost_locked_set_forcing_chain(&mut self) -> bool {
        warn!("almost_locked_set_forcing_chain isn't implemented yet");
        false
    }

    // règle 52: http://www.taupierbw.be/SudokuCoach/SC_DeathBlossom.shtml
    pub(super) fn death_blossom(&mut self) -> bool {
        warn!("death_blossom isn't implemented yet");
        false
    }

    // règle 53: http://www.taupierbw.be/SudokuCoach/SC_PatternOverlay.shtml
    pub(super) fn pattern_overlay(&mut self) -> bool {
        warn!("pattern_overlay isn't implemented yet");
        false
    }

    // règle 54: http://www.taupierbw.be/SudokuCoach/SC_BowmanBingo.shtml
    pub(super) fn bowmans_bingo(&mut self) -> bool {
        warn!("bowmans_bingo isn't implemented yet");
        false
    }
}
