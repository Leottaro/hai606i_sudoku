# HAI606I Sudoku Project

This project is a Sudoku application written in Rust. It includes features such as Sudoku generation, solving, and database integration for storing and retrieving Sudoku puzzles. It also supports multiple carpet patterns, allowing for diverse Sudoku configurations.

## Requirements

To run this project, you need the following installed on your system:

1. **Rust**: Install Rust from [rust-lang.org](https://www.rust-lang.org/).
2. **PostgreSQL**: Install [PostgreSQL](https://www.postgresql.org/download/) for database support.
3. **Diesel CLI**: If you want to set up your own database, install Diesel CLI by running:
    ```bash
    cargo install diesel_cli --no-default-features --features postgres
    ```

## Setting Up the Database

To set up your own database:

1. Create a PostgreSQL database.
2. Configure the database URL in a `.env` file:
    ```env
    DATABASE_URL=postgres://username:password@localhost/database_name
    ```
3. Run Diesel migrations to initialize the database schema:
    ```bash
    diesel setup
    diesel migration run
    ```

## Features

-   **Database Integration**: Enable the `database` feature to store and retrieve puzzles from a PostgreSQL database.
-   **Sudoku Variants**: Supports different Sudoku patterns such as Samurai, Torus, and Diagonal.
-   **Interactive UI**: Play and analyze Sudoku puzzles with an intuitive interface.

## Key Shortcuts

The following key shortcuts are available in the Sudoku application (defined in `src/display/display.rs`):

-   **1-9**: Input numbers into the selected cell.
-   **N**: Toggle notes mode.
-   **F**: Fill notes for the current puzzle.
-   **U**: Undo the last action.
-   **Escape**: Deselect the currently selected cell.
-   **A**: Switch to "Analyse" mode.
-   **P**: Switch to "Play" mode.
-   **S**: Solve the puzzle.
-   **Arrow Keys**: Navigate between cells.
-   **Alt + Arrow Keys**: Move the view of a Torus sudoku.

## Building and Running

To build and run the project:

1. Clone the repository.
2. Install the required dependencies.
3. Run the main application:
    ```bash
    cargo run --release
    ```

## Notes

-   Ensure the `DATABASE_URL` is correctly set in your `.env` file if using the database feature.
-   Use the `--features database` flag to enable database-related functionality:
    ```bash
    cargo run --release --features database
    ```

Enjoy solving Sudoku puzzles!

## Executables

The project includes multiple executables:

1. **`hai606i_sudoku`**: The main Sudoku application to create, browse and explore sudokus.
2. **`fill_database`**: Requires the `database` feature. Populates the database with Sudoku puzzles.
3. **`generation_benchmark`**: Benchmarks the Sudoku generation process.

To run a specific executable, use:

```bash
cargo run [--release] --bin <executable_name>
```
