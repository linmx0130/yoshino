use std::os::raw::{c_ulong};
use yoshino_core::db::{DbAdaptor, DbError, DbDataType};
use std::ffi::{CString, CStr};
use std::ptr;

macro_rules! db_stmt_try {
    ($stmt: ident, $e: expr) => {{
        {
            let return_value = $e;
            if return_value != 0 {
                let error_c = mysqlclient_sys::mysql_stmt_error($stmt);
                let error_msg = String::from_utf8_lossy(CStr::from_ptr(error_c).to_bytes()).into_owned();
                return Err(DbError(format!("MySQL database error: {}", error_msg)));
            }
        }
    }};
}
pub struct MySQLAdaptor{
    handler: *mut mysqlclient_sys::MYSQL
}

impl MySQLAdaptor {
    pub fn connect(host: &str, user: &str, passwd: &str, db: &str) -> Result<Self, DbError> {
        let c_host = CString::new(host).unwrap();
        let c_user = CString::new(user).unwrap();
        let c_passwd = CString::new(passwd).unwrap();
        let c_db = CString::new(db).unwrap();
        let adaptor = MySQLAdaptor {
            handler: unsafe {
                mysqlclient_sys::mysql_init(ptr::null_mut())
            }
        };
        let connect_result = unsafe {
            mysqlclient_sys::mysql_real_connect(
                adaptor.handler,
                c_host.as_ptr(), 
                c_user.as_ptr(), 
                c_passwd.as_ptr(), 
                c_db.as_ptr(), 0, 
                ptr::null(), 
                0)
        };
        return if connect_result.is_null() {
            let error_message: String = unsafe {
                let error_c = mysqlclient_sys::mysql_error(adaptor.handler);
                String::from_utf8_lossy(CStr::from_ptr(error_c).to_bytes()).into_owned()
            };
            Err(DbError(format!("MySQL connection failure: {}", error_message)))
        } else {
            Ok(adaptor)
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
                DbDataType::Int => "BIGINT NOT NULL",
                DbDataType::NullableInt => "BIGINT",
                DbDataType::Text => "TEXT NOT NULL",
                DbDataType::NullableText => "TEXT",
                DbDataType::Float => "DOUBLE",
                DbDataType::RowID => "BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY"
            }
        }
        s = s + ");";
        s
    }

}
impl Drop for MySQLAdaptor {
    fn drop(&mut self) {
        unsafe {
            mysqlclient_sys::mysql_close(self.handler);
        }
    }
}
impl DbAdaptor for MySQLAdaptor {
    fn create_table_for_schema<T: yoshino_core::types::Schema>(&mut self) -> Result<(), yoshino_core::db::DbError> {
        let create_table_stmt = MySQLAdaptor::get_create_table_stmt_code(&T::get_schema_name(), &T::get_fields());
        let stmt_cstring = CString::new(create_table_stmt.as_str()).unwrap();
        unsafe {
            let stmt = mysqlclient_sys::mysql_stmt_init(self.handler);
            if stmt.is_null() {
                return Err(DbError(format!("Mysql database error: out of memory.")));
            }
            db_stmt_try!(
                stmt,
                mysqlclient_sys::mysql_stmt_prepare(
                    stmt, 
                    stmt_cstring.as_ptr(),
                    create_table_stmt.len() as c_ulong
                )
            );
            db_stmt_try!(stmt, mysqlclient_sys::mysql_stmt_execute(stmt));
            mysqlclient_sys::mysql_stmt_close(stmt);
        }
        Ok(())
    } 

    fn insert_record<T: yoshino_core::types::Schema>(&mut self, record: T) -> Result<(), yoshino_core::db::DbError> {
        todo!()
    }

    fn query_all<T: yoshino_core::types::Schema>(&mut self) -> Result<yoshino_core::db::DbQueryResult<T>, yoshino_core::db::DbError> {
        todo!()
    }

    fn query_with_cond<T: yoshino_core::types::Schema>(&mut self, cond: yoshino_core::Cond) -> Result<yoshino_core::db::DbQueryResult<T>, yoshino_core::db::DbError> {
        todo!()
    }

    fn delete_with_cond<T: yoshino_core::types::Schema>(&mut self, cond: yoshino_core::Cond) -> Result<(), yoshino_core::db::DbError> {
        todo!()
    }

    fn update_with_cond<T: yoshino_core::types::Schema>(&mut self, cond:yoshino_core::Cond, record: T) -> Result<(), yoshino_core::db::DbError> {
        todo!()
    }
}