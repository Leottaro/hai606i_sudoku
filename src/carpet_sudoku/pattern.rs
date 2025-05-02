use super::{CarpetLinks, CarpetPattern, RawLink};
use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, RwLock},
};

type PatternSubLinksKey = (usize, CarpetPattern);
static PATTERN_SUB_LINKS: LazyLock<RwLock<HashMap<PatternSubLinksKey, Vec<CarpetLinks>>>> =
    LazyLock::new(Default::default);

impl CarpetPattern {
    pub fn to_db(&self) -> (i16, Option<i16>) {
        match *self {
            CarpetPattern::Simple => (0, None),
            CarpetPattern::Samurai => (1, None),
            CarpetPattern::Diagonal(size) => (2, Some(size as i16)),
            CarpetPattern::DenseDiagonal(size) => (3, Some(size as i16)),
            CarpetPattern::Carpet(size) => (4, Some(size as i16)),
            CarpetPattern::DenseCarpet(size) => (5, Some(size as i16)),
            CarpetPattern::Thorus(size) => (6, Some(size as i16)),
            CarpetPattern::DenseThorus(size) => (7, Some(size as i16)),
            CarpetPattern::Custom(_) => panic!("Custom pattern not supported in DB"),
        }
    }

    pub fn from_db(pattern: i16, pattern_size: Option<i16>) -> Self {
        match (pattern, pattern_size) {
            (0, None) => CarpetPattern::Simple,
            (1, None) => CarpetPattern::Samurai,
            (2, Some(n)) => CarpetPattern::Diagonal(n as usize),
            (3, Some(n)) => CarpetPattern::DenseDiagonal(n as usize),
            (4, Some(n)) => CarpetPattern::Carpet(n as usize),
            (5, Some(n)) => CarpetPattern::DenseCarpet(n as usize),
            (6, Some(n)) => CarpetPattern::Thorus(n as usize),
            (7, Some(n)) => CarpetPattern::DenseThorus(n as usize),
            (a, b) => panic!("pattern:{a} & pattern_size:{:?} not recognized !", b),
        }
    }

    pub fn iter() -> impl Iterator<Item = CarpetPattern> {
        vec![
            CarpetPattern::Simple,
            CarpetPattern::Diagonal(2),
            CarpetPattern::DenseDiagonal(2),
            CarpetPattern::Diagonal(3),
            CarpetPattern::DenseDiagonal(3),
            CarpetPattern::Diagonal(4),
            CarpetPattern::DenseDiagonal(4),
            CarpetPattern::Carpet(2),
            CarpetPattern::Thorus(2),
            CarpetPattern::DenseCarpet(2),
            CarpetPattern::DenseThorus(2),
            CarpetPattern::Samurai,
            CarpetPattern::Diagonal(5),
            CarpetPattern::DenseDiagonal(5),
            CarpetPattern::Carpet(3),
            CarpetPattern::Thorus(3),
            CarpetPattern::DenseCarpet(3),
            CarpetPattern::DenseThorus(3),
        ]
        .into_iter()
    }

    pub fn iter_simple() -> impl Iterator<Item = CarpetPattern> {
        vec![
            CarpetPattern::Simple,
            CarpetPattern::Samurai,
            CarpetPattern::Diagonal(2),
            CarpetPattern::DenseDiagonal(2),
            CarpetPattern::Carpet(2),
            CarpetPattern::DenseCarpet(2),
            CarpetPattern::Thorus(2),
            CarpetPattern::DenseThorus(2),
        ]
        .into_iter()
    }

    pub fn get_n_sudokus(&self) -> usize {
        match *self {
            CarpetPattern::Simple => 1,
            CarpetPattern::Samurai => 5,
            CarpetPattern::Diagonal(size)
            | CarpetPattern::DenseDiagonal(size)
            | CarpetPattern::Custom(size) => size,
            CarpetPattern::Carpet(size)
            | CarpetPattern::DenseCarpet(size)
            | CarpetPattern::Thorus(size)
            | CarpetPattern::DenseThorus(size) => size * size,
        }
    }

    pub fn get_size(&self) -> usize {
        match *self {
            CarpetPattern::Simple => 1,
            CarpetPattern::Samurai => 5,
            CarpetPattern::Diagonal(size)
            | CarpetPattern::DenseDiagonal(size)
            | CarpetPattern::Carpet(size)
            | CarpetPattern::DenseCarpet(size)
            | CarpetPattern::Thorus(size)
            | CarpetPattern::DenseThorus(size)
            | CarpetPattern::Custom(size) => size,
        }
    }

    pub fn set_size(&mut self, new_size: usize) {
        match self {
            CarpetPattern::Diagonal(size)
            | CarpetPattern::Carpet(size)
            | CarpetPattern::DenseDiagonal(size)
            | CarpetPattern::DenseCarpet(size)
            | CarpetPattern::Thorus(size)
            | CarpetPattern::DenseThorus(size)
            | CarpetPattern::Custom(size) => *size = new_size,
            _ => (),
        }
    }

    pub fn get_raw_links(&self, n: usize) -> Vec<RawLink> {
        let up_left = 0;
        let up_right = n - 1;
        let bottom_left = n * (n - 1);
        let bottom_right = n * n - 1;

        match *self {
            CarpetPattern::Simple => vec![],
            CarpetPattern::Samurai => vec![
                ((0, up_left), (1, bottom_right)),
                ((0, up_right), (2, bottom_left)),
                ((0, bottom_left), (3, up_right)),
                ((0, bottom_right), (4, up_left)),
            ],
            CarpetPattern::Diagonal(size) => (1..size)
                .map(|i| ((i - 1, up_right), (i, bottom_left)))
                .collect(),
            CarpetPattern::Carpet(size) => {
                let mut links = Vec::new();
                for y in 0..size {
                    for x in 0..size {
                        let sudoku_i = y * size + x;

                        if y < size - 1 {
                            let bottom_i = (y + 1) * size + x;
                            links.extend(
                                (0..n).map(|k| {
                                    ((sudoku_i, bottom_left + k), (bottom_i, up_left + k))
                                }),
                            );
                        }

                        if x < size - 1 {
                            let right_i = y * size + x + 1;
                            links.extend((0..n).map(|k| {
                                ((sudoku_i, n * k + up_right), (right_i, n * k + up_left))
                            }));
                        }

                        if y < size - 1 && x < size - 1 {
                            let corner_i = (y + 1) * size + x + 1;
                            links.push(((sudoku_i, bottom_right), (corner_i, up_left)));
                        }

                        if y < size - 1 && x > 0 {
                            let corner_i = (y + 1) * size + x - 1;
                            links.push(((sudoku_i, bottom_left), (corner_i, up_right)));
                        }
                    }
                }
                links
            }
            CarpetPattern::Thorus(size) => {
                let mut links = Vec::new();
                for y in 0..size {
                    for x in 0..size {
                        let sudoku_i = y * size + x;

                        let bottom_i = ((y + 1) % size) * size + x;
                        links.extend(
                            (0..n).map(|k| ((sudoku_i, bottom_left + k), (bottom_i, up_left + k))),
                        );

                        let right_i = y * size + (x + 1) % size;
                        links.extend(
                            (0..n).map(|k| {
                                ((sudoku_i, n * k + up_right), (right_i, n * k + up_left))
                            }),
                        );

                        let corner_i = ((y + 1) % size) * size + (x + 1) % size;
                        links.push(((sudoku_i, bottom_right), (corner_i, up_left)));
                        let corner_i =
                            ((y + 1) % size) * size + if x == 0 { size - 1 } else { x - 1 };
                        links.push(((sudoku_i, bottom_left), (corner_i, up_right)));
                    }
                }
                links
            }
            CarpetPattern::DenseDiagonal(size) => {
                let mut links = Vec::new();
                for sudoku_i in 0..size - 1 {
                    for j in 1..n {
                        let sudoku_j = sudoku_i + j;
                        if sudoku_j >= size {
                            continue;
                        }
                        for y1 in 0..n - j {
                            let y2 = y1 + j;
                            for x1 in j..n {
                                let x2 = x1 - j;
                                links.push(((sudoku_i, (y1 * n) + x1), (sudoku_j, (y2 * n) + x2)));
                            }
                        }
                    }
                }
                links
            }
            CarpetPattern::DenseCarpet(size) => {
                let mut links = Vec::new();
                for y in 0..size {
                    for x in 0..size {
                        let sudoku_i = y * size + x;

                        for dx in 1..n {
                            if x + dx >= size {
                                continue;
                            }
                            let right_i = y * size + x + dx;
                            for y1 in 0..n {
                                let y2 = y1;
                                for x1 in dx..n {
                                    let x2 = x1 - dx;
                                    links.push((
                                        (sudoku_i, (y1 * n) + x1),
                                        (right_i, (y2 * n) + x2),
                                    ));
                                }
                            }
                        }

                        for dy in 1..n {
                            if y + dy >= size {
                                continue;
                            }
                            let bottom_i = (y + dy) * size + x;
                            for y1 in dy..n {
                                let y2 = y1 - dy;
                                for x1 in 0..n {
                                    let x2 = x1;
                                    links.push((
                                        (sudoku_i, (y1 * n) + x1),
                                        (bottom_i, (y2 * n) + x2),
                                    ));
                                }
                            }
                        }

                        for dy in 1..n {
                            if y + dy >= size {
                                continue;
                            }
                            for dx in 1..n {
                                if x + dx >= size {
                                    continue;
                                }
                                let corner_i = (y + dy) * size + x + dx;
                                for y1 in dy..n {
                                    let y2 = y1 - dy;
                                    for x1 in dx..n {
                                        let x2 = x1 - dx;
                                        links.push((
                                            (sudoku_i, (y1 * n) + x1),
                                            (corner_i, (y2 * n) + x2),
                                        ));
                                    }
                                }
                            }
                        }

                        for dy in 1..n {
                            if y + dy >= size {
                                continue;
                            }
                            for dx in 1..n {
                                if x < dx {
                                    continue;
                                }
                                let corner_i = (y + dy) * size + x - dx;
                                for y1 in dy..n {
                                    let y2 = y1 - dy;
                                    for x1 in 0..n - dx {
                                        let x2 = x1 + dx;
                                        links.push((
                                            (sudoku_i, (y1 * n) + x1),
                                            (corner_i, (y2 * n) + x2),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                links
            }
            CarpetPattern::DenseThorus(size) => {
                let mut links = Vec::new();
                for y in 0..size {
                    for x in 0..size {
                        let sudoku_i = y * size + x;

                        for dx in 1..n {
                            let right_i = y * size + (x + dx) % size;
                            for y1 in 0..n {
                                let y2 = y1;
                                for x1 in dx..n {
                                    let x2 = x1 - dx;
                                    links.push((
                                        (sudoku_i, (y1 * n) + x1),
                                        (right_i, (y2 * n) + x2),
                                    ));
                                }
                            }
                        }

                        for dy in 1..n {
                            let bottom_i = ((y + dy) % size) * size + x;
                            for y1 in dy..n {
                                let y2 = y1 - dy;
                                for x1 in 0..n {
                                    let x2 = x1;
                                    links.push((
                                        (sudoku_i, (y1 * n) + x1),
                                        (bottom_i, (y2 * n) + x2),
                                    ));
                                }
                            }
                        }

                        for dy in 1..n {
                            for dx in 1..n {
                                let corner_i = ((y + dy) % size) * size + (x + dx) % size;
                                for y1 in dy..n {
                                    let y2 = y1 - dy;
                                    for x1 in dx..n {
                                        let x2 = x1 - dx;
                                        links.push((
                                            (sudoku_i, (y1 * n) + x1),
                                            (corner_i, (y2 * n) + x2),
                                        ));
                                    }
                                }
                            }
                        }

                        for dy in 1..n {
                            for dx in 1..n {
                                let corner_i = ((y + dy) % size) * size
                                    + if x < dx { size - dx } else { x - dx };
                                for y1 in dy..n {
                                    let y2 = y1 - dy;
                                    for x1 in 0..n - dx {
                                        let x2 = x1 + dx;
                                        links.push((
                                            (sudoku_i, (y1 * n) + x1),
                                            (corner_i, (y2 * n) + x2),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                links
            }
            CarpetPattern::Custom(_) => vec![],
        }
    }

    pub fn get_carpet_links(&self, n: usize) -> CarpetLinks {
        let mut links: CarpetLinks = HashMap::new();

        for ((sudoku1, square1), (sudoku2, square2)) in self.get_raw_links(n) {
            links
                .entry(sudoku1)
                .and_modify(|sudoku1_links| {
                    sudoku1_links.insert((square1, sudoku2, square2));
                })
                .or_insert_with(|| vec![(square1, sudoku2, square2)].into_iter().collect());

            links
                .entry(sudoku2)
                .and_modify(|sudoku2_links| {
                    sudoku2_links.insert((square2, sudoku1, square1));
                })
                .or_insert_with(|| vec![(square2, sudoku1, square1)].into_iter().collect());
        }

        links
    }

    pub fn get_sub_links(&self, n: usize) -> Vec<CarpetLinks> {
        match self {
            CarpetPattern::Custom(_) => (),
            pattern => {
                if let Some(links) = PATTERN_SUB_LINKS.read().unwrap().get(&(n, *pattern)) {
                    return links.clone();
                }
            }
        }

        let links = self.get_carpet_links(n);
        let mut already_explored_combinaisons = HashSet::new();

        let sub_links = Self::_get_sub_links(
            self.get_n_sudokus(),
            &links,
            &mut already_explored_combinaisons,
        );

        match self {
            CarpetPattern::Custom(_) => (),
            pattern => {
                PATTERN_SUB_LINKS
                    .write()
                    .unwrap()
                    .insert((n, *pattern), sub_links.clone());
            }
        }
        sub_links
    }

    fn _get_sub_links(
        n_sudokus: usize,
        current_links: &CarpetLinks,
        already_explored_combinaisons: &mut HashSet<Vec<bool>>,
    ) -> Vec<CarpetLinks> {
        let mut sub_links = vec![];

        for &sudoku1 in current_links.clone().keys() {
            let mut testing_links = current_links.clone();

            let mut removed_sudokus = vec![sudoku1];
            while let Some(removed_sudoku) = removed_sudokus.pop() {
                // Remove sudoku1 -> .. links
                if testing_links.remove(&sudoku1).is_none() {
                    continue;
                }

                // Remove all .. -> sudoku1 links
                for value in testing_links.values_mut() {
                    for (square1, sudoku2, square2) in value.clone() {
                        if removed_sudoku == sudoku2 {
                            value.remove(&(square1, sudoku2, square2));
                        }
                    }
                }

                // clean up empty links
                for key in testing_links.keys().cloned().collect::<Vec<_>>() {
                    if testing_links[&key].is_empty() {
                        removed_sudokus.push(key);
                    }
                }
            }

            let current_combinaisons = (0..n_sudokus)
                .map(|i| testing_links.contains_key(&i))
                .collect::<Vec<_>>();

            if testing_links.is_empty()
                || !already_explored_combinaisons.insert(current_combinaisons.clone())
            {
                continue;
            }

            // Check if i can get every keys from one link
            let mut sudokus_got = HashSet::new();
            sudokus_got.insert(*testing_links.iter().next().unwrap().0);
            let mut changed_sudokus_got = true;
            while changed_sudokus_got {
                changed_sudokus_got = false;
                for sudoku_got in sudokus_got.clone() {
                    for (_, new_sudoku, _) in testing_links.get(&sudoku_got).unwrap() {
                        if sudokus_got.insert(*new_sudoku) {
                            changed_sudokus_got = true;
                        }
                    }
                }
            }

            if sudokus_got.len() != testing_links.len() {
                continue;
            }

            sub_links.extend(Self::_get_sub_links(
                n_sudokus,
                &testing_links,
                already_explored_combinaisons,
            ));
            sub_links.push(testing_links);
        }

        sub_links
    }
}

impl std::fmt::Display for CarpetPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CarpetPattern::Simple => write!(f, "Simple"),
            CarpetPattern::Samurai => write!(f, "Samurai"),
            CarpetPattern::Diagonal(size) => write!(f, "Diagonal({size})"),
            CarpetPattern::DenseDiagonal(size) => write!(f, "DenseDiagonal({size})"),
            CarpetPattern::Carpet(size) => write!(f, "Carpet({size})"),
            CarpetPattern::DenseCarpet(size) => write!(f, "DenseCarpet({size})"),
            CarpetPattern::Thorus(size) => write!(f, "Thorus({size})"),
            CarpetPattern::DenseThorus(size) => write!(f, "DenseThorus({size})"),
            CarpetPattern::Custom(size) => write!(f, "Custom({size})"),
        }
    }
}
