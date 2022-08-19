/// It can be serialized as a database text field
use crate::db::{DbData, DbDataType};

pub trait TextField: Sized{
    fn from_db_data(data: &str) -> Option<Self>;
    fn to_db_data(&self) -> String;
    fn db_field_type() -> DbDataType {
        DbDataType::Text
    }
}

pub trait NullableTextField: Sized {
    fn from_db_data(data: &Option<String>) -> Option<Option<String>>;
    fn to_db_data(&self) -> Option<String>;
    fn db_field_type() -> DbDataType {
        DbDataType::NullableText
    }
}

/// It can be serailized as a 64-bit integer
pub trait IntegerField: Sized {
    fn from_db_data(data: i64) -> Option<Self>;
    fn to_db_data(&self)-> i64;
    fn db_field_type() -> DbDataType {
        DbDataType::Int
    }
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
    pub fn db_field_type() -> DbDataType {
        DbDataType::RowID
    }
}

/// Make the type a data schema in the relational database.
pub trait Schema {
    fn get_schema_name() -> String;
    fn get_fields() -> Vec<(String, DbDataType)>;
    fn get_values(&self) -> Vec<Box<dyn DbData>>;
}
