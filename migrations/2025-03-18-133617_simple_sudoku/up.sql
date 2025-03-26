CREATE TABLE IF NOT EXISTS simple_sudoku_canonical (
	canonical_board_hash BIGINT UNSIGNED NOT NULL,
	sudoku_n TINYINT UNSIGNED NOT NULL,
	canonical_board TINYBLOB NOT NULL,
	PRIMARY KEY (canonical_board_hash)
);

CREATE TABLE IF NOT EXISTS simple_sudoku_canonical_squares (
	square_canonical_board_hash BIGINT UNSIGNED NOT NULL,
	square_id TINYINT UNSIGNED NOT NULL,
	square_hash BIGINT UNSIGNED NOT NULL,
	PRIMARY KEY (square_canonical_board_hash, square_id),
	FOREIGN KEY (square_canonical_board_hash) REFERENCES simple_sudoku_canonical(canonical_board_hash)
);

CREATE TABLE IF NOT EXISTS simple_sudoku_games (
	game_id INT AUTO_INCREMENT,
	game_canonical_board_hash BIGINT UNSIGNED NOT NULL,
	game_n TINYINT UNSIGNED NOT NULL,
	game_board TINYBLOB NOT NULL,
	game_difficulty TINYINT UNSIGNED NOT NULL,
	game_filled_cells SMALLINT UNSIGNED NOT NULL,
	PRIMARY KEY (game_id),
	FOREIGN KEY (game_canonical_board_hash) REFERENCES simple_sudoku_canonical(canonical_board_hash)
);