#[macro_use]
#[cfg(feature = "database")]
extern crate diesel;

#[cfg(feature = "database")]
pub mod database;
pub mod simple_sudoku;
pub mod tests;

#[macro_export]
macro_rules! debug_only {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}
