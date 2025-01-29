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
    // CHECK https://sudoku.com·/sudoku-rules/
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT FIXED SOME CELLS

    // règle 1
    fn last_free_cells(&mut self) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n).into_iter() {
            let mut value = 0;
            let g: Vec<usize> = group.iter().map(|&(x, y)| self.board[y][x]).collect();
            if g.iter().filter(|&n| *n == 0).count() == 1 {
                for i in 1..=self.n2 {
                    if !g.contains(&i) {
                        value = i;
                    }
                }
                for (x, y) in group {
                    if self.board[y][x] == 0 {
                        modified = true;
                        self.fix_value(x, y, value);
                    }
                }
            }
        }
        modified
    }

    // règle 3 OU 4
    fn last_possible_number(&mut self) -> bool {
        let mut modified = false;
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() == 1 {
                    let value = self.possibility_board[y][x].iter().next().unwrap();
                    modified = true;
                    self.fix_value(x, y, *value);
                }
            }
        }
        modified
    }

    // rule 6
    fn obvious_pairs(&mut self) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n).into_iter() {
            let pairs: Vec<&(usize, usize)> = group
                .iter()
                .filter(|&&(x, y)| self.possibility_board[y][x].len() == 2)
                .collect();

            let mut obvious_pairs: HashSet<(usize, usize)> = HashSet::new();
            let mut obvious_values: HashSet<usize> = HashSet::new();
            for i in 0..pairs.len() {
                for j in (i + 1)..pairs.len() {
                    let &(x1, y1) = pairs[i];
                    let &(x2, y2) = pairs[j];
                    if self.possibility_board[y1][x1] == self.possibility_board[y2][x2] {
                        for &value in self.possibility_board[y1][x1].iter() {
                            obvious_pairs.insert((x1, y1));
                            obvious_pairs.insert((x2, y2));
                            obvious_values.insert(value);
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

            if obvious_pairs.iter().count() < 2
                || obvious_values_count.iter().all(|&count| count < 3)
            {
                continue;
            }
            modified = true;

            for &(x, y) in group.iter() {
                if obvious_pairs.contains(&(x, y)) {
                    continue;
                }
                for value in obvious_values.iter() {
                    self.possibility_board[y][x].remove(value);
                }
            }
        }
        modified
    }

    // rule 7
    fn obvious_triples(&mut self) -> bool {
        let mut modified = false;
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
            modified = true;

            for &(x, y) in group.iter() {
                if obvious_triples.contains(&(x, y)) {
                    continue;
                }
                for value in obvious_values.iter() {
                    self.possibility_board[y][x].remove(value);
                }
            }
        }
        modified
    }

    // règle 8
    fn hidden_singles(&mut self) -> bool {
        let groups = Sudoku::get_groups(self.n);
        let mut modified = false;
        for group in groups.into_iter() {
            for value in 1..=self.n2 {
                let cells_with_value: Vec<&(usize, usize)> = group
                    .iter()
                    .filter(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .collect();
                if cells_with_value.len() == 1 {
                    let &&(x, y) = cells_with_value.first().unwrap();
                    modified = true;
                    self.fix_value(x, y, value);
                }
            }
        }

        modified
    }

    // règle 9
    fn intersection<T: PartialEq + Clone>(vec1: &Vec<T>, vec2: &Vec<T>) -> Vec<T> {
        let mut result = vec1.clone();
        result.retain(|item| vec2.contains(item));
        result
    }
    fn hidden_pairs(&mut self) -> bool {
        let mut modified = false;
        for group in Sudoku::get_groups(self.n).into_iter() {
            for case in group.clone() {
                for other_case in group.clone() {
                    if case == other_case {
                        continue;
                    }
                    let intersection = Sudoku::intersection(
                        &self.possibility_board[case.1][case.0]
                            .iter()
                            .map(|&x| x)
                            .collect(),
                        &self.possibility_board[other_case.1][other_case.0]
                            .iter()
                            .map(|&x| x)
                            .collect(),
                    );
                    if intersection.len() == 2 {
                        self.possibility_board[case.1][case.0] =
                            intersection.iter().cloned().collect();
                        self.possibility_board[other_case.1][other_case.0] =
                            intersection.iter().cloned().collect();
                        modified = true;
                        //println!("hidden_pairs a été utilisée pour x:{} y:{} et x:{} y:{} avec pour valeurs {:?}", case.0, case.1, other_case.0, other_case.1, intersection);
                    }
                }
            }
        }
        modified
    }

    // rule 13
    pub fn x_wing(&mut self) -> bool {
        let mut modified = false;
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

                    // if there are 2 cells with the same value in the same 2 lines
                    // eliminate this value from the other cells in these columns
                    if !invalidate_x && x_positions.len() == 2 {
                        let (x1, x2) = (x_positions[0], x_positions[1]);
                        for y in 0..self.n2 {
                            if y == i1 || y == i2 {
                                continue;
                            }

                            if self.possibility_board[y][x1].remove(&value)
                                || self.possibility_board[y][x2].remove(&value)
                            {
                                modified = true;
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

                            if self.possibility_board[y1][x].remove(&value)
                                || self.possibility_board[y2][x].remove(&value)
                            {
                                modified = true;
                            }
                        }
                    }
                }
            }
        }
        modified
    }

    // tente d'exécuter chaque règles jusqu'à ce qu'aucune ne puisse être appliquée ou que le sudoku soit fini
    pub fn rule_solve(&mut self) -> usize {
        let rules: Vec<(fn(&mut Sudoku) -> bool, usize)> = vec![
            (Sudoku::last_free_cells, 1),
            (Sudoku::last_possible_number, 3),
            (Sudoku::obvious_pairs, 6),
            (Sudoku::obvious_triples, 7),
            (Sudoku::hidden_singles, 8),
            (Sudoku::x_wing, 13),
        ];

        let mut difficulty: usize = 0;
            // try the rules and set the difficulty in consequence
            for &(rule, diff) in rules.iter() {
                // if the rule can't be applied, then pass to the next one
                if !rule(self) {
                    continue;
                }

                println!("la rule {} a été appliquée", diff);
                println!("{}", self);
                self.display_possibilities();

                difficulty = max(difficulty, diff);
                if let Err(((x1, y1), (x2, y2))) = self.is_valid() {
                    println!("Sudoku isn't valid ! \n the cells ({},{}) and ({},{}) contains the same value\nThere must be an error in a rule", x1, y1, x2, y2);
                    return 0;
                }
                break;
            }
            // if no rules can be applied, then stop
            
        difficulty
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
                    "─".repeat(self.n2 * self.n + self.n * 2),
                    "─".repeat(self.n2 * self.n + self.n * 2),
                    "─".repeat(self.n2 * self.n + self.n * 2),
                );
            }
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    print!("│");
                }
                print!(
                    " {}{}{} ",
                    " ".repeat((self.n2 + 1 - self.possibility_board[y][x].len()) / 2),
                    self.possibility_board[y][x]
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<String>>()
                        .join(""),
                    " ".repeat((self.n2 - self.possibility_board[y][x].len()) / 2)
                )
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
