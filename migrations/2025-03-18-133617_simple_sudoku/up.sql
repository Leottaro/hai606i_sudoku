CREATE TABLE IF NOT EXISTS canonical_sudokus (
	filled_board_hash BIGINT NOT NULL,
	sudoku_n SMALLINT NOT NULL,
	canonical_board BYTEA NOT NULL,
	PRIMARY KEY (filled_board_hash)
);

CREATE TABLE IF NOT EXISTS canonical_sudoku_squares (
	square_filled_board_hash BIGINT NOT NULL,
	square_id SMALLINT NOT NULL,
	square_hash BIGINT NOT NULL,
	PRIMARY KEY (square_filled_board_hash, square_id),
	FOREIGN KEY (square_filled_board_hash) REFERENCES canonical_sudokus(filled_board_hash)
);

CREATE TABLE IF NOT EXISTS canonical_sudoku_games (
	game_id SERIAL,
	game_filled_board_hash BIGINT NOT NULL,
	game_n SMALLINT NOT NULL,
	game_board BYTEA UNIQUE NOT NULL,
	game_difficulty SMALLINT NOT NULL,
	game_filled_cells SMALLINT NOT NULL,
	PRIMARY KEY (game_id),
	FOREIGN KEY (game_filled_board_hash) REFERENCES canonical_sudokus(filled_board_hash)
);