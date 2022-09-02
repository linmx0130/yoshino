use yoshino_core::db::DbDataType;

use crate::SQLiteAdaptor;

fn get_test_fields() -> Vec<(String, DbDataType)> {
    vec![
        ("row_id".to_string(), DbDataType::RowID),
        ("name".to_string(), DbDataType::Text),
        ("desc".to_string(), DbDataType::NullableText),
        ("counter".to_string(), DbDataType::Int)
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
    assert_eq!(stmt, "INSERT INTO test_table_name (row_id, name, desc, counter) VALUES (?1, ?2, ?3, ?4);")
}

#[test]
fn test_query_clause() {
    let stmt = SQLiteAdaptor::get_query_clause(TEST_TABLE_NAME, &get_test_fields());
    assert_eq!(stmt, "SELECT row_id, name, desc, counter FROM test_table_name");
}
