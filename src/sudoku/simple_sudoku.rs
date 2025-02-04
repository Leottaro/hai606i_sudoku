use std::{cmp::max, collections::HashSet, env::current_dir};

pub struct Sudoku {
    n: usize,
    n2: usize,
    board: Vec<Vec<usize>>,
    possibility_board: Vec<Vec<HashSet<usize>>>,
}

#[allow(dead_code)] // no warning due to unused functions
impl Sudoku {
    // GLOBAL FUNCTIONS

    // retourne la liste de tout les groupes de cellules (lignes, colonnes et carrés)
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

    // retourne toutes les cellules qui sont dans un groupe commun avec (x,y) (ligne, colonne ou carré)
    pub fn get_cell_group(n: usize, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut cells: Vec<(usize, usize)> = Vec::new();
        // lines and columns
        for i in 0..n * n {
            if i != y {
                cells.push((x, i));
            }
            if i != x {
                cells.push((i, y));
            }
        }

        // squares
        let x0 = x - x % n;
        let y0 = y - y % n;
        for i in 0..n {
            if y0 + i == y {
                continue;
            }
            for j in 0..n {
                if x0 + j != x {
                    cells.push((x0 + j, y0 + i));
                }
            }
        }
        cells
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

    // crées un sudoku à partir d'un fichier
    pub fn parse_file(file_name: &str) -> Self {
        let file_path = {
            let mut path_builder = current_dir().unwrap();
            path_builder.push("res/sudoku_samples/");
            path_builder.push(file_name);
            path_builder.into_os_string().into_string().unwrap()
        };
        let file = std::fs::read_to_string(file_path).expect("Failed to read file");
        let mut lines = file.lines();
        let n: usize = lines.next().unwrap().parse().unwrap();
        let n2 = n * n;

        let board: Vec<Vec<usize>> = lines
            .into_iter()
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
                possibility_board[y][x] = HashSet::new();
                for (x, y) in Sudoku::get_cell_group(n, x, y) {
                    possibility_board[y][x].remove(&value);
                }
            }
        }
        Sudoku {
            n,
            n2,
            board,
            possibility_board,
        }
    }

    // RULES SOLVING
    // CHECK https://sudoku.com/sudoku-rules/
    // THE RULES ARE LISTED BY INCREASING DIFFICULTY
    // A RULE RETURN TRUE IF IT FIXED SOME CELLS

    // règle 1
    fn last_free_cells(&mut self) -> bool {
        let mut last_free_cells: Vec<((usize, usize), usize)> = Vec::new();
        for y in 0..self.n2 {
            for x in 0..self.n2 {
                if self.possibility_board[y][x].len() != 1 {
                    continue;
                }
                let value = *self.possibility_board[y][x].iter().next().unwrap();
                self.board[y][x] = value;
                self.possibility_board[y][x] = HashSet::new();
                for (x, y) in Sudoku::get_cell_group(self.n, x, y) {
                    self.possibility_board[y][x].remove(&value);
                }
                last_free_cells.push(((x, y), value));
            }
        }
        if last_free_cells.is_empty() {
            false
        } else {
            for ((x, y), value) in last_free_cells.into_iter() {
                self.board[y][x] = value;
                self.possibility_board[y][x] = HashSet::new();
                for (x, y) in Sudoku::get_cell_group(self.n, x, y) {
                    self.possibility_board[y][x].remove(&value);
                }
            }
            true
        }
    }

    // règle 2
    fn last_remaining_cells(&mut self) -> bool {
        let groups = Sudoku::get_groups(self.n);
        let mut last_remaining_cell: Vec<((usize, usize), usize)> = Vec::new();
        for group in groups.into_iter() {
            let mut apparition_count: Vec<usize> = vec![0; self.n2 + 1];
            for &(x, y) in group.iter() {
                for &value in self.possibility_board[y][x].iter() {
                    apparition_count[value] += 1;
                }
            }
            for (value, &count) in apparition_count.iter().enumerate() {
                if count != 1 {
                    continue;
                }
                let &(x, y) = group
                    .iter()
                    .find(|&&(x, y)| self.possibility_board[y][x].contains(&value))
                    .unwrap();
                last_remaining_cell.push(((x, y), value));
            }
        }
        if last_remaining_cell.is_empty() {
            false
        } else {
            for ((x, y), value) in last_remaining_cell.into_iter() {
                self.board[y][x] = value;
                self.possibility_board[y][x] = HashSet::new();
                for (x, y) in Sudoku::get_cell_group(self.n, x, y) {
                    self.possibility_board[y][x].remove(&value);
                }
            }
            true
        }
    }

    // tente d'exécuter chaque règles jusqu'à ce qu'aucune ne puisse être appliquée ou que le sudoku soit fini
    pub fn rule_solve(&mut self) -> usize {
        let mut difficulty: usize = 0;
        while !self.is_finished() {
            // try the rules and set the difficulty in consequence
            if self.last_free_cells() {
                difficulty = max(difficulty, 1);
                continue;
            }
            if self.last_remaining_cells() {
                difficulty = max(difficulty, 2);
                continue;
            }
            break;
        }
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
        let cell_group = Sudoku::get_cell_group(self.n, x, y);
        self.possibility_board[y][x] = HashSet::new();
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

    // affiche le tableau des possibilitées
    pub fn display_possibilities(&self) {
        for y in 0..self.n2 {
            if y != 0 && y % self.n == 0 {
                println!("{}", "-".repeat(self.n2 * self.n2 * 2 + self.n2 + 1));
            }
            for x in 0..self.n2 {
                if x != 0 && x % self.n == 0 {
                    print!("| ");
                }
                print!(
                    "{}{}",
                    self.possibility_board[y][x]
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                    " ".repeat((self.n2 + 1 - self.possibility_board[y][x].len()) * 2)
                )
            }
            println!();
        }
        println!("\n\n");
    }

    // UTILITY

    // check si le sudoku actuel est valide (si aucune cellule n'entre en conflit avec une autre)
    pub fn is_valid(&self) -> bool {
        // lines
        for y in 0..self.n2 {
            let mut values = vec![false; self.n2];
            for x in 0..self.n2 {
                if self.board[y][x] == 0 || values[self.board[y][x] - 1] {
                    return false;
                }
                values[self.board[y][x] - 1] = true;
            }
        }

        // columns
        for x in 0..self.n2 {
            let mut values = vec![false; self.n2];
            for y in 0..self.n2 {
                if self.board[y][x] == 0 || values[self.board[y][x] - 1] {
                    return false;
                }
                values[self.board[y][x] - 1] = true;
            }
        }

        // squares
        for y0 in 0..self.n {
            for x0 in 0..self.n {
                let mut values = vec![false; self.n2];
                for j in 0..self.n {
                    for i in 0..self.n {
                        let y = y0 * self.n + j;
                        let x = x0 * self.n + i;
                        if self.board[y][x] == 0 || values[self.board[y][x] - 1] {
                            return false;
                        }
                        values[self.board[y][x] - 1] = true;
                    }
                }
            }
        }

        return true;
    }

    // check si le sudoku est fini
    pub fn is_finished(&self) -> bool {
        for line in self.board.iter() {
            for cell in line.iter() {
                if *cell == 0 {
                    return false;
                }
            }
        }
        return true;
    }
}

// on peut print le sudoku directement
impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
