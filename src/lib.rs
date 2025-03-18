#[macro_use]
extern crate diesel;

pub mod database;
pub mod simple_sudoku;
pub mod tests;

#[macro_export]
macro_rules! debug_only {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}
