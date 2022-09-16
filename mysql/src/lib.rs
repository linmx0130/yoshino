use yoshino_core::db::{DbAdaptor, DbError};
use mysqlclient_sys::{self, mysql_error};
use std::ffi::{CString, CStr};
use std::ptr;

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
                let error_c = mysql_error(adaptor.handler);
                String::from_utf8_lossy(CStr::from_ptr(error_c).to_bytes()).into_owned()
            };
            Err(DbError(format!("MySQL connection failure: {}", error_message)))
        } else {
            Ok(adaptor)
        }
    }
}

impl DbAdaptor for MySQLAdaptor {
    fn create_table_for_schema<T: yoshino_core::types::Schema>(&mut self) -> Result<(), yoshino_core::db::DbError> {
        todo!()
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