CREATE TABLE IF NOT EXISTS simple_sudoku_filled (
	filled_board_hash BIGINT UNSIGNED NOT NULL,
	filled_n TINYINT UNSIGNED NOT NULL,
	filled_board TINYBLOB NOT NULL,
	filled_up_left_corner BIGINT UNSIGNED NOT NULL,
	filled_up_right_corner BIGINT UNSIGNED NOT NULL,
	filled_bottom_left_corner BIGINT UNSIGNED NOT NULL,
	filled_bottom_right_corner BIGINT UNSIGNED NOT NULL,
	PRIMARY KEY (filled_board_hash)
);

CREATE TABLE IF NOT EXISTS simple_sudoku_games (
	game_id INT AUTO_INCREMENT,
	filled_board_hash BIGINT UNSIGNED NOT NULL,
	game_n TINYINT UNSIGNED NOT NULL,
	game_board TINYBLOB NOT NULL,
	game_difficulty TINYINT UNSIGNED NOT NULL,
	PRIMARY KEY (game_id),
	FOREIGN KEY (filled_board_hash) REFERENCES simple_sudoku_filled(filled_board_hash)
);