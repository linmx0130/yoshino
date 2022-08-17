/// SQLite adaptor for Project Yoshino
use yoshino_core::Schema;
use yoshino_core::db::{DbAdaptor, DbError};
use libsqlite3_sys::{sqlite3, sqlite3_stmt};
use std::ptr;
use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ops::Drop;

pub struct SQLiteAdaptor {
    db_handler: *mut sqlite3
}

impl SQLiteAdaptor {
    pub fn open(filename: &str) -> SQLiteAdaptor {
        let filename_cstring = CString::new(filename).unwrap();
        let mut db_handler: *mut sqlite3 = ptr::null_mut();
        unsafe {
            libsqlite3_sys::sqlite3_open(filename_cstring.as_ptr(), &mut db_handler);
        }
        SQLiteAdaptor {
            db_handler
        }
    }
}

impl Drop for SQLiteAdaptor {
    fn drop(&mut self) {
        unsafe {
            libsqlite3_sys::sqlite3_close(self.db_handler);
        }
    }
}

impl DbAdaptor for SQLiteAdaptor {
    fn create_table_for_schema<T: Schema>(&mut self) -> Result<(), DbError>{
        let create_table_stmt = T::create_table_stmt();
        let stmt_cstring = CString::new(create_table_stmt.as_str()).unwrap();
        let mut stmt : *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        unsafe {
            let r = libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                65536 as c_int,
                &mut stmt,
                &mut tail
            );
            let r2 = libsqlite3_sys::sqlite3_step(stmt);
            libsqlite3_sys::sqlite3_finalize(stmt);
        };
        Ok(())
    }
    fn insert_record<T: Schema>(&mut self, record: T) {
        todo!();
        //self.conn.execute(&T::insert_value_stmt(), record.insert_value_params());
    }
}