use std::ptr;

use crate::RowID;

// Database related core stuff
#[derive(Debug, Clone)]
pub struct DbError(String);
pub trait DbAdaptor {
    fn create_table_for_schema<T: crate::types::Schema>(&mut self) -> Result<(), DbError>;
    fn insert_record<T: crate::types::Schema>(&mut self, record: T) -> Result<(), DbError>;
}

pub enum DbDataType {
    NullableText,
    NullableInt,
    Text,
    Int 
}
/// The mark trait to indicate that this type can be directly obtained from data base.
pub trait DbData {
    /// data type in `DbDataType`
    fn db_data_type(&self) -> DbDataType;
    /// pointer to the data
    fn db_data_ptr(&self) -> *const core::ffi::c_void;
    /// data length in bytes
    fn db_data_len(&self) -> usize;
}

impl DbData for String {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::Text
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        self.as_ptr() as *const core::ffi::c_void
    }

    fn db_data_len(&self) -> usize {
        self.len()
    }
}
impl DbData for i64 {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::Int
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        *self as *const core::ffi::c_void
    }

    fn db_data_len(&self) -> usize {
        8
    }
}

impl DbData for Option<String> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableText
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            None => ptr::null(),
            Some(s) => s.as_ptr() as *const core::ffi::c_void
        }
    }

    fn db_data_len(&self) -> usize {
        match self {
            None => 0,
            Some(s) => s.len()
        }
    }
}

impl DbData for crate::types::RowID {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableInt
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            RowID::NEW => ptr::null(),
            RowID::ID(v) => *v as *const core::ffi::c_void
        }
    }

    fn db_data_len(&self) -> usize {
        8
    }
}