use std::time::Duration;

#[macro_use]
#[cfg(feature = "database")]
extern crate diesel;

pub mod carpet_sudoku;
#[cfg(feature = "database")]
pub mod database;
pub mod display;
pub mod simple_sudoku;
pub mod tests;

#[macro_export]
macro_rules! debug_only {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

pub fn duration_to_string(duration: &Duration) -> String {
    let milliseconds = duration.as_millis();
    let seconds = milliseconds / 1000;
    let minutes = milliseconds / 60_000;
    let hours = milliseconds / 3_600_000;
    if hours > 0 {
        format!(
            "{}h {}m {}.{:03}s",
            hours,
            minutes % 60,
            seconds % 60,
            milliseconds % 1000
        )
    } else if minutes > 0 {
        format!(
            "{}m {}.{:03}s",
            minutes % 60,
            seconds % 60,
            milliseconds % 1000
        )
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds % 60, milliseconds % 1000)
    } else {
        format!("{}ms", milliseconds % 1000)
    }
}
