// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Enum"))]
    pub struct SimpleSudokusDifficultyEnum;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SimpleSudokusDifficultyEnum;

    simple_sudokus (id) {
        id -> Integer,
        n -> Unsigned<Tinyint>,
        board -> Tinyblob,
        #[max_length = 7]
        difficulty -> SimpleSudokusDifficultyEnum,
    }
}
