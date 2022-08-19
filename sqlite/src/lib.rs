/// SQLite adaptor for Project Yoshino
use yoshino_core::Schema;
use yoshino_core::db::{DbAdaptor, DbDataType, DbError};
use libsqlite3_sys::{sqlite3, sqlite3_stmt};
use std::ptr;
use std::ffi::CString;
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

    fn get_create_table_stmt_code(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut s = format!("CREATE TABLE IF NOT EXISTS {} (", schema_name);
        for i in 0..fields.len() {
            if i != 0 {
                s = s + ", ";
            }
            let (field_name, field_type) = fields.get(i).unwrap();
            s = s + field_name + " ";
            s = s + match  field_type {
                DbDataType::Int => " INTEGER NOT NULL",
                DbDataType::NullableInt => "INTEGER",
                DbDataType::Text => "TEXT NOT NULL",
                DbDataType::NullableText => "TEXT",
                DbDataType::RowID => "INTEGER PRIMARY KEY"
            }
        }
        s = s + ");";
        s
    }

    fn get_insert_value_stmt_code(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut s = format!("INSERT INTO {} (", schema_name);
        for i in 0..fields.len() {
            if i != 0 {
                s = s + ", ";
            }
            let (field_name, _) = fields.get(i).unwrap();
            s = s + &field_name;
        }
        s = s + ") VALUES (";
        for i in 0..fields.len() {
            if i != 0 {
                s = s + ", ";
            }
            s = s + format!("?{}", i+1).as_ref();
        }
        s = s + ");";
        s
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
        let schema_name = T::get_schema_name();
        let fields = T::get_fields();
        let create_table_stmt = SQLiteAdaptor::get_create_table_stmt_code(&schema_name, &fields);
        let stmt_cstring = CString::new(create_table_stmt.as_str()).unwrap();
        let mut stmt : *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        unsafe {
            // TODO: check result value and generate errors
            let r = libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                create_table_stmt.len() as c_int,
                &mut stmt,
                &mut tail
            );
            let r2 = libsqlite3_sys::sqlite3_step(stmt);
            libsqlite3_sys::sqlite3_finalize(stmt);
        };
        Ok(())
    }
    fn insert_record<T: Schema>(&mut self, record: T) -> Result<(), DbError>{
        let schema_name = T::get_schema_name();
        let fields = T::get_fields();
        let insert_record_stmt = SQLiteAdaptor::get_insert_value_stmt_code(&schema_name, &fields);
        let stmt_cstring = CString::new(insert_record_stmt.as_str()).unwrap();
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        let params = record.get_values();
        unsafe {
            let r = libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                insert_record_stmt.len() as c_int,
                &mut stmt, 
            &mut tail);
        }
        for ii in 0..params.len() {
            let db_data_box = params.get(ii).unwrap();
            let i = (ii+1) as i32;
            unsafe{
                match db_data_box.db_data_type() {
                    yoshino_core::db::DbDataType::Int => {
                        let data_ptr = db_data_box.db_data_ptr() as *const i64;
                        let data_value = *data_ptr;
                        libsqlite3_sys::sqlite3_bind_int64(stmt, i, data_value);
                    }
                    yoshino_core::db::DbDataType::NullableInt | yoshino_core::db::DbDataType::RowID => {
                        let data_ptr = db_data_box.db_data_ptr() as *const i64;
                        if data_ptr != ptr::null() {
                            let data_value = *data_ptr;
                            libsqlite3_sys::sqlite3_bind_int64(stmt, i, data_value);
                        } else {
                            libsqlite3_sys::sqlite3_bind_null(stmt, i);
                        }
                    }
                    yoshino_core::db::DbDataType::Text | yoshino_core::db::DbDataType::NullableText => {
                        let data_ptr = db_data_box.db_data_ptr() as *const i8;
                        let data_len = db_data_box.db_data_len();
                        libsqlite3_sys::sqlite3_bind_text(stmt, i, data_ptr, data_len as i32, libsqlite3_sys::SQLITE_TRANSIENT());
                    }
                }
            }
        }
        unsafe{
            let r =libsqlite3_sys::sqlite3_step(stmt);
            let r = libsqlite3_sys::sqlite3_finalize(stmt);
        }
        Ok(())
    }
}