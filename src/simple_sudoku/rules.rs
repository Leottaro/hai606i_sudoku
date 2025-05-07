use log::warn;
use std::collections::{HashMap, HashSet};

use super::{
    Coords, Sudoku,
    SudokuDifficulty::{self, *},
    SudokuError,
    SudokuGroups::*,
    SudokuRule,
};
use crate::debug_only;

impl Sudoku {
    pub const RULES: &'static [(usize, SudokuDifficulty, SudokuRule)] = &[
        (0, Easy, Sudoku::naked_singles),
        (1, Easy, Sudoku::hidden_singles),
        (2, Easy, Sudoku::naked_pairs),
        (3, Easy, Sudoku::naked_triples),
        (4, Medium, Sudoku::hidden_pairs),
        (5, Medium, Sudoku::hidden_triples),
        (8, Medium, Sudoku::pointing_pair),
        (9, Medium, Sudoku::pointing_triple),
        (10, Medium, Sudoku::box_reduction),
        (6, Medium, Sudoku::naked_quads),
        (7, Hard, Sudoku::hidden_quads),
        (11, Hard, Sudoku::x_wing),
        (12, Hard, Sudoku::finned_x_wing),
        (19, Hard, Sudoku::swordfish),
        (15, Master, Sudoku::skyscraper),
        (17, Master, Sudoku::y_wing),
        (16, Master, Sudoku::simple_coloring),
        (18, Extreme, Sudoku::w_wing),
        (13, Extreme, Sudoku::franken_x_wing),
        (20, Extreme, Sudoku::finned_swordfish),
        // Useless rules
        (29, Useless, Sudoku::bi_value_universal_grave),
        (35, Useless, Sudoku::avoidable_rectangle),
        (36, Useless, Sudoku::unique_rectangle),
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
        (37, Unimplemented, Sudoku::hidden_unique_rectangle),
        (38, Unimplemented, Sudoku::wxyz_wing),
        (39, Unimplemented, Sudoku::firework),
        (40, Unimplemented, Sudoku::subset_exclusion),
        (41, Unimplemented, Sudoku::empty_rectangle),
        (42, Unimplemented, Sudoku::sue_de_coq_extended),
        (43, Unimplemented, Sudoku::sk_loop),
        (44, Unimplemented, Sudoku::exocet),
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

    pub const fn get_rule_name_by_id(id: usize) -> &'static str {
        match id {
            0 => "Naked Singles",
            1 => "Hidden Singles",
            2 => "Naked Pairs",
            3 => "Naked Triples",
            4 => "Hidden Pairs",
            5 => "Hidden Triples",
            6 => "Naked Quads",
            7 => "Hidden Quads",
            8 => "Pointing Pair",
            9 => "Pointing Triple",
            10 => "Box Reduction",
            11 => "X-Wing",
            12 => "Finned X-Wing",
            13 => "Franken X-Wing",
            14 => "Finned Mutant X-Wing",
            15 => "Skyscraper",
            16 => "Simple Coloring",
            17 => "Y-Wing",
            18 => "W-Wing",
            19 => "Swordfish",
            20 => "Finned Swordfish",
            21 => "Sashimi Finned Swordfish",
            22 => "Franken Swordfish",
            23 => "Mutant Swordfish",
            24 => "Finned Mutant Swordfish",
            25 => "Sashimi Finned Mutant Swordfish",
            26 => "Sue de Coq",
            27 => "XYZ-Wing",
            28 => "X-Cycle",
            29 => "Bi-Value Universal Grave",
            30 => "XY-Chain",
            31 => "3D Medusa",
            32 => "Jellyfish",
            33 => "Finned Jellyfish",
            34 => "Sashimi Finned Jellyfish",
            35 => "Avoidable Rectangle",
            36 => "Unique Rectangle",
            37 => "Hidden Unique Rectangle",
            38 => "WXYZ-Wing",
            39 => "Firework",
            40 => "Subset Exclusion",
            41 => "Empty Rectangle",
            42 => "Sue de Coq Extended",
            43 => "SK Loop",
            44 => "Exocet",
            45 => "Almost Locked Sets",
            46 => "Alternating Inference Chain",
            47 => "Digit Forcing Chains",
            48 => "Nishio Forcing Chains",
            49 => "Cell Forcing Chains",
            50 => "Unit Forcing Chains",
            51 => "Almost Locked Set Forcing Chain",
            52 => "Death Blossom",
            53 => "Pattern Overlay",
            54 => "Bowman's Bingo",
            _ => "Unknown Rule",
        }
    }

    // RULES SOLVING
    // CHECK https://www.taupierbw.be/SudokuCoach
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT CHANGED SOMETHING

    // règle 0: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    fn naked_singles(&mut self) -> Result<bool, SudokuError> {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() == 1 {
                    let &value = self.possibility_board[y][x].iter().next().unwrap();
                    self.set_value(x, y, value)?;
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // règle 1: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    fn hidden_singles(&mut self) -> Result<bool, SudokuError> {
        for group in self.get_group(All) {
            for value in 1..=self.n2 {
                let cells_with_value: Vec<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if cells_with_value.len() == 1 {
                    let &&(x, y) = cells_with_value.first().unwrap();
                    self.set_value(x, y, value)?;
                    debug_only!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // règle 2: http://www.taupierbw.be/SudokuCoach/SC_NakedPairs.shtml
    fn naked_pairs(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for group in self.get_group(All) {
            let pairs: Vec<&Coords> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].len() == 2)
                .collect();

            for i in 0..pairs.len() {
                for j in i + 1..pairs.len() {
                    let &(x1, y1) = pairs[i];
                    let &(x2, y2) = pairs[j];
                    if self.possibility_board[y1][x1] == self.possibility_board[y2][x2] {
                        for &(x, y) in group.iter() {
                            if (x, y) == *pairs[i] || (x, y) == *pairs[j] {
                                continue;
                            }
                            for value in self.possibility_board[y1][x1].clone() {
                                if self.possibility_board[y][x].remove(&value) {
                                    debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 3: http://www.taupierbw.be/SudokuCoach/SC_NakedTriples.shtml
    fn naked_triples(&mut self) -> Result<bool, SudokuError> {
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
                for j in i + 1..pairs_or_triples.len() {
                    for k in j + 1..pairs_or_triples.len() {
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
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x,
                                            y,
                                            value
                                        );
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 4: http://www.taupierbw.be/SudokuCoach/SC_HiddenPairs.shtml
    fn hidden_pairs(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.len() != 2 {
                    continue;
                }
                for value2 in value1 + 1..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value1 != occurences_value2 {
                        continue;
                    }
                    for &&(x, y) in occurences_value1.iter() {
                        for value in 1..=self.n2 {
                            if value != value1
                                && value != value2
                                && self.possibility_board[y][x].remove(&value)
                            {
                                debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                modified = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 5: http://www.taupierbw.be/SudokuCoach/SC_HiddenTriples.shtml
    fn hidden_triples(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in value1 + 1..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in value2 + 1..=self.n2 {
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

                        for &&(x, y) in common_occurences.into_iter() {
                            for value in 1..=self.n2 {
                                if value != value1
                                    && value != value2
                                    && value != value3
                                    && self.possibility_board[y][x].remove(&value)
                                {
                                    debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 6: http://www.taupierbw.be/SudokuCoach/SC_NakedQuads.shtml
    fn naked_quads(&mut self) -> Result<bool, SudokuError> {
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
                for j in i + 1..pairs_or_triples_or_quads.len() {
                    for k in j + 1..pairs_or_triples_or_quads.len() {
                        for l in k + 1..pairs_or_triples_or_quads.len() {
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
                                                "({}, {}): possibilité {} supprimée",
                                                x,
                                                y,
                                                value
                                            );
                                            modified = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 7: http://www.taupierbw.be/SudokuCoach/SC_HiddenQuads.shtml
    fn hidden_quads(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for group in self.get_group(All) {
            for value1 in 1..self.n2 {
                let occurences_value1: HashSet<&Coords> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                    .collect();
                if occurences_value1.is_empty() {
                    continue;
                }
                for value2 in value1 + 1..=self.n2 {
                    let occurences_value2: HashSet<&Coords> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value2.is_empty() {
                        continue;
                    }
                    for value3 in value2 + 1..=self.n2 {
                        let occurences_value3: HashSet<&Coords> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();
                        if occurences_value3.is_empty() {
                            continue;
                        }
                        for value4 in value3 + 1..=self.n2 {
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
                            for &&(x, y) in common_occurences.into_iter() {
                                for value in 1..=self.n2 {
                                    if value != value1
                                        && value != value2
                                        && value != value3
                                        && value != value4
                                        && self.possibility_board[y][x].remove(&value)
                                    {
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x,
                                            y,
                                            value
                                        );
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 8: http://www.taupierbw.be/SudokuCoach/SC_PointingPair.shtml
    fn pointing_pair(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
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
                if x1 == x2 {
                    for y in 0..self.n2 {
                        if y == y1 || y == y2 {
                            continue;
                        }
                        if self.possibility_board[y][x1].remove(&value) {
                            debug_only!("({}, {}): possibilité {} supprimée", x1, y, value);
                            modified = true;
                        }
                    }
                } else if y1 == y2 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            debug_only!("({}, {}): possibilité {} supprimée", x, y1, value);
                            modified = true;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        Ok(modified)
    }

    // règle 9: http://www.taupierbw.be/SudokuCoach/SC_PointingTriple.shtml
    fn pointing_triple(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
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
                if x1 == x2 && x2 == x3 {
                    for y in 0..self.n2 {
                        if y == y1 || y == y2 || y == y3 {
                            continue;
                        }
                        if self.possibility_board[y][x1].remove(&value) {
                            debug_only!("({}, {}): possibilité {} supprimée", x1, y, value);
                            modified = true;
                        }
                    }
                } else if y1 == y2 && y2 == y3 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 || x == x3 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            debug_only!("({}, {}): possibilité {} supprimée", x, y1, value);
                            modified = true;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        Ok(modified)
    }

    // règle 10: http://www.taupierbw.be/SudokuCoach/SC_BoxReduction.shtml
    fn box_reduction(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for rows in self.get_group(Row) {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&Coords> = rows
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() < 2 || occurences.len() > 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(x, _)| x / self.n == x1 / self.n) {
                    for (x, y) in self.get_cell_group(x1, y1, Square) {
                        if y == y1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!(
                                "row x1:{x1} y1:{y1} x:{} y:{}: possibilité {} supprimée",
                                x,
                                y,
                                value
                            );
                            modified = true;
                        }
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
                if occurences.len() < 2 || occurences.len() > 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(_, y)| y / self.n == y1 / self.n) {
                    for (x, y) in self.get_cell_group(x1, y1, Square) {
                        if x == x1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!("col {x1}: ({}, {}) possibilité {} supprimée", x, y, value);
                            modified = true;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 11: http://www.taupierbw.be/SudokuCoach/SC_XWing.shtml
    fn x_wing(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..self.n2 - 1 {
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

                for i2 in i1 + 1..self.n2 {
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
                            debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                            modified = true;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 12: http://www.taupierbw.be/SudokuCoach/SC_FinnedXWing.shtml
    fn finned_x_wing(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..self.n2 {
            for i1 in 0..self.n2 - 1 {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();

                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();

                for i2 in i1 + 1..self.n2 {
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
                                debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                modified = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 13: http://www.taupierbw.be/SudokuCoach/SC_FrankenXWing.shtml
    fn franken_x_wing(&mut self) -> Result<bool, SudokuError> {
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
                            debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                            modified = true;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 14: https://www.taupierbw.be/SudokuCoach/SC_FinnedMutantXWing.shtml
    fn finned_mutant_x_wing(&mut self) -> Result<bool, SudokuError> {
        warn!("finned_mutant_x_wing not yet implemented");
        Ok(false)
    }

    // règle 15: http://www.taupierbw.be/SudokuCoach/SC_Skyscraper.shtml
    fn skyscraper(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..self.n2 - 1 {
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

                for i2 in i1 + 1..self.n2 {
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
                                debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                modified = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 16: http://www.taupierbw.be/SudokuCoach/SC_SimpleColoring.shtml
    pub fn simple_coloring(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..=self.n2 {
            let raw_strong_links = self.get_strong_links(value);
            let mut strong_links: HashMap<Coords, HashSet<Coords>> = HashMap::new();
            for (cell1, cell2) in raw_strong_links {
                strong_links
                    .entry(cell1)
                    .and_modify(|cell1_links| {
                        cell1_links.insert(cell2);
                    })
                    .or_insert_with(|| vec![cell2].into_iter().collect());

                strong_links
                    .entry(cell2)
                    .and_modify(|cell2_links| {
                        cell2_links.insert(cell1);
                    })
                    .or_insert_with(|| vec![cell1].into_iter().collect());
            }

            let mut chains: Vec<Vec<Coords>> = Vec::new();
            for cell in strong_links.keys().cloned().collect::<Vec<_>>() {
                let mut visited: HashSet<Coords> = HashSet::new();

                let mut stack = vec![(cell, vec![])];
                while let Some((current, mut path)) = stack.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    visited.insert(current);
                    path.push(current);

                    for neighbor in strong_links.get(&current).unwrap() {
                        if !visited.contains(neighbor) {
                            stack.push((*neighbor, path.clone()));
                        } else if path.contains(neighbor) {
                            chains.push(path.clone());
                        }
                    }
                }
            }

            let mut chains_hashset: Vec<(Vec<Coords>, HashSet<Coords>)> = chains
                .into_iter()
                .map(|chain| {
                    (
                        chain.clone(),
                        chain.into_iter().collect::<HashSet<Coords>>(),
                    )
                })
                .collect();
            chains_hashset.sort_by_key(|(chain, _hashset)| chain.len());

            let mut keeped_hashsets: Vec<&HashSet<Coords>> = Vec::new();
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
                    keeped_chains.push(chain1.clone());
                }
            }

            for chain in keeped_chains {
                let &(x1, y1) = chain.first().unwrap();
                let &(x2, y2) = chain.last().unwrap();
                if chain.len() % 2 == 0 {
                    let cell_group1: HashSet<Coords> = self.get_cell_group(x1, y1, All);
                    let cell_group2: HashSet<Coords> = self.get_cell_group(x2, y2, All);
                    let common_cells: HashSet<&Coords> =
                        cell_group1.intersection(&cell_group2).collect();
                    for &(x3, y3) in common_cells {
                        if (x3, y3) == (x1, y1) || (x3, y3) == (x2, y2) {
                            continue;
                        }
                        if self.possibility_board[y3][x3].remove(&value) {
                            debug_only!(
                                "({}, {}): possibilité {} supprimée (chaine {:?})",
                                x3,
                                y3,
                                value,
                                chain
                            );
                            modified = true;
                        }
                    }
                } else if self.is_same_group(x1, y1, x2, y2) {
                    for &(x, y) in chain.iter().step_by(2) {
                        if self.possibility_board[y][x].remove(&value) {
                            debug_only!(
                                "({}, {}): possibilité {} supprimée (chaine {:?})",
                                x,
                                y,
                                value,
                                chain
                            );
                            modified = true;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 17: http://www.taupierbw.be/SudokuCoach/SC_YWing.shtml
    fn y_wing(&mut self) -> Result<bool, SudokuError> {
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
                            debug_only!("({}, {}): possibilité {} supprimée", x3, y3, value);
                            modified = true;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 18: http://www.taupierbw.be/SudokuCoach/SC_WWing.shtml
    fn w_wing(&mut self) -> Result<bool, SudokuError> {
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
                                    if x3 == x
                                        || y3 == y
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
                                        "({}, {}): possibilité {} supprimée",
                                        x3,
                                        y3,
                                        value2
                                    );
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 19: http://www.taupierbw.be/SudokuCoach/SC_Swordfish.shtml
    fn swordfish(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..self.n2 - 1 {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                for i2 in i1 + 1..self.n2 {
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    for i3 in i2 + 1..self.n2 {
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
                                    debug_only!("({}, {}): possibilité {} supprimée", x, y, value);
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 20: http://www.taupierbw.be/SudokuCoach/SC_FinnedSwordfish.shtml
    fn finned_swordfish(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for value in 1..=self.n2 {
            for i1 in 0..self.n2 - 1 {
                let row1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|x| self.possibility_board[i1][*x].contains(&value))
                    .collect();
                let col1_positions: HashSet<usize> = (0..self.n2)
                    .filter(|y| self.possibility_board[*y][i1].contains(&value))
                    .collect();
                for i2 in i1 + 1..self.n2 {
                    let row2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|x| self.possibility_board[i2][*x].contains(&value))
                        .collect();
                    let col2_positions: HashSet<usize> = (0..self.n2)
                        .filter(|y| self.possibility_board[*y][i2].contains(&value))
                        .collect();
                    for i3 in i2 + 1..self.n2 {
                        let row3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|x| self.possibility_board[i3][*x].contains(&value))
                            .collect();
                        let col3_positions: HashSet<usize> = (0..self.n2)
                            .filter(|y| self.possibility_board[*y][i3].contains(&value))
                            .collect();

                        // i1, i2 and i3 represents rows or columns
                        let mut picked_cells: Vec<(bool, Coords)> = Vec::new();

                        if 2 <= row1_positions.len()
                            && row1_positions.len() <= 3
                            && 2 <= row2_positions.len()
                            && row2_positions.len() <= 3
                            && 2 <= row3_positions.len()
                            && row3_positions.len() <= 3
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
                                            debug_only!(
                                                "rows i1:{i1}, i2:{i2}, i3:{i3}: fin:{fin_x},{fin_y} picked:{x},{fin_y}"
                                            );
                                            picked_cells.push((true, (x, fin_y)));
                                        }
                                    }
                                }
                            }
                        }

                        if 2 <= col1_positions.len()
                            && col1_positions.len() <= 3
                            && 2 <= col2_positions.len()
                            && col2_positions.len() <= 3
                            && 2 <= col3_positions.len()
                            && col3_positions.len() <= 3
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
                                            debug_only!(
                                                "cols i1:{i1}, i2:{i2}, i3:{i3}: fin:{fin_x},{fin_y} picked:{fin_x},{y}"
                                            );
                                            picked_cells.push((false, (y, fin_x)));
                                        }
                                    }
                                }
                            }
                        }

                        for (is_row, data) in picked_cells {
                            if is_row {
                                let (finned_cell_x, finned_cell_y) = data;
                                let square_y = finned_cell_y - (finned_cell_y % self.n);
                                for dy in 0..self.n {
                                    let y = square_y + dy;
                                    if y == i1 || y == i2 || y == i3 {
                                        continue;
                                    }
                                    if self.possibility_board[y][finned_cell_x].remove(&value) {
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            finned_cell_x,
                                            y,
                                            value
                                        );
                                        modified = true;
                                    }
                                }
                            } else {
                                let (finned_cell_y, finned_cell_x) = data;
                                let square_x = finned_cell_x - (finned_cell_x % self.n);
                                for dx in 0..self.n {
                                    let x = square_x + dx;
                                    if x == i1 || x == i2 || x == i3 {
                                        continue;
                                    }
                                    if self.possibility_board[finned_cell_y][x].remove(&value) {
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x,
                                            finned_cell_y,
                                            value
                                        );
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 21: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedSwordfish.shtml
    fn sashimi_finned_swordfish(&mut self) -> Result<bool, SudokuError> {
        warn!("sashimi_finned_swordfish isn't implemented yet");
        Ok(false)
    }

    // règle 22: https://www.taupierbw.be/SudokuCoach/SC_FrankenSwordfish.shtml
    fn franken_swordfish(&mut self) -> Result<bool, SudokuError> {
        warn!("franken_swordfish not yet implemented");
        Ok(false)
    }

    // règle 23: https://www.taupierbw.be/SudokuCoach/SC_MutantSwordfish.shtml
    fn mutant_swordfish(&mut self) -> Result<bool, SudokuError> {
        warn!("mutant_swordfish not yet implemented");
        Ok(false)
    }

    // règle 24: https://www.taupierbw.be/SudokuCoach/SC_FinnedMutantSwordfish.shtml
    fn finned_mutant_swordfish(&mut self) -> Result<bool, SudokuError> {
        warn!("finned_mutant_swordfish not yet implemented");
        Ok(false)
    }

    // règle 25: https://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedMutantSwordfish.shtml
    fn sashimi_finned_mutant_swordfish(&mut self) -> Result<bool, SudokuError> {
        warn!("sashimi_finned_mutant_swordfish not yet implemented");
        Ok(false)
    }

    // règle 26: https://www.taupierbw.be/SudokuCoach/SC_Suedecoq.shtml
    fn sue_de_coq(&mut self) -> Result<bool, SudokuError> {
        warn!("sue_de_coq not yet implemented");
        Ok(false)
    }

    // règle 27: http://www.taupierbw.be/SudokuCoach/SC_XYZWing.shtml
    fn xyz_wing(&mut self) -> Result<bool, SudokuError> {
        warn!("xyz_wing isn't implemented yet");
        Ok(false)
    }

    // règle 28: https://www.taupierbw.be/SudokuCoach/SC_XCycle.shtml
    fn x_cycle(&mut self) -> Result<bool, SudokuError> {
        warn!("x_cycle not yet implemented");
        Ok(false)
    }

    // règle 29: http://www.taupierbw.be/SudokuCoach/SC_BUG.shtml
    fn bi_value_universal_grave(&mut self) -> Result<bool, SudokuError> {
        let mut unique_triple: Option<Coords> = None;
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let possibilities_number = self.possibility_board[y][x].len();
                if possibilities_number == 0 || possibilities_number == 2 {
                    continue;
                }
                if possibilities_number != 3 {
                    return Ok(false);
                }
                if unique_triple.is_some() {
                    return Ok(false);
                }
                unique_triple = Some((x, y));
            }
        }

        if unique_triple.is_none() {
            return Ok(false);
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
                self.set_value(x0, y0, *value)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    // règle 30: http://www.taupierbw.be/SudokuCoach/SC_XYChain.shtml
    fn xy_chain(&mut self) -> Result<bool, SudokuError> {
        warn!("xy_chain isn't implemented yet");
        Ok(false)
    }

    // règle 31: https://www.taupierbw.be/SudokuCoach/SC_Medusa.shtml
    fn three_d_medusa(&mut self) -> Result<bool, SudokuError> {
        warn!("three_d_medusa not yet implemented");
        Ok(false)
    }

    // règle 32: http://www.taupierbw.be/SudokuCoach/SC_Jellyfish.shtml
    fn jellyfish(&mut self) -> Result<bool, SudokuError> {
        warn!("jellyfish isn't implemented yet");
        Ok(false)
    }

    // règle 33: http://www.taupierbw.be/SudokuCoach/SC_FinnedJellyfish.shtml
    fn finned_jellyfish(&mut self) -> Result<bool, SudokuError> {
        warn!("finned_jellyfish isn't implemented yet");
        Ok(false)
    }

    // règle 34: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedJellyfish.shtml
    fn sashimi_finned_jellyfish(&mut self) -> Result<bool, SudokuError> {
        warn!("sashimi_finned_jellyfish isn't implemented yet");
        Ok(false)
    }

    // règle 35: https://www.taupierbw.be/SudokuCoach/SC_AvoidableRectangle.shtml
    fn avoidable_rectangle(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for y0 in 0..self.n2 {
            for x0 in 0..self.n2 {
                for y1 in y0 + 1..self.n2 {
                    for x1 in x0 + 1..self.n2 {
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

                            if self.possibility_board[y][x].remove(&val1) {
                                debug_only!("({}, {}): possibilité {} supprimée", x, y, val1);
                                modified = true;
                            }
                            if self.possibility_board[y][x].remove(&val2) {
                                debug_only!("({}, {}): possibilité {} supprimée", x, y, val2);
                                modified = true;
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
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x2,
                                            y,
                                            common_possibility
                                        );
                                        modified = true;
                                    }
                                }
                            } else {
                                for x in 0..self.n2 {
                                    if x == x2 || x == x3 {
                                        continue;
                                    }
                                    if self.possibility_board[y2][x].remove(&common_possibility) {
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x,
                                            y2,
                                            common_possibility
                                        );
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 36: https://www.taupierbw.be/SudokuCoach/SC_UniqueRectangle.shtml
    fn unique_rectangle(&mut self) -> Result<bool, SudokuError> {
        let mut modified = false;
        for y0 in 0..self.n2 {
            for x0 in 0..self.n2 {
                for y1 in y0 + 1..self.n2 {
                    for x1 in x0 + 1..self.n2 {
                        //For every rectangle possible
                        let rectangle = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];

                        //Get the different possibilities of the rectangle corner
                        let possval = rectangle
                            .iter()
                            .filter_map(|(x, y)| {
                                let poss = &self.possibility_board[*y][*x];
                                if !poss.is_empty() {
                                    Some(poss.iter().cloned().collect::<Vec<usize>>())
                                } else {
                                    None
                                }
                            })
                            .collect::<HashSet<Vec<usize>>>();

                        if possval.len() == 2 {
                            //Type 1 & 2
                            let (val1, val2) = {
                                let mut values_iter = possval.into_iter();
                                (
                                    values_iter
                                        .next()
                                        .unwrap()
                                        .into_iter()
                                        .collect::<HashSet<usize>>(),
                                    values_iter
                                        .next()
                                        .unwrap()
                                        .into_iter()
                                        .collect::<HashSet<usize>>(),
                                )
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
                                    .collect::<Vec<&&Coords>>();
                                let (x2, y2) = last_cell.pop().unwrap();
                                //remove the bi-value possibilities from the other corner
                                if val1.len() == 2 {
                                    for val in val1 {
                                        if self.possibility_board[*y2][*x2].remove(&val) {
                                            debug_only!(
                                                "({}, {}): possibilité {} supprimée",
                                                *x2,
                                                *y2,
                                                val
                                            );
                                            modified = true;
                                        }
                                    }
                                } else if val2.len() == 2 {
                                    for val in val2 {
                                        if self.possibility_board[*y2][*x2].remove(&val) {
                                            debug_only!(
                                                "({}, {}): possibilité {} supprimée",
                                                *x2,
                                                *y2,
                                                val
                                            );
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
                                let see_three_val =
                                    group1.intersection(&group2).collect::<HashSet<_>>();
                                //get the extra value of the three-value compared to the bi-value
                                let xtraval = if val1.len() == 2 {
                                    val2.difference(&val1).next().unwrap()
                                } else {
                                    val1.difference(&val2).next().unwrap()
                                };
                                //remove the extra value from the cells that sees both three-value-cells
                                for &(x4, y4) in see_three_val {
                                    if (x4 == x2 && y4 == y2) || (x4 == x3 && y4 == y3) {
                                        continue;
                                    }

                                    if self.possibility_board[y4][x4].remove(xtraval) {
                                        debug_only!(
                                            "({}, {}): possibilité {} supprimée",
                                            x4,
                                            y4,
                                            xtraval
                                        );
                                        modified = true;
                                    }
                                }
                            }
                        } else if possval.len() == 3 {
                            //Type 3
                            continue;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    // règle 37: https://www.taupierbw.be/SudokuCoach/SC_HiddenUniqueRectangle.shtml
    fn hidden_unique_rectangle(&mut self) -> Result<bool, SudokuError> {
        warn!("hidden_unique_rectangle not yet implemented");
        Ok(false)
    }

    // règle 38: http://www.taupierbw.be/SudokuCoach/SC_WXYZWing.shtml
    fn wxyz_wing(&mut self) -> Result<bool, SudokuError> {
        warn!("wxyz_wing isn't implemented yet");
        Ok(false)
    }

    // règle 39: https://www.taupierbw.be/SudokuCoach/SC_Firework.shtml
    fn firework(&mut self) -> Result<bool, SudokuError> {
        warn!("firework not yet implemented");
        Ok(false)
    }

    // règle 40: http://www.taupierbw.be/SudokuCoach/SC_APE.shtml
    fn subset_exclusion(&mut self) -> Result<bool, SudokuError> {
        warn!("subset_exclusion isn't implemented yet");
        Ok(false)
    }

    // règle 41: http://www.taupierbw.be/SudokuCoach/SC_EmptyRectangle.shtml
    fn empty_rectangle(&mut self) -> Result<bool, SudokuError> {
        warn!("empty_rectangle isn't implemented yet");
        Ok(false)
    }

    // règle 42: https://www.taupierbw.be/SudokuCoach/SC_SuedecoqExtended.shtml
    fn sue_de_coq_extended(&mut self) -> Result<bool, SudokuError> {
        warn!("sue_de_coq_extended not yet implemented");
        Ok(false)
    }

    // règle 43: https://www.taupierbw.be/SudokuCoach/SC_SKLoop.shtml
    fn sk_loop(&mut self) -> Result<bool, SudokuError> {
        warn!("sk_loop not yet implemented");
        Ok(false)
    }

    // règle 44: https://www.taupierbw.be/SudokuCoach/SC_Exocet.shtml
    fn exocet(&mut self) -> Result<bool, SudokuError> {
        warn!("exocet not yet implemented");
        Ok(false)
    }

    // règle 45: https://www.taupierbw.be/SudokuCoach/SC_ALS.shtml
    fn almost_locked_sets(&mut self) -> Result<bool, SudokuError> {
        warn!("almost_locked_sets not yet implemented");
        Ok(false)
    }

    // règle 46: https://www.taupierbw.be/SudokuCoach/SC_AIC.shtml
    fn alternating_inference_chain(&mut self) -> Result<bool, SudokuError> {
        warn!("alternating_inference_chain not yet implemented");
        Ok(false)
    }

    // règle 47: https://www.taupierbw.be/SudokuCoach/SC_DigitForcingChains.shtml
    fn digit_forcing_chains(&mut self) -> Result<bool, SudokuError> {
        warn!("digit_forcing_chains not yet implemented");
        Ok(false)
    }

    // règle 48: https://www.taupierbw.be/SudokuCoach/SC_NishioForcingChains.shtml
    fn nishio_forcing_chains(&mut self) -> Result<bool, SudokuError> {
        warn!("nishio_forcing_chains not yet implemented");
        Ok(false)
    }

    // règle 49: https://www.taupierbw.be/SudokuCoach/SC_CellForcingChains.shtml
    fn cell_forcing_chains(&mut self) -> Result<bool, SudokuError> {
        warn!("cell_forcing_chains not yet implemented");
        Ok(false)
    }

    // règle 50: https://www.taupierbw.be/SudokuCoach/SC_UnitForcingChains.shtml
    fn unit_forcing_chains(&mut self) -> Result<bool, SudokuError> {
        warn!("unit_forcing_chains not yet implemented");
        Ok(false)
    }

    // règle 51: http://www.taupierbw.be/SudokuCoach/SC_ALSchain.shtml
    fn almost_locked_set_forcing_chain(&mut self) -> Result<bool, SudokuError> {
        warn!("almost_locked_set_forcing_chain isn't implemented yet");
        Ok(false)
    }

    // règle 52: http://www.taupierbw.be/SudokuCoach/SC_DeathBlossom.shtml
    fn death_blossom(&mut self) -> Result<bool, SudokuError> {
        warn!("death_blossom isn't implemented yet");
        Ok(false)
    }

    // règle 53: http://www.taupierbw.be/SudokuCoach/SC_PatternOverlay.shtml
    fn pattern_overlay(&mut self) -> Result<bool, SudokuError> {
        warn!("pattern_overlay isn't implemented yet");
        Ok(false)
    }

    // règle 54: http://www.taupierbw.be/SudokuCoach/SC_BowmanBingo.shtml
    fn bowmans_bingo(&mut self) -> Result<bool, SudokuError> {
        warn!("bowmans_bingo isn't implemented yet");
        Ok(false)
    }
}
