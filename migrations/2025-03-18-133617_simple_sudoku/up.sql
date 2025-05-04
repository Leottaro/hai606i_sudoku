CREATE TABLE IF NOT EXISTS canonical_sudokus (
	filled_board_hash BIGINT NOT NULL,
	sudoku_n SMALLINT NOT NULL,
	canonical_board BYTEA NOT NULL,
	PRIMARY KEY (filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_sudoku_n ON canonical_sudokus(sudoku_n);
CREATE INDEX IF NOT EXISTS idx_canonical_board ON canonical_sudokus(canonical_board);

CREATE TABLE IF NOT EXISTS canonical_sudoku_squares (
	square_filled_board_hash BIGINT NOT NULL,
	square_id SMALLINT NOT NULL,
	square_hash BIGINT NOT NULL,
	PRIMARY KEY (square_filled_board_hash, square_id),
	FOREIGN KEY (square_filled_board_hash) REFERENCES canonical_sudokus(filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_square_filled_board_hash ON canonical_sudoku_squares(square_filled_board_hash);
CREATE INDEX IF NOT EXISTS idx_square_id ON canonical_sudoku_squares(square_id);
CREATE INDEX IF NOT EXISTS idx_square_hash ON canonical_sudoku_squares(square_hash);

CREATE TABLE IF NOT EXISTS canonical_sudoku_games (
	sudoku_game_id SERIAL,
	sudoku_game_filled_board_hash BIGINT NOT NULL,
	sudoku_game_difficulty SMALLINT NOT NULL,
	sudoku_game_filled_cells BYTEA NOT NULL,
	sudoku_game_filled_cells_count SMALLINT NOT NULL,
	UNIQUE (sudoku_game_filled_board_hash, sudoku_game_filled_cells),
	PRIMARY KEY (sudoku_game_id),
	FOREIGN KEY (sudoku_game_filled_board_hash) REFERENCES canonical_sudokus(filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_sudoku_game_filled_board_hash ON canonical_sudoku_games(sudoku_game_filled_board_hash);
CREATE INDEX IF NOT EXISTS idx_sudoku_game_difficulty ON canonical_sudoku_games(sudoku_game_difficulty);
CREATE INDEX IF NOT EXISTS idx_sudoku_game_filled_cells ON canonical_sudoku_games(sudoku_game_filled_cells);
CREATE INDEX IF NOT EXISTS idx_sudoku_game_filled_cells_count ON canonical_sudoku_games(sudoku_game_filled_cells_count);

CREATE TABLE IF NOT EXISTS canonical_carpets (
	carpet_filled_board_hash BIGINT NOT NULL,
	carpet_n SMALLINT NOT NULL,
	carpet_sudoku_number SMALLINT NOT NULL,
	carpet_pattern SMALLINT NOT NULL,
	carpet_pattern_size SMALLINT,
	PRIMARY KEY (carpet_filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_carpet_n ON canonical_carpets(carpet_n);
CREATE INDEX IF NOT EXISTS idx_carpet_sudoku_number ON canonical_carpets(carpet_sudoku_number);
CREATE INDEX IF NOT EXISTS idx_carpet_pattern ON canonical_carpets(carpet_pattern);
CREATE INDEX IF NOT EXISTS idx_carpet_pattern_size ON canonical_carpets(carpet_pattern_size);

CREATE TABLE IF NOT EXISTS canonical_carpet_sudokus (
	carpet_sudoku_carpet_filled_board_hash BIGINT NOT NULL,
	carpet_sudoku_i SMALLINT NOT NULL,
	carpet_sudoku_filled_board_hash BIGINT NOT NULL,
	PRIMARY KEY (carpet_sudoku_carpet_filled_board_hash, carpet_sudoku_i, carpet_sudoku_filled_board_hash),
	FOREIGN KEY (carpet_sudoku_carpet_filled_board_hash) REFERENCES canonical_carpets(carpet_filled_board_hash),
	FOREIGN KEY (carpet_sudoku_filled_board_hash) REFERENCES canonical_sudokus(filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_carpet_sudoku_carpet_filled_board_hash ON canonical_carpet_sudokus(carpet_sudoku_carpet_filled_board_hash);
CREATE INDEX IF NOT EXISTS idx_carpet_sudoku_i ON canonical_carpet_sudokus(carpet_sudoku_i);
CREATE INDEX IF NOT EXISTS idx_carpet_sudoku_filled_board_hash ON canonical_carpet_sudokus(carpet_sudoku_filled_board_hash);

CREATE TABLE IF NOT EXISTS canonical_carpet_games (
	carpet_game_id SERIAL,
	carpet_game_carpet_filled_board_hash BIGINT NOT NULL,
	carpet_game_difficulty SMALLINT NOT NULL,
	carpet_game_difficulty_score SMALLINT NOT NULL,
	carpet_game_filled_cells BYTEA NOT NULL,
	carpet_game_filled_cells_count SMALLINT NOT NULL,
	UNIQUE (carpet_game_id, carpet_game_filled_cells),
	PRIMARY KEY (carpet_game_id),
	FOREIGN KEY (carpet_game_carpet_filled_board_hash) REFERENCES canonical_carpets(carpet_filled_board_hash)
);
CREATE INDEX IF NOT EXISTS idx_carpet_game_carpet_filled_board_hash ON canonical_carpet_games(carpet_game_carpet_filled_board_hash);
CREATE INDEX IF NOT EXISTS idx_carpet_game_difficulty ON canonical_carpet_games(carpet_game_difficulty);
CREATE INDEX IF NOT EXISTS idx_carpet_game_difficulty_score ON canonical_carpet_games(carpet_game_difficulty_score);
CREATE INDEX IF NOT EXISTS idx_carpet_game_filled_cells ON canonical_carpet_games(carpet_game_filled_cells);
CREATE INDEX IF NOT EXISTS idx_carpet_game_filled_cells_count ON canonical_carpet_games(carpet_game_filled_cells_count);