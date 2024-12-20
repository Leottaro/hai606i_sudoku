use std::{collections::HashSet, env::current_dir};

pub struct Sudoku {
    n: usize,
    n2: usize,
    board: Vec<Vec<usize>>,
}

impl Sudoku {
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

    pub fn get_possible_values(&self, x: usize, y: usize) -> HashSet<usize> {
        if self.board[y][x] != 0 {
            let mut set = HashSet::new();
            set.insert(self.board[y][x]);
            return set;
        }
        let mut possible_values_bool = vec![true; self.n2 + 1];
        possible_values_bool[0] = false; // 0 is not a possible value
        for (x, y) in Self::get_cell_group(self.n, x, y) {
            possible_values_bool[self.board[y][x]] = false;
        }
        possible_values_bool
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| i)
            .collect::<HashSet<usize>>()
    }

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
        let board: Vec<Vec<usize>> = lines
            .into_iter()
            .map(|line| {
                line.split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect()
            })
            .collect();
        Sudoku {
            n,
            n2: n * n,
            board,
        }
    }

    pub fn solve(&mut self, mut x: usize, mut y: usize) -> bool {
        // Check if we have reached the end of the matrix
        if y == self.n2 - 1 && x == self.n2 {
            return true;
        }

        // Move to the next row if we have reached the end of the current row
        if x == self.n2 {
            y += 1;
            x = 0;
        }

        // Skip cells that are already filled
        if self.board[y][x] != 0 {
            return self.solve(x + 1, y);
        }

        // Try filling the current cell with a valid value
        let possible_values = self.get_possible_values(x, y);
        for value in possible_values {
            self.board[y][x] = value;
            if self.solve(x + 1, y) {
                return true;
            }
            self.board[y][x] = 0;
        }

        // No valid value was found, so backtrack
        return false;
    }

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
                    line.push(Self::BASE_64[self.board[y][x] - 1].to_string());
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

        lines.join("\n") + "\n"
    }

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
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
