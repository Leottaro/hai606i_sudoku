use hai606i_sudoku::{
    duration_to_string,
    simple_sudoku::{Sudoku, SudokuDifficulty},
};
use std::{collections::HashMap, time::Duration};

fn main() {
    let mut time_samples = SudokuDifficulty::iter()
        .map(|diff| (diff, Vec::new()))
        .collect::<HashMap<_, _>>();
    let iterations: usize = 5;

    let end_function = |time_samples: HashMap<SudokuDifficulty, Vec<Duration>>,
                        iterations: usize| {
        for (difficulty, mut samples) in time_samples {
            samples.sort();
            let null_duration = Duration::from_millis(0);

            let min = samples.first().unwrap_or(&null_duration);
            let max = samples.last().unwrap_or(&null_duration);
            let average = samples.iter().sum::<Duration>() / iterations as u32;
            let median = samples.get(samples.len() / 2).unwrap_or(&null_duration);

            println!(
                "Difficulty {}:\n\tmin: {}ms\n\tmax: {}ms\n\taverage {:.2} ms\n\tmedian: {}ms",
                difficulty,
                duration_to_string(min),
                duration_to_string(max),
                duration_to_string(&average),
                duration_to_string(median)
            );
        }
    };

    for difficulty in SudokuDifficulty::iter() {
        println!("testing difficulty {difficulty}{}", " ".repeat(50));

        for j in 0..iterations {
            println!("iteration {j}:{}", " ".repeat(50));

            let start = std::time::Instant::now();
            let _sudoku = Sudoku::generate_new(3, difficulty);
            time_samples
                .get_mut(&difficulty)
                .unwrap()
                .push(start.elapsed());
        }
    }

    end_function(time_samples, iterations);
}
