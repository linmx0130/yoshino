//! SQLite adaptor for Project Yoshino
use yoshino_core::Schema;
use yoshino_core::db::{DbAdaptor, DbData, DbDataType, DbError, DbQueryResult};
use libsqlite3_sys::{sqlite3, sqlite3_stmt};
use std::ptr;
use std::ffi::CString;
use std::os::raw::{c_int, c_char};
use std::ops::Drop;
use std::marker::PhantomData;

pub struct SQLiteAdaptor {
    db_handler: *mut sqlite3
}

macro_rules! db_try {
    ($e: expr) => {{
        {
            let return_value = $e;
            match return_value {
                libsqlite3_sys::SQLITE_OK | libsqlite3_sys::SQLITE_DONE => {
                    // success, ignore it
                }
                error_code => {
                    return Err(DbError(format!("SQLite3 error {}", error_code)))
                }
            }
        }
    }};
}

impl SQLiteAdaptor {
    pub fn open(filename: &str) -> Result<SQLiteAdaptor, DbError> {
        let filename_cstring = CString::new(filename).unwrap();
        let mut db_handler: *mut sqlite3 = ptr::null_mut();
        unsafe {
            db_try!(libsqlite3_sys::sqlite3_open(filename_cstring.as_ptr(), &mut db_handler));
        }
        Ok(SQLiteAdaptor {
            db_handler
        })
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
                DbDataType::Int => "INTEGER NOT NULL",
                DbDataType::NullableInt => "INTEGER",
                DbDataType::Text => "TEXT NOT NULL",
                DbDataType::NullableText => "TEXT",
                DbDataType::Float => "REAL",
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

    fn get_query_clause(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut s = format!("SELECT ");
        for i in 0..fields.len() {
            if i != 0 {
                s = s + ", ";
            }
            let (field_name, _) = fields.get(i).unwrap();
            s = s + &field_name;
        }
        s = s + " FROM " + schema_name;
        s 
    }

    fn get_update_clause(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut s = format!("UPDATE {} SET ", schema_name);
        for i in 0..fields.len() {
            if i != 0 {
                s = s + ", "
            }
            let (field_name, _) = fields.get(i).unwrap();
            s = s + format!("{} = ?", field_name).as_ref();
        }
        s
    }

    fn get_condition_stmt_and_params(cond: yoshino_core::query_cond::Cond) -> (String, Vec<Box<dyn DbData>>) {
        use yoshino_core::query_cond::Cond::*;
        match cond {
            IsNull{field_name} => {
                (format!("{} IS NULL", field_name), vec![])
            }
            IsNotNull { field_name } => {
                (format!("{} IS NOT NULL", field_name), vec![])
            }
            IntegerEqualTo { field_name, value } => {
                (format!("{}=?", field_name), vec![Box::new(value)])
            }
            IntegerNotEqualTo { field_name, value } => {
                (format!("{}<>?", field_name), vec![Box::new(value)])
            }
            IntegerGreaterThan { field_name, value } => {
                (format!("{}>?", field_name), vec![Box::new(value)])
            }
            IntegerLessThan { field_name, value } => {
                (format!("{}<?", field_name), vec![Box::new(value)])
            }
            IntegerGreaterThanOrEqualTo { field_name, value } => {
                (format!("{}>=?", field_name), vec![Box::new(value)])
            }
            IntegerLessThanOrEqualTo { field_name, value } => {
                (format!("{}<=?", field_name), vec![Box::new(value)])
            }
            TextEqualTo { field_name, value } => {
                (format!("{}=?", field_name), vec![Box::new(value)])
            }
            And{left, right} => {
                let (left_stmt, left_params) = Self::get_condition_stmt_and_params(*left);
                let (right_stmt, right_params) = Self::get_condition_stmt_and_params(*right);
                let mut params = left_params;
                params.extend(right_params.into_iter());
                (format!("({}) AND ({})", left_stmt, right_stmt), params)
            }
            Or{left, right} => {
                let (left_stmt, left_params) = Self::get_condition_stmt_and_params(*left);
                let (right_stmt, right_params) = Self::get_condition_stmt_and_params(*right);
                let mut params = left_params;
                params.extend(right_params.into_iter());
                (format!("({}) OR ({})", left_stmt, right_stmt), params)
            }
            Not{cond} => {
                let (stmt, params) = Self::get_condition_stmt_and_params(*cond);          
                (format!("NOT ({})", stmt), params)
            }
        }
    }

    fn get_delete_clause(schema_name: &str) -> String {
        format!("DELETE FROM {}", schema_name)
    }

    fn bind_params_to_stmt(stmt: *mut sqlite3_stmt, params: &Vec<Box<dyn DbData>>) {
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
                    yoshino_core::db::DbDataType::Float => {
                        let data_ptr = db_data_box.db_data_ptr() as *const f64;
                        let data_value = *data_ptr;
                        libsqlite3_sys::sqlite3_bind_double(stmt, i, data_value);
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
    }
}

impl Drop for SQLiteAdaptor {
    fn drop(&mut self) {
        unsafe {
            libsqlite3_sys::sqlite3_close(self.db_handler);
        }
    }
}

pub struct SQLiteRowIterator<T: Schema + 'static> {
    stmt: *mut sqlite3_stmt,
    phantom: PhantomData<T>
}

impl<T: Schema> Iterator for SQLiteRowIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let r = unsafe {
            libsqlite3_sys::sqlite3_step(self.stmt)
        };
        match r {
            libsqlite3_sys::SQLITE_DONE => None,
            libsqlite3_sys::SQLITE_ROW => {
                let mut values:Vec<Box<dyn DbData>> = vec![];
                let fields = T::get_fields();
                for i in 0..fields.len() {
                    let (_, field_type) = fields.get(i).unwrap();
                    match field_type {
                        DbDataType::NullableInt => {
                            let type_code = unsafe {
                                libsqlite3_sys::sqlite3_column_type(self.stmt, i as i32)
                            };
                            let v = match type_code {
                                libsqlite3_sys::SQLITE_INTEGER => {
                                    Some(unsafe {
                                        libsqlite3_sys::sqlite3_column_int64(self.stmt, i as i32)
                                    })
                                }
                                _ => {
                                    None
                                }
                            };
                            values.push(Box::new(v));
                            
                        }
                        DbDataType::Int => {
                            let v = unsafe { libsqlite3_sys::sqlite3_column_int64(self.stmt, i as i32) as i64};
                            values.push(Box::new(v));               
                        }
                        DbDataType::Float => {
                            let v = unsafe {
                                libsqlite3_sys::sqlite3_column_double(self.stmt, i as i32) as f64
                            };
                            values.push(Box::new(v));
                        }
                        DbDataType::RowID => {
                            let v = unsafe { libsqlite3_sys::sqlite3_column_int64(self.stmt, i as i32) as i64};
                            values.push(Box::new(yoshino_core::RowID::ID(v)))
                        }
                        DbDataType::NullableText| DbDataType::Text => {
                            let v = unsafe { 
                                let str_ptr = libsqlite3_sys::sqlite3_column_text(self.stmt, i as i32) as *const c_char;
                                let str_len = libc::strlen(str_ptr);
                                let str_copy = libc::malloc(str_len) as *mut i8;
                                libc::strncpy(str_copy, str_ptr, str_len);
                                String::from_raw_parts(str_copy as *mut u8, str_len, str_len)
                            };
                            values.push(Box::new(v));
                        }
                    };
                }
                Some(T::create_with_values(values))
            }
            _ => None
        }
    }
}

impl<T:Schema> Drop for SQLiteRowIterator<T> {
    fn drop(&mut self) {
        unsafe {
            libsqlite3_sys::sqlite3_finalize(self.stmt);
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
            db_try!(libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                create_table_stmt.len() as c_int,
                &mut stmt,
                &mut tail
            ));
            db_try!(libsqlite3_sys::sqlite3_step(stmt));
            db_try!(libsqlite3_sys::sqlite3_finalize(stmt));
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
            db_try!(libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                insert_record_stmt.len() as c_int,
                &mut stmt, 
            &mut tail));
        }
        SQLiteAdaptor::bind_params_to_stmt(stmt, &params);
        unsafe{
            db_try!(libsqlite3_sys::sqlite3_step(stmt));
            db_try!(libsqlite3_sys::sqlite3_finalize(stmt));
        }
        Ok(())
    }

    fn query_all<T:Schema>(&mut self) -> Result<DbQueryResult<T>, DbError>{
        let schema_name = T::get_schema_name();
        let fields = T::get_fields();
        let query_stmt = SQLiteAdaptor::get_query_clause(&schema_name, &fields) + ";";
        let stmt_cstring = CString::new(query_stmt.as_str()).unwrap();
        let mut stmt : *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        unsafe {
            // TODO: check result value and generate errors
            db_try!(libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                query_stmt.len() as c_int,
                &mut stmt,
                &mut tail
            ));
        };
        let iter:Box<SQLiteRowIterator<T>> = Box::new(SQLiteRowIterator{stmt, phantom: PhantomData});
        Ok(DbQueryResult{data_iter: iter})
    }

    fn query_with_cond<T:Schema>(&mut self, cond: yoshino_core::query_cond::Cond) -> Result<DbQueryResult<T>, DbError> {
        let schema_name = T::get_schema_name();
        let fields = T::get_fields();
        let query_stmt = SQLiteAdaptor::get_query_clause(&schema_name, &fields);
        let (cond_stmt, cond_params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        let query_where_cond_stmt = format!("{} WHERE {};", query_stmt, cond_stmt);
        let stmt_cstring = CString::new(query_where_cond_stmt.as_str()).unwrap();
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        unsafe {
            db_try!(
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db_handler, 
                    stmt_cstring.as_ptr(), 
                    query_where_cond_stmt.len() as c_int,
                     &mut stmt,
                     &mut tail
                ));
            SQLiteAdaptor::bind_params_to_stmt(stmt, &cond_params);
        }
        let iter:Box<SQLiteRowIterator<T>> = Box::new(SQLiteRowIterator{stmt, phantom: PhantomData});
        Ok(DbQueryResult{data_iter: iter})
    }

    fn delete_with_cond<T: Schema>(&mut self, cond: yoshino_core::Cond) -> Result<(), DbError> {
        let schema_name = T::get_schema_name();
        let delete_clause = SQLiteAdaptor::get_delete_clause(&schema_name);
        let (cond_stmt, cond_params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        let delete_where_cond_stmt = format!("{} WHERE {};", delete_clause, cond_stmt);
        let stmt_cstring = CString::new(delete_where_cond_stmt.as_str()).unwrap();
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();
        unsafe {
            db_try!(libsqlite3_sys::sqlite3_prepare_v2(
                self.db_handler, 
                stmt_cstring.as_ptr(),
                delete_where_cond_stmt.len() as c_int,
                &mut stmt, &mut tail));
            SQLiteAdaptor::bind_params_to_stmt(stmt, &cond_params);
            db_try!(libsqlite3_sys::sqlite3_step(stmt));
            db_try!(libsqlite3_sys::sqlite3_finalize(stmt));
        }
        Ok(())
    }

    fn update_with_cond<T: Schema>(&mut self, cond:yoshino_core::Cond, record: T) -> Result<(), DbError> {
        let schema_name = T::get_schema_name();
        let fields = T::get_fields();
        let update_clause = SQLiteAdaptor::get_update_clause(&schema_name, &fields);
        let (cond_stmt, cond_params) = SQLiteAdaptor::get_condition_stmt_and_params(cond);
        let update_where_cond_stmt = format!("{} WHERE {};", update_clause, cond_stmt);
        let mut update_stmt_params = record.get_values();
        update_stmt_params.extend(cond_params);

        let stmt_cstring = CString::new(update_where_cond_stmt.as_str()).unwrap();
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let mut tail = ptr::null();

        unsafe {
            db_try!(
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db_handler,
                    stmt_cstring.as_ptr(),
                    update_where_cond_stmt.len() as c_int,
                    &mut stmt, &mut tail)
                );
            SQLiteAdaptor::bind_params_to_stmt(stmt, &update_stmt_params);
            db_try!(libsqlite3_sys::sqlite3_step(stmt));
            db_try!(libsqlite3_sys::sqlite3_finalize(stmt));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;