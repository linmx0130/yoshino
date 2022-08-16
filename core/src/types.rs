/// It can be serialized as a database text field
pub trait TextField: Sized{
    fn from_db_data(data: &str) -> Option<Self>;
    fn to_db_data(&self) -> String;
    fn db_field_type() -> String {"TEXT NOT NULL".to_owned()}
}

impl TextField for String {
    fn from_db_data(data: &str) -> Option<String> {
        Some(String::from(data))
    }
    fn to_db_data(&self) -> String {
        self.to_owned()
    }
}

pub trait NullableTextField: Sized {
    fn from_db_data(data: &Option<String>) -> Option<Option<String>>;
    fn to_db_data(&self) -> Option<String>;
    fn db_field_type() -> String {"TEXT".to_owned()}
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

/// It can be serailized as an integer
pub trait IntegerField: Sized {
    fn from_db_data(data: i64) -> Option<Self>;
    fn to_db_data(&self)-> i64;
}

/// The mark trait to indicate that this type can be directly obtained from data base.
pub trait DbData {}
impl DbData for String {}
impl DbData for i64 {}
impl DbData for Option<String> {}

/// Make the type a data schema in the relational database.
pub trait Schema {
    /// Get the SQL statement to create a table for this data type.
    fn create_table_stmt() -> String;
    /// Get the SQL statement to insert a value to the table for this data type.
    fn insert_value_stmt() -> String;
    /// Get the parameter list for insert value statement.
    fn insert_value_params(&self) -> Vec<Box<dyn DbData>>;
}

pub struct DbError(String);
pub trait DbAdaptor {
    fn create_table_for_schema<T: Schema>(&mut self) -> Result<(), DbError>;
    fn insert_record<T: Schema>(&mut self, record: T);
}
