use hai606i_sudoku::simple_sudoku::{ Sudoku, SudokuDifficulty };

fn main() {
    let mut time_samples = SudokuDifficulty::iter()
        .map(|diff| (diff, Vec::new()))
        .collect::<Vec<_>>();
    let iterations: usize = 100;

    let end_function = |time_samples: Vec<(SudokuDifficulty, Vec<u128>)>, iterations: usize| {
        for (difficulty, mut samples) in time_samples {
            samples.sort();

            let min = samples.first().unwrap_or(&0);
            let max = samples.last().unwrap_or(&0);

            let average = (samples.iter().sum::<u128>() as f32) / (iterations as f32);
            let median = samples.get(samples.len() / 2).unwrap_or(&0);

            println!(
                "Difficulty {}:\n\tmin: {}ms\n\tmax: {}ms\n\taverage {:.2} ms\n\tmedian: {}ms",
                difficulty,
                min,
                max,
                average,
                median
            );
        }
    };

    for (i, difficulty) in SudokuDifficulty::iter().enumerate() {
        println!("testing difficulty {difficulty}{}", " ".repeat(50));

        for j in 0..iterations {
            println!("iteration {j}:{}", " ".repeat(50));

            let start = std::time::Instant::now();
            let _sudoku = Sudoku::generate_new(3, difficulty);
            time_samples[i].1.push(start.elapsed().as_millis());
        }
    }

    end_function(time_samples, iterations);
}
