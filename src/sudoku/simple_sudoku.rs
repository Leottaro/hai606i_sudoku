use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env::current_dir,
};

#[derive(Debug)]
pub struct Sudoku {
    n: usize,
    n2: usize,
    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
}

#[allow(dead_code)] // no warning due to unused functions
impl Sudoku {
    // GETTERS / SETTERS
    pub fn get_n(&self) -> usize {
        self.n
    }
    pub fn get_n2(&self) -> usize {
        self.n2
    }
    pub fn get_board(&self) -> Vec<Vec<usize>> {
        self.board.clone()
    }
    pub fn get_possibility_board(&self) -> Vec<Vec<HashSet<usize>>> {
        self.possibility_board.clone()
    }

    pub fn fix_value(&mut self, x: usize, y: usize, value: usize) {
        self.board[y][x] = value;
        self.possibility_board[y][x].clear();
        for group in Sudoku::get_cell_groups(self.n, x, y) {
            for (x, y) in group {
                self.possibility_board[y][x].remove(&value);
            }
        }
    }

    // GLOBAL FUNCTIONS

    pub fn get_groups(n: usize) -> Vec<Vec<(usize, usize)>> {
        let n2 = n * n;
        // lines and columns
        let mut lines: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut cols: Vec<Vec<(usize, usize)>> = Vec::new();
        for y in 0..n2 {
            let mut line: Vec<(usize, usize)> = Vec::new();
            let mut col: Vec<(usize, usize)> = Vec::new();
            for x in 0..n2 {
                line.push((x, y));
                col.push((y, x));
            }
            lines.push(line);
            cols.push(col);
        }

        // squares
        let mut squares: Vec<Vec<(usize, usize)>> = Vec::new();
        for y0 in (0..n2).step_by(n) {
            for x0 in (0..n2).step_by(n) {
                let mut square: Vec<(usize, usize)> = Vec::new();
                for j in 0..n {
                    for i in 0..n {
                        square.push((x0 + i, y0 + j));
                    }
                }
                squares.push(square);
            }
        }

        lines
            .into_iter()
            .chain(cols.into_iter())
            .chain(squares.into_iter())
            .collect()
    }

    pub fn get_cell_groups(n: usize, x: usize, y: usize) -> Vec<Vec<(usize, usize)>> {
        let mut groups: Vec<Vec<(usize, usize)>> = Vec::new();

        // line and culumn
        let mut line: Vec<(usize, usize)> = Vec::new();
        let mut col: Vec<(usize, usize)> = Vec::new();

        for i in 0..n * n {
            line.push((x, i));
            col.push((i, y));
        }

        // square
        let mut square: Vec<(usize, usize)> = Vec::new();
        let x0 = x - x % n;
        let y0 = y - y % n;

        for i in 0..n {
            for j in 0..n {
                square.push((x0 + i, y0 + j));
            }
        }

        groups.push(line);
        groups.push(col);
        groups.push(square);

        groups
    }

    // CREATION

    pub fn new(n: usize) -> Self {
        Self {
            n: n,
            n2: n * n,
            board: vec![vec![0; n * n]; n * n],
            possibility_board: vec![vec![(1..=n * n).collect(); n * n]; n * n],
        }
    }

    pub fn parse_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = {
            let mut path_builder = current_dir().unwrap();
            path_builder.push("res/sudoku_samples/");
            path_builder.push(file_name);
            path_builder.into_os_string().into_string().unwrap()
        };
        let file = std::fs::read_to_string(file_path)?;
        let mut lines = file.lines();
        let n: usize = lines.next().unwrap().parse()?;
        let n2 = n * n;

        let board: Vec<Vec<usize>> = lines
            .take(n2)
            .map(|line| {
                line.split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect()
            })
            .collect();

        let mut possibility_board: Vec<Vec<HashSet<usize>>> =
            vec![vec![(1..=n2).collect(); n2]; n2];

        for y in 0..n2 {
            for x in 0..n2 {
                let value = board[y][x];
                if value == 0 {
                    continue;
                }
                possibility_board[y][x].clear();
                for group in Sudoku::get_cell_groups(n, x, y) {
                    for (x, y) in group {
                        possibility_board[y][x].remove(&value);
                    }
                }
            }
        }

        let sudoku = Self {
            n,
            n2,
            board,
            possibility_board,
        };

        if let Err(((x1, y1), (x2, y2))) = sudoku.is_valid() {
            return Err(format!(
				"Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in the file",
				x1, y1, x2, y2
			)
			.into());
        }
        Ok(sudoku)
    }

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
        let groups = Sudoku::get_groups(self.n);
        for group in groups.into_iter() {
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
        for group in Sudoku::get_groups(self.n).into_iter() {
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
        for group in Sudoku::get_groups(self.n).into_iter() {
            let triples: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].len() == 2)
                .collect();

            let mut obvious_triples: HashSet<(usize, usize)> = HashSet::new();
            let mut obvious_values: HashSet<usize> = HashSet::new();
            for i in 0..triples.len() {
                for j in (i + 1)..triples.len() {
                    for k in (j + 1)..triples.len() {
                        let &(x1, y1) = triples[i];
                        let &(x2, y2) = triples[j];
                        let &(x3, y3) = triples[k];
                        let values: HashSet<usize> = self.possibility_board[y1][x1]
                            .clone()
                            .into_iter()
                            .chain(self.possibility_board[y2][x2].clone().into_iter())
                            .chain(self.possibility_board[y3][x3].clone().into_iter())
                            .collect();
                        if values.len() == 3 {
                            for value in values.into_iter() {
                                obvious_triples.insert((x1, y1));
                                obvious_triples.insert((x2, y2));
                                obvious_triples.insert((x3, y3));
                                obvious_values.insert(value);
                            }
                        }
                    }
                }
            }
            let obvious_values_count: Vec<usize> = obvious_values
                .iter()
                .map(|value| {
                    group
                        .iter()
                        .filter(|&&(x, y)| self.possibility_board[y][x].contains(value))
                        .count()
                })
                .collect();

            if obvious_triples.iter().count() < 3
                || obvious_values_count.iter().all(|&count| count < 4)
            {
                continue;
            }

            for &(x, y) in group.iter() {
                if obvious_triples.contains(&(x, y)) {
                    continue;
                }
                for value in obvious_values.iter() {
                    self.possibility_board[y][x].remove(value);
                    if debug {
                        println!("possibilitée {} supprimée de x: {}, y: {}", value, x, y);
                    }
                }
            }
            return true;
        }
        false
    }

    // règle 5: http://www.taupierbw.be/SudokuCoach/SC_HiddenPairs.shtml
    fn hidden_pairs(&mut self, debug: bool) -> bool {
        for group in Sudoku::get_groups(self.n).into_iter() {
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
        for group in Sudoku::get_groups(self.n).into_iter() {
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
                            println!("occurence1 {}: {:?}", value1, occurences_value1);
                            println!("occurence2 {}: {:?}", value2, occurences_value2);
                            println!("occurence3 {}: {:?}", value3, occurences_value3);
                            println!("{:?}", common_occurences);
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
        if debug {
            println!("naked_quads isn't implemented yet");
        }
        false
    }

    // règle 8: http://www.taupierbw.be/SudokuCoach/SC_HiddenQuads.shtml
    fn hidden_quads(&mut self, debug: bool) -> bool {
        if debug {
            println!("hidden_quads isn't implemented yet");
        }
        false
    }

    // règle 9: http://www.taupierbw.be/SudokuCoach/SC_PointingPair.shtml
    fn pointing_pair(&mut self, debug: bool) -> bool {
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                let group = Sudoku::get_cell_groups(self.n, x, y);
                let line_poss: HashSet<usize> = group[0]
                    .iter()
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let line_nosquare_poss: HashSet<usize> = group[0]
                    .iter()
                    .filter(|&&(x, y)| !group[2].contains(&(x, y)))
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let col_poss: HashSet<usize> = group[1]
                    .iter()
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let col_nosquare_poss: HashSet<usize> = group[1]
                    .iter()
                    .filter(|&&(x, y)| !group[2].contains(&(x, y)))
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let square_poss: HashSet<usize> = group[2]
                    .iter()
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let square_noline_poss: HashSet<usize> = group[2]
                    .iter()
                    .filter(|&&(x, y)| !group[0].contains(&(x, y)))
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                let square_nocol_poss: HashSet<usize> = group[2]
                    .iter()
                    .filter(|&&(x, y)| !group[1].contains(&(x, y)))
                    .flat_map(|&(x, y)| self.possibility_board[y][x].clone())
                    .collect();

                for i in 1..=self.n2 {
                    if line_poss.contains(&i) && square_poss.contains(&i) {
                        if line_nosquare_poss.contains(&i) && !square_noline_poss.contains(&i) {
                            for &(x1, y1) in group[0].iter() {
                                if !group[2].contains(&(x1, y1)) {
                                    self.possibility_board[y1][x1].remove(&i);
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            i, x, y
                                        );
                                    }
                                }
                            }
                            return true;
                        }
                        if !line_nosquare_poss.contains(&i) && square_noline_poss.contains(&i) {
                            for &(x1, y1) in group[2].iter() {
                                if !group[0].contains(&(x1, y1)) {
                                    self.possibility_board[y1][x1].remove(&i);
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            i, x, y
                                        );
                                    }
                                }
                            }
                            return true;
                        }
                    }
                    if col_poss.contains(&i) && square_poss.contains(&i) {
                        if col_nosquare_poss.contains(&i) && !square_nocol_poss.contains(&i) {
                            for &(x1, y1) in group[1].iter() {
                                if !group[2].contains(&(x1, y1)) {
                                    self.possibility_board[y1][x1].remove(&i);
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            i, x, y
                                        );
                                    }
                                }
                            }
                            return true;
                        }
                        if !col_nosquare_poss.contains(&i) && square_nocol_poss.contains(&i) {
                            for &(x1, y1) in group[2].iter() {
                                if !group[1].contains(&(x1, y1)) {
                                    self.possibility_board[y1][x1].remove(&i);
                                    if debug {
                                        println!(
                                            "possibilitée {} supprimée de x: {}, y: {}",
                                            i, x, y
                                        );
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // règle 10: http://www.taupierbw.be/SudokuCoach/SC_PointingTriple.shtml
    fn pointing_triple(&mut self, debug: bool) -> bool {
        if debug {
            println!("pointing_triple isn't implemented yet");
        }
        false
    }

    // règle 11: http://www.taupierbw.be/SudokuCoach/SC_BoxReduction.shtml
    fn box_reduction(&mut self, debug: bool) -> bool {
        if debug {
            println!("box_reduction isn't implemented yet");
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

    // tente d'exécuter chaque règles jusqu'à ce qu'aucune ne puisse être appliquée ou que le sudoku soit fini
    pub fn rule_solve(&mut self, debug: bool) -> Result<usize, ((usize, usize), (usize, usize))> {
        let rules: Vec<(fn(&mut Sudoku, bool) -> bool, usize)> = vec![
            (Sudoku::naked_singles, 1),
            (Sudoku::hidden_singles, 2),
            (Sudoku::naked_pairs, 3),
            (Sudoku::naked_triples, 4),
            (Sudoku::hidden_pairs, 5),
            (Sudoku::hidden_triples, 6),
            (Sudoku::naked_quads, 7),
            (Sudoku::hidden_quads, 8),
            (Sudoku::pointing_pair, 9),
            (Sudoku::pointing_triple, 10),
            (Sudoku::box_reduction, 11),
            (Sudoku::x_wing, 12),
            (Sudoku::finned_x_wing, 13),
            (Sudoku::sashimi_finned_x_wing, 14),
            (Sudoku::franken_x_wing, 15),
            (Sudoku::skyscraper, 16),
            (Sudoku::y_wing, 17),
            (Sudoku::w_wing, 18),
            (Sudoku::swordfish, 19),
            (Sudoku::finned_swordfish, 20),
            (Sudoku::sashimi_finned_swordfish, 21),
            (Sudoku::xyz_wing, 22),
            (Sudoku::bi_value_universal_grave, 23),
            (Sudoku::xy_chain, 24),
            (Sudoku::jellyfish, 25),
            (Sudoku::finned_jellyfish, 26),
            (Sudoku::sashimi_finned_jellyfish, 27),
            (Sudoku::wxyz_wing, 28),
            (Sudoku::subset_exclusion, 29),
            (Sudoku::empty_rectangle, 30),
            (Sudoku::almost_locked_set_forcing_chain, 31),
            (Sudoku::death_blossom, 32),
            (Sudoku::pattern_overlay, 33),
            (Sudoku::bowmans_bingo, 34),
        ];

        let mut difficulty: usize = 0;
        let mut modified;
        loop {
            // try the rules and set the difficulty in consequence
            modified = false;
            for &(rule, diff) in rules.iter() {
                // if the rule can't be applied, then pass to the next one
                if !rule(self, debug) {
                    continue;
                }

                if debug {
                    println!("règle {} appliquée", diff);
                    println!("{}", self);
                    self.display_possibilities();
                }

                difficulty = max(difficulty, diff);
                modified = true;
                let is_valid = self.is_valid();
                if is_valid.is_err() {
                    return Err(is_valid.unwrap_err());
                }
            }
            // if no rules can be applied, then stop
            if !modified {
                break;
            }
        }
        Ok(difficulty)
    }

    // BACKTRACK SOLVING

    pub fn backtrack_solve(&mut self, mut x: usize, mut y: usize) -> bool {
        if y == self.n2 - 1 && x == self.n2 {
            return true;
        }

        if x == self.n2 {
            y += 1;
            x = 0;
        }

        if self.board[y][x] != 0 {
            return self.backtrack_solve(x + 1, y);
        }

        let possible_values = self.possibility_board[y][x].clone();
        let cell_group: Vec<(usize, usize)> = Sudoku::get_cell_groups(self.n, x, y)
            .into_iter()
            .flatten()
            .collect();
        self.possibility_board[y][x].clear();
        for value in possible_values.clone().into_iter() {
            self.board[y][x] = value;
            let changing_cells: Vec<&(usize, usize)> = cell_group
                .iter()
                .filter(|(x, y)| self.possibility_board[*y][*x].contains(&value))
                .collect();
            for &&(x, y) in changing_cells.iter() {
                self.possibility_board[y][x].remove(&value);
            }
            if self.backtrack_solve(x + 1, y) {
                return true;
            }
            self.board[y][x] = 0;
            for &(x, y) in changing_cells {
                self.possibility_board[y][x].insert(value);
            }
        }
        self.possibility_board[y][x] = possible_values;

        return false;
    }

    // DISPLAY

    const BASE_64: [char; 64] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
        'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a',
        'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
        't', 'u', 'v', 'w', 'x', 'y', 'z', 'α', 'β', 'δ',
    ];
    pub fn to_string(&self) -> String {
        if self.n > 8 {
            return "Can't display a sudoku with n > 8".to_string();
        }
        let mut lines: Vec<String> = Vec::new();
        let n_dash = "─".repeat(2 * self.n + 1);

        lines.push({
            let mut first_line: Vec<String> = Vec::new();
            for _ in 0..self.n {
                first_line.push(n_dash.clone());
            }
            format!("┌{}┐", first_line.join("┬"))
        });

        for y in 0..self.n2 {
            let mut line: Vec<String> = Vec::new();
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    line.push("│".to_string());
                }
                if self.board[y][x] == 0 {
                    line.push("·".to_string());
                } else {
                    line.push(Sudoku::BASE_64[self.board[y][x] - 1].to_string());
                };
            }
            lines.push(format!("│ {} │", line.join(" ")));

            if y != self.n2 - 1 && (y + 1) % self.n == 0 {
                let mut temp: Vec<String> = Vec::new();
                for _ in 0..self.n {
                    temp.push(n_dash.clone());
                }
                lines.push(format!("├{}┤", temp.join("┼")));
            }
        }

        lines.push({
            let mut last_line: Vec<String> = Vec::new();
            for _ in 0..self.n {
                last_line.push(n_dash.clone());
            }
            format!("└{}┘", last_line.join("┴"))
        });

        lines.join("\n")
    }

    pub fn display_possibilities(&self) {
        println!("");
        for y in 0..self.n2 {
            if y != 0 && y % self.n == 0 {
                println!(
                    "{}┼{}┼{}",
                    "─".repeat(self.n2 * self.n + self.n * 4),
                    "─".repeat(self.n2 * self.n + self.n * 4),
                    "─".repeat(self.n2 * self.n + self.n * 4),
                );
            }
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    print!("│");
                }
                print!(" {{");
                for value in 1..=self.n2 {
                    if self.possibility_board[y][x].contains(&value) {
                        print!("{}", value);
                    } else {
                        print!(" ");
                    }
                }
                print!("}} ");
            }
            println!();
        }
        println!("\n\n");
    }

    // UTILITY

    pub fn is_valid(&self) -> Result<(), ((usize, usize), (usize, usize))> {
        // check si un groupe contient 2 fois la même valeur
        for group in Sudoku::get_groups(self.n).into_iter() {
            let mut already_seen_values: HashMap<usize, (usize, usize)> = HashMap::new();
            for (x, y) in group.into_iter() {
                let cell_value = self.board[y][x];
                if cell_value == 0 {
                    continue;
                }
                if already_seen_values.contains_key(&cell_value) {
                    return Err((
                        (x, y),
                        already_seen_values.get(&cell_value).unwrap().clone(),
                    ));
                }
                already_seen_values.insert(cell_value, (x, y));
            }
        }

        // check si une cellule pas encore fixée n'a plus de possibilités
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.board[y][x] == 0 && self.possibility_board[y][x].is_empty() {
                    return Err(((x, y), (x, y)));
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Sudoku {
    fn eq(&self, other: &Self) -> bool {
        if self.n != other.n {
            return false;
        }

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.board[y][x] != other.board[y][x] {
                    return false;
                }
            }
        }

        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if !self.possibility_board[y][x].eq(&other.possibility_board[y][x]) {
                    return false;
                }
            }
        }
        true
    }
}
