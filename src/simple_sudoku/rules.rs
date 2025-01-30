use std::collections::HashSet;

use super::{Sudoku, SudokuRules};

impl SudokuRules for Sudoku {
    // RULES SOLVING
    // CHECK https://www.taupierbw.be/SudokuCoach
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT CHANGED SOMETHING

    // règle 1: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    fn naked_singles(&mut self, debug: bool) -> bool {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() == 1 {
                    let &value = self.possibility_board[y][x].iter().next().unwrap();
                    self.fix_value(x, y, value);
                    if debug {
                        println!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    }
                    return true;
                }
            }
        }
        false
    }

    // règle 2: http://www.taupierbw.be/SudokuCoach/SC_Singles.shtml
    fn hidden_singles(&mut self, debug: bool) -> bool {
        for group in Sudoku::get_groups(self.n) {
            for value in 1..=self.n2 {
                let cells_with_value: Vec<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if cells_with_value.len() == 1 {
                    let &&(x, y) = cells_with_value.first().unwrap();
                    self.fix_value(x, y, value);
                    if debug {
                        println!("valeur {} fixée en x: {}, y: {}", value, x, y);
                    }
                    return true;
                }
            }
        }
        false
    }

    // règle 3: http://www.taupierbw.be/SudokuCoach/SC_NakedPairs.shtml
    fn naked_pairs(&mut self, debug: bool) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n) {
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
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            value, x, y
                                        );
                                    }
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

    // règle 4: http://www.taupierbw.be/SudokuCoach/SC_NakedTriples.shtml
    fn naked_triples(&mut self, debug: bool) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n) {
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
                            .cloned()
                            .collect::<HashSet<usize>>()
                            .union(&self.possibility_board[y3][x3])
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
                                        if debug {
                                            println!(
                                                "possibilitée {} supprimée de x: {}, y: {}",
                                                value, x, y
                                            );
                                        }
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

    // règle 5: http://www.taupierbw.be/SudokuCoach/SC_HiddenPairs.shtml
    fn hidden_pairs(&mut self, debug: bool) -> bool {
        for group in Sudoku::get_groups(self.n) {
            for value1 in 1..self.n2 {
                for value2 in (value1 + 1)..=self.n2 {
                    let occurences_value1: Vec<&(usize, usize)> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                        .collect();
                    let occurences_value2: Vec<&(usize, usize)> = group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                        .collect();
                    if occurences_value1.len() == 2 && occurences_value1 == occurences_value2 {
                        let mut modified = false;
                        for &(x, y) in occurences_value1.into_iter() {
                            for value in 1..=self.n2 {
                                if value != value1
                                    && value != value2
                                    && self.possibility_board[y][x].remove(&value)
                                {
                                    modified = true;
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            value, x, y
                                        );
                                    }
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

    // règle 6: http://www.taupierbw.be/SudokuCoach/SC_HiddenTriples.shtml
    fn hidden_triples(&mut self, debug: bool) -> bool {
        for group in Sudoku::get_groups(self.n) {
            for value1 in 1..self.n2 {
                for value2 in (value1 + 1)..=self.n2 {
                    for value3 in (value2 + 1)..=self.n2 {
                        let occurences_value1: HashSet<&(usize, usize)> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                            .collect();
                        let occurences_value2: HashSet<&(usize, usize)> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                            .collect();
                        let occurences_value3: HashSet<&(usize, usize)> = group
                            .iter()
                            .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                            .collect();
                        let common_occurences = occurences_value1
                            .union(&occurences_value2)
                            .cloned()
                            .collect::<HashSet<&(usize, usize)>>()
                            .union(&occurences_value3)
                            .cloned()
                            .collect::<HashSet<&(usize, usize)>>();

                        if !occurences_value1.is_empty()
                            && !occurences_value2.is_empty()
                            && !occurences_value3.is_empty()
                            && common_occurences.len() == 3
                        {
                            let mut modified = false;
                            for &(x, y) in common_occurences.into_iter() {
                                for value in 1..=self.n2 {
                                    if value != value1
                                        && value != value2
                                        && value != value3
                                        && self.possibility_board[y][x].remove(&value)
                                    {
                                        modified = true;
                                        if debug {
                                            println!(
                                                "possibilitée {} supprimée de x: {}, y: {}",
                                                value, x, y
                                            );
                                        }
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

    // règle 7: http://www.taupierbw.be/SudokuCoach/SC_NakedQuads.shtml
    fn naked_quads(&mut self, debug: bool) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n) {
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
                                .cloned()
                                .collect::<HashSet<usize>>()
                                .union(&self.possibility_board[y3][x3])
                                .cloned()
                                .collect::<HashSet<usize>>()
                                .union(&self.possibility_board[y4][x4])
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
                                            if debug {
                                                println!(
                                                    "possibilitée {} supprimée de x: {}, y: {}",
                                                    value, x, y
                                                );
                                            }
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
    fn hidden_quads(&mut self, debug: bool) -> bool {
        for group in Sudoku::get_groups(self.n) {
            for value1 in 1..self.n2 {
                for value2 in (value1 + 1)..=self.n2 {
                    for value3 in (value2 + 1)..=self.n2 {
                        for value4 in (value3 + 1)..=self.n2 {
                            let occurences_value1: HashSet<&(usize, usize)> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value1))
                                .collect();
                            let occurences_value2: HashSet<&(usize, usize)> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value2))
                                .collect();
                            let occurences_value3: HashSet<&(usize, usize)> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value3))
                                .collect();
                            let occurences_value4: HashSet<&(usize, usize)> = group
                                .iter()
                                .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value4))
                                .collect();
                            let common_occurences = occurences_value1
                                .union(&occurences_value2)
                                .cloned()
                                .collect::<HashSet<&(usize, usize)>>()
                                .union(&occurences_value3)
                                .cloned()
                                .collect::<HashSet<&(usize, usize)>>()
                                .union(&occurences_value4)
                                .cloned()
                                .collect::<HashSet<&(usize, usize)>>();

                            if !occurences_value1.is_empty()
                                && !occurences_value2.is_empty()
                                && !occurences_value3.is_empty()
                                && !occurences_value4.is_empty()
                                && common_occurences.len() == 4
                            {
                                let mut modified = false;
                                for &(x, y) in common_occurences.into_iter() {
                                    for value in 1..=self.n2 {
                                        if value != value1
                                            && value != value2
                                            && value != value3
                                            && value != value4
                                            && self.possibility_board[y][x].remove(&value)
                                        {
                                            modified = true;
                                            if debug {
                                                println!(
                                                    "possibilitée {} supprimée de x: {}, y: {}",
                                                    value, x, y
                                                );
                                            }
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

    // règle 9: http://www.taupierbw.be/SudokuCoach/SC_PointingPair.shtml
    fn pointing_pair(&mut self, debug: bool) -> bool {
        for square in Sudoku::get_squares(self.n) {
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
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x1, y);
                            }
                            modified = true;
                        }
                    }
                } else if y1 == y2 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x, y1);
                            }
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
    fn pointing_triple(&mut self, debug: bool) -> bool {
        for square in Sudoku::get_squares(self.n) {
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
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x1, y);
                            }
                            modified = true;
                        }
                    }
                } else if y1 == y2 && y2 == y3 {
                    for x in 0..self.n2 {
                        if x == x1 || x == x2 || x == x3 {
                            continue;
                        }
                        if self.possibility_board[y1][x].remove(&value) {
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x, y1);
                            }
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
    fn box_reduction(&mut self, debug: bool) -> bool {
        let mut modified = false;
        for lines in Sudoku::get_lines(self.n) {
            for value in 1..=self.n2 {
                let mut occurences: Vec<&(usize, usize)> = lines
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if occurences.len() != 2 && occurences.len() != 3 {
                    continue;
                }
                let &(x1, y1) = occurences.pop().unwrap();
                if occurences.iter().all(|&(x, _)| x / self.n == x1 / self.n) {
                    for (x, y) in Sudoku::get_cell_square(self.n, x1, y1) {
                        if y == y1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x, y);
                            }
                            modified = true;
                        }
                    }
                    if modified {
                        return true;
                    }
                }
            }
        }

        for cols in Sudoku::get_cols(self.n) {
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
                    for (x, y) in Sudoku::get_cell_square(self.n, x1, y1) {
                        if x == x1 {
                            continue;
                        }
                        if self.possibility_board[y][x].remove(&value) {
                            if debug {
                                println!("possibilitée {} supprimée de x: {}, y: {}", value, x, y);
                            }
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
    fn x_wing(&mut self, debug: bool) -> bool {
        for value in 1..self.n2 {
            for i1 in 0..(self.n2 - 1) {
                for i2 in (i1 + 1)..self.n2 {
                    // i1 and i2 represents rows or columns

                    // collect the indexes of the cells that contains the value in the lines (x_position) or the column (y_position) i1 and i2
                    let mut x_positions: Vec<usize> = Vec::new();
                    let mut invalidate_x = false;

                    let mut y_positions: Vec<usize> = Vec::new();
                    let mut invalidate_y = false;

                    for j in 0..self.n2 {
                        // if this value is on the same cell in lines i1 and i2
                        if !invalidate_x {
                            let cell1 = self.possibility_board[i1][j].contains(&value);
                            let cell2 = self.possibility_board[i2][j].contains(&value);
                            if cell1 ^ cell2 {
                                invalidate_x = true;
                            } else if cell1 && cell2 {
                                x_positions.push(j);
                            }
                        }

                        // if this value is on the same cell in columns i1 and i2
                        if !invalidate_y {
                            let cell1 = self.possibility_board[j][i1].contains(&value);
                            let cell2 = self.possibility_board[j][i2].contains(&value);
                            if cell1 ^ cell2 {
                                invalidate_y = true;
                            } else if cell1 && cell2 {
                                y_positions.push(j);
                            }
                        }
                    }

                    let mut modified = false;

                    // if there are 2 cells with the same value in the same 2 lines
                    // eliminate this value from the other cells in these columns
                    if !invalidate_x && x_positions.len() == 2 {
                        let (x1, x2) = (x_positions[0], x_positions[1]);
                        for y in 0..self.n2 {
                            if y == i1 || y == i2 {
                                continue;
                            }

                            if self.possibility_board[y][x1].remove(&value) {
                                if debug {
                                    println!(
                                        "possibilitée {} supprimée de x: {}, y: {}",
                                        value, x1, y
                                    );
                                }
                                modified = true
                            }

                            if self.possibility_board[y][x2].remove(&value) {
                                if debug {
                                    println!(
                                        "possibilitée {} supprimée de x: {}, y: {}",
                                        value, x2, y
                                    );
                                }
                                modified = true
                            }

                            if modified {
                                return true;
                            }
                        }
                    }

                    // if there are 2 cells with the same value in the same 2 columns
                    // eliminate this value from the other cells in these lines
                    if !invalidate_y && y_positions.len() == 2 {
                        let (y1, y2) = (y_positions[0], y_positions[1]);
                        for x in 0..self.n2 {
                            if x == i1 || x == i2 {
                                continue;
                            }

                            if self.possibility_board[y1][x].remove(&value) {
                                if debug {
                                    println!(
                                        "possibilitée {} supprimée de x: {}, y: {}",
                                        value, x, y1
                                    );
                                }
                                modified = true
                            }

                            if self.possibility_board[y2][x].remove(&value) {
                                if debug {
                                    println!(
                                        "possibilitée {} supprimée de x: {}, y: {}",
                                        value, x, y2
                                    );
                                }
                                modified = true
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

    // règle 13: http://www.taupierbw.be/SudokuCoach/SC_FinnedXWing.shtml
    fn finned_x_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("finned_x_wing isn't implemented yet");
        }
        false
    }

    // règle 14: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedXWing.shtml
    fn sashimi_finned_x_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("sashimi_finned_x_wing isn't implemented yet");
        }
        false
    }

    // règle 15: http://www.taupierbw.be/SudokuCoach/SC_FrankenXWing.shtml
    fn franken_x_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("franken_x_wing isn't implemented yet");
        }
        false
    }

    // règle 16: http://www.taupierbw.be/SudokuCoach/SC_Skyscraper.shtml
    fn skyscraper(&mut self, debug: bool) -> bool {
        if debug {
            println!("skyscraper isn't implemented yet");
        }
        false
    }

    // règle 17: http://www.taupierbw.be/SudokuCoach/SC_YWing.shtml
    fn y_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("y_wing isn't implemented yet");
        }
        false
    }

    // règle 18: http://www.taupierbw.be/SudokuCoach/SC_WWing.shtml
    fn w_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("w_wing isn't implemented yet");
        }
        false
    }

    // règle 19: http://www.taupierbw.be/SudokuCoach/SC_Swordfish.shtml
    fn swordfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("swordfish isn't implemented yet");
        }
        false
    }

    // règle 20: http://www.taupierbw.be/SudokuCoach/SC_FinnedSwordfish.shtml
    fn finned_swordfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("finned_swordfish isn't implemented yet");
        }
        false
    }

    // règle 21: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedSwordfish.shtml
    fn sashimi_finned_swordfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("sashimi_finned_swordfish isn't implemented yet");
        }
        false
    }

    // règle 22: http://www.taupierbw.be/SudokuCoach/SC_XYZWing.shtml
    fn xyz_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("xyz_wing isn't implemented yet");
        }
        false
    }

    // règle 23: http://www.taupierbw.be/SudokuCoach/SC_BUG.shtml
    fn bi_value_universal_grave(&mut self, debug: bool) -> bool {
        if debug {
            println!("bi_value_universal_grave isn't implemented yet");
        }
        false
    }

    // règle 24: http://www.taupierbw.be/SudokuCoach/SC_XYChain.shtml
    fn xy_chain(&mut self, debug: bool) -> bool {
        if debug {
            println!("xy_chain isn't implemented yet");
        }
        false
    }

    // règle 25: http://www.taupierbw.be/SudokuCoach/SC_Jellyfish.shtml
    fn jellyfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("jellyfish isn't implemented yet");
        }
        false
    }

    // règle 26: http://www.taupierbw.be/SudokuCoach/SC_FinnedJellyfish.shtml
    fn finned_jellyfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("finned_jellyfish isn't implemented yet");
        }
        false
    }

    // règle 27: http://www.taupierbw.be/SudokuCoach/SC_SashimiFinnedJellyfish.shtml
    fn sashimi_finned_jellyfish(&mut self, debug: bool) -> bool {
        if debug {
            println!("sashimi_finned_jellyfish isn't implemented yet");
        }
        false
    }

    // règle 28: http://www.taupierbw.be/SudokuCoach/SC_WXYZWing.shtml
    fn wxyz_wing(&mut self, debug: bool) -> bool {
        if debug {
            println!("wxyz_wing isn't implemented yet");
        }
        false
    }

    // règle 29: http://www.taupierbw.be/SudokuCoach/SC_APE.shtml
    fn subset_exclusion(&mut self, debug: bool) -> bool {
        if debug {
            println!("subset_exclusion isn't implemented yet");
        }
        false
    }

    // règle 30: http://www.taupierbw.be/SudokuCoach/SC_EmptyRectangle.shtml
    fn empty_rectangle(&mut self, debug: bool) -> bool {
        if debug {
            println!("empty_rectangle isn't implemented yet");
        }
        false
    }

    // règle 31: http://www.taupierbw.be/SudokuCoach/SC_ALSchain.shtml
    fn almost_locked_set_forcing_chain(&mut self, debug: bool) -> bool {
        if debug {
            println!("almost_locked_set_forcing_chain isn't implemented yet");
        }
        false
    }

    // règle 32: http://www.taupierbw.be/SudokuCoach/SC_DeathBlossom.shtml
    fn death_blossom(&mut self, debug: bool) -> bool {
        if debug {
            println!("death_blossom isn't implemented yet");
        }
        false
    }

    // règle 33: http://www.taupierbw.be/SudokuCoach/SC_PatternOverlay.shtml
    fn pattern_overlay(&mut self, debug: bool) -> bool {
        if debug {
            println!("pattern_overlay isn't implemented yet");
        }
        false
    }

    // règle 34: http://www.taupierbw.be/SudokuCoach/SC_BowmanBingo.shtml
    fn bowmans_bingo(&mut self, debug: bool) -> bool {
        if debug {
            println!("bowmans_bingo isn't implemented yet");
        }
        false
    }
}
