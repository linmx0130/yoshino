/// It can be serialized as a database text field
use crate::db::DbData;
pub trait TextField: Sized{
    fn from_db_data(data: &str) -> Option<Self>;
    fn to_db_data(&self) -> String;
    fn db_field_type() -> &'static str {"TEXT NOT NULL"}
}

pub trait NullableTextField: Sized {
    fn from_db_data(data: &Option<String>) -> Option<Option<String>>;
    fn to_db_data(&self) -> Option<String>;
    fn db_field_type() -> &'static str {"TEXT"}
}

/// It can be serailized as a 64-bit integer
pub trait IntegerField: Sized {
    fn from_db_data(data: i64) -> Option<Self>;
    fn to_db_data(&self)-> i64;
    fn db_field_type() -> &'static str {"BIGINT NOT NULL"}
}

impl TextField for String {
    fn from_db_data(data: &str) -> Option<String> {
        Some(String::from(data))
    }
    fn to_db_data(&self) -> String {
        self.to_owned()
    }
}

impl NullableTextField for Option<String> {
    fn from_db_data(data: &Option<String>) -> Option<Option<String>> {
        match data {
            Some(x) => {
                Some(Some(x.to_owned()))
            }
            None => {
                Some(None)
            }
        }
    }
    fn to_db_data(&self) -> Option<String> {
        match self {
            None => None,
            Some(x) => Some(x.to_owned())
        }
    }
}

/// Auto increment row ID field. It will be represented as an integer primary key.
#[derive(Clone, Copy)]
pub enum RowID {
    NEW,
    ID(i64)
}
impl RowID {
    pub fn from_db_data(data: i64) -> RowID{
        RowID::ID(data)
    }
    pub fn to_db_data(&self) -> RowID {
        self.clone()
    }
    pub fn db_field_type() -> &'static str {"INTEGER PRIMARY KEY"}
}

/// Make the type a data schema in the relational database.
pub trait Schema {
    /// Get the SQL statement to create a table for this data type.
    fn create_table_stmt() -> String;
    /// Get the SQL statement to insert a value to the table for this data type.
    fn insert_value_stmt() -> String;
    /// Get the parameter list for insert value statement.
    fn insert_value_params(&self) -> Vec<Box<dyn DbData>>;
}
