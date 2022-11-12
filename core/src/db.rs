//! Database related core stuff
use crate::query_cond::Cond;
use crate::{RowID};
use std::ptr;

/// Database error
#[derive(Debug, Clone)]
pub struct DbError(pub String);

/// Yoshino database adaptor trait.
///
/// Every database adaptor implementation should implement this trait.
pub trait DbAdaptor {
    type Iterator<T: crate::types::Schema>: Iterator<Item = T>;
    /// Create data table in the database for a Yoshino schema.
    fn create_table_for_schema<T: crate::types::Schema>(&mut self) -> Result<(), DbError>;
    /// Insert a record to the database.
    fn insert_record<T: crate::types::Schema>(&mut self, record: T) -> Result<(), DbError>;
    /// Query all records of the schema.
    fn query_all<T: crate::types::Schema>(&mut self) -> Result<Self::Iterator<T>, DbError>;
    /// Query records of the schema that matches the condition.
    fn query_with_cond<T: crate::types::Schema>(
        &mut self,
        cond: Cond,
    ) -> Result<Self::Iterator<T>, DbError>;
    /// Delete records of the schema that matches the condition.
    fn delete_with_cond<T: crate::types::Schema>(&mut self, cond: Cond) -> Result<(), DbError>;
    /// Update records of the schema that matches the condition.
    fn update_with_cond<T: crate::types::Schema>(
        &mut self,
        cond: Cond,
        record: T,
    ) -> Result<(), DbError>;
}

/// Database data type supported by Yoshino.
pub enum DbDataType {
    NullableText,
    NullableInt,
    Text,
    Int,
    Float,
    Binary,
    NullableBinary,
    RowID 
}

/// The mark trait to indicate that this type can be directly obtained from / sent to the databases.
pub trait DbData {
    /// Data type in `DbDataType`
    fn db_data_type(&self) -> DbDataType;
    /// A pointer to the data. The data should be raw data in contiguous memory.
    /// For null values, it should return a null pointer. 
    fn db_data_ptr(&self) -> *const core::ffi::c_void;
    /// Data length in bytes.
    fn db_data_len(&self) -> usize;
    /// To restore data from a boxed db data object. 
    /// A copy of the value should be created as this method only takes a reference as the parameter.
    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Self where Self: Sized;
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

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> String {
        unsafe {
            {
                let str_len = src.db_data_len();
                let str_copy = libc::malloc(str_len) as *mut i8;
                libc::strncpy(str_copy, src.db_data_ptr() as *mut i8, str_len);
                String::from_raw_parts(str_copy as *mut u8, str_len, str_len)
            }
        }
    }
}

impl DbData for i64 {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::Int
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        self as *const i64 as *const core::ffi::c_void
    }

    fn db_data_len(&self) -> usize {
        8
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> i64 {
        unsafe { *(src.db_data_ptr() as *const i64) }
    }
}

impl DbData for Option<i64> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableInt
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            None => ptr::null(),
            Some(v) => v as *const i64 as *const core::ffi::c_void
        }
    }

    fn db_data_len(&self) -> usize {
        8
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Option<i64> {
        if src.db_data_ptr().is_null() {
            None
        } else {
            Some(unsafe { *(src.db_data_ptr() as *const i64) })
        }
    }
}

impl DbData for Option<String> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableText
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            None => ptr::null(),
            Some(s) => s.as_ptr() as *const core::ffi::c_void,
        }
    }

    fn db_data_len(&self) -> usize {
        match self {
            None => 0,
            Some(s) => s.len(),
        }
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Option<String> {
        if src.db_data_ptr().is_null() {
            None
        } else {
            Some(unsafe {
                let str_len = src.db_data_len();
                let str_copy = libc::malloc(str_len) as *mut i8;
                libc::strncpy(str_copy, src.db_data_ptr() as *mut i8, str_len);
                String::from_raw_parts(str_copy as *mut u8, str_len, str_len)
            })
        }
    }
}

impl DbData for crate::types::RowID {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::RowID
    }
    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            RowID::NEW => ptr::null(),
            RowID::ID(v) => v as *const i64 as *const core::ffi::c_void,
        }
    }
    fn db_data_len(&self) -> usize {
        8
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> RowID {
        if src.db_data_ptr().is_null() {
            RowID::NEW
        } else {
            unsafe { RowID::ID(*(src.db_data_ptr() as *const i64)) }
        }
    }
}

impl DbData for f64 {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::Float
    }
    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        self as *const f64 as *const core::ffi::c_void
    }
    fn db_data_len(&self) -> usize {
        8
    }
    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Self
    where
        Self: Sized,
    {
        unsafe { *(src.db_data_ptr() as *const f64) }
    }
}

impl DbData for Vec<u8> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::Binary
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        self.as_ptr() as *mut u8 as *const core::ffi::c_void
    }

    fn db_data_len(&self) -> usize {
        self.len()
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Vec<u8> {
        let len = src.db_data_len();
        let ptr = src.db_data_ptr() as *const u8;
        let mut buf = Vec::with_capacity(len);
        for i in 0..len {
            unsafe{
                buf.push(*(ptr.offset(i as isize)))
            }
        }
        buf
    }
}

impl DbData for Option<Vec<u8>> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableBinary
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            None => ptr::null(),
            Some(v) => v.as_ptr() as *mut u8 as *const core::ffi::c_void
        }
    }

    fn db_data_len(&self) -> usize {
        match self {
            None => 0,
            Some(v) => v.len()
        }
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Self where Self: Sized {
        if src.db_data_ptr().is_null() {
            None
        } else {
            Some(Vec::<u8>::from_boxed_db_data(src))
        }
    }
}
