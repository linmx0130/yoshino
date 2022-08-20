use std::ptr;

use crate::{RowID, Schema};

// Database related core stuff
#[derive(Debug, Clone)]
pub struct DbError(pub String);

pub struct DbQueryResult<T:Schema> {
    pub data_iter: Box<dyn Iterator<Item=T>>
}
impl<T:Schema> Iterator for DbQueryResult<T>{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.data_iter.next()
    }
}

pub trait DbAdaptor {
    fn create_table_for_schema<T: crate::types::Schema>(&mut self) -> Result<(), DbError>;
    fn insert_record<T: crate::types::Schema>(&mut self, record: T) -> Result<(), DbError>;
    fn query_all<T: crate::types::Schema>(&mut self) -> Result<DbQueryResult<T>, DbError>;
}

pub enum DbDataType {
    NullableText,
    NullableInt,
    Text,
    Int,
    RowID 
}
/// The mark trait to indicate that this type can be directly obtained from data base.
pub trait DbData {
    /// data type in `DbDataType`
    fn db_data_type(&self) -> DbDataType;
    /// pointer to the data
    fn db_data_ptr(&self) -> *const core::ffi::c_void;
    /// data length in bytes
    fn db_data_len(&self) -> usize;
    // restore data from a boxed db data object
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
        unsafe {
            *(src.db_data_ptr() as *const i64)
        }
    }
}

impl DbData for Option<i64> {
    fn db_data_type(&self) -> DbDataType {
        DbDataType::NullableInt
    }

    fn db_data_ptr(&self) -> *const core::ffi::c_void {
        match self {
            None => ptr::null(),
            Some(v) => (v as *const i64 as *const core::ffi::c_void)
        }
    }
    
    fn db_data_len(&self) -> usize {
        8
    }

    fn from_boxed_db_data(src: &Box<dyn DbData>) -> Option<i64> {
        if src.db_data_ptr().is_null() {
            None
        } else {
            Some(unsafe {
                *(src.db_data_ptr() as *const i64)
            })
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
            Some(s) => s.as_ptr() as *const core::ffi::c_void
        }
    }

    fn db_data_len(&self) -> usize {
        match self {
            None => 0,
            Some(s) => s.len()
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
            RowID::ID(_) => self as *const RowID as *const core::ffi::c_void
        }
    }
    fn db_data_len(&self) -> usize {
        8
    }
    
    fn from_boxed_db_data(src: &Box<dyn DbData>) -> RowID {
        if src.db_data_ptr().is_null() {
            RowID::NEW
        } else {
            unsafe{
                match *(src.db_data_ptr() as *const RowID) {
                    RowID::NEW => RowID::NEW,
                    RowID::ID(v) => RowID::ID(v)
                }
            }   
        }
    }
}