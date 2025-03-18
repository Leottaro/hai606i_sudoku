// @generated automatically by Diesel CLI.

diesel::table! {
    simple_sudokus (id) {
        id -> Integer,
        n -> Unsigned<Tinyint>,
        board -> Tinyblob,
        difficulty -> Unsigned<Tinyint>,
    }
}
