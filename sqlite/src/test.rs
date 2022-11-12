use yoshino_core::db::DbDataType;

use crate::SQLiteAdaptor;

fn get_test_fields() -> Vec<(String, DbDataType)> {
    vec![
        ("row_id".to_string(), DbDataType::RowID),
        ("name".to_string(), DbDataType::Text),
        ("desc".to_string(), DbDataType::NullableText),
        ("counter".to_string(), DbDataType::Int),
    ]
}
const TEST_TABLE_NAME: &str = "test_table_name";

#[test]
fn test_create_table_stmt_creation() {
    let stmt = SQLiteAdaptor::get_create_table_stmt_code(TEST_TABLE_NAME, &get_test_fields());
    assert_eq!(stmt, "CREATE TABLE IF NOT EXISTS test_table_name (row_id INTEGER PRIMARY KEY, name TEXT NOT NULL, desc TEXT, counter INTEGER NOT NULL);");
}

#[test]
fn test_insert_value_stmt_creation() {
    let stmt = SQLiteAdaptor::get_insert_value_stmt_code(TEST_TABLE_NAME, &get_test_fields());
    assert_eq!(
        stmt,
        "INSERT INTO test_table_name (row_id, name, desc, counter) VALUES (?1, ?2, ?3, ?4);"
    )
}

#[test]
fn test_query_clause() {
    let stmt = SQLiteAdaptor::get_query_clause(TEST_TABLE_NAME, &get_test_fields());
    assert_eq!(
        stmt,
        "SELECT row_id, name, desc, counter FROM test_table_name"
    );
}

#[test]
fn test_update_clause() {
    let stmt = SQLiteAdaptor::get_update_clause(TEST_TABLE_NAME, &get_test_fields());
    assert_eq!(
        stmt,
        "UPDATE test_table_name SET row_id = ?, name = ?, desc = ?, counter = ?"
    );
}

mod cond_parsing_test {
    use crate::SQLiteAdaptor;
    use yoshino_core::{db::DbData, Cond};

    #[test]
    fn test_int_eq_cond() {
        let cond = Cond::integer_equal_to("value", 0xff);
        let (clause, params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        assert_eq!(clause, "value=?");
        assert_eq!(i64::from_boxed_db_data(&params[0]), 0xff);
    }

    #[test]
    fn test_int_not_eq_cond() {
        let cond = Cond::integer_not_equal_to("value", 0xff);
        let (clause, params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        assert_eq!(clause, "value<>?");
        assert_eq!(i64::from_boxed_db_data(&params[0]), 0xff);
    }

    #[test]
    fn test_and_cond() {
        let cond = Cond::and(
            Cond::integer_equal_to("value1", 0xf0),
            Cond::text_equal_to("value2", "str"),
        );
        let (clause, params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        assert_eq!(clause, "(value1=?) AND (value2=?)");
        assert_eq!(i64::from_boxed_db_data(&params[0]), 0xf0);
        assert_eq!(String::from_boxed_db_data(&params[1]), "str");
    }

    #[test]
    fn test_or_cond() {
        let cond = Cond::or(
            Cond::integer_equal_to("value1", 0xf0),
            Cond::text_equal_to("value2", "str"),
        );
        let (clause, params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        assert_eq!(clause, "(value1=?) OR (value2=?)");
        assert_eq!(i64::from_boxed_db_data(&params[0]), 0xf0);
        assert_eq!(String::from_boxed_db_data(&params[1]), "str");
    }

    #[test]
    fn test_not_cond() {
        let cond = Cond::not(Cond::integer_equal_to("value1", 0xf0));
        let (clause, params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        assert_eq!(clause, "NOT (value1=?)");
        assert_eq!(i64::from_boxed_db_data(&params[0]), 0xf0);
    }
}
