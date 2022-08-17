// Database related core stuff
pub struct DbError(String);
pub trait DbAdaptor {
    fn create_table_for_schema<T: crate::types::Schema>(&mut self) -> Result<(), DbError>;
    fn insert_record<T: crate::types::Schema>(&mut self, record: T);
}

/// The mark trait to indicate that this type can be directly obtained from data base.
pub trait DbData {}
impl DbData for String {}
impl DbData for i64 {}
impl DbData for Option<String> {}