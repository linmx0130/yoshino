use std::marker::PhantomData;
use std::os::raw::{c_ulong};
use yoshino_core::db::{DbAdaptor, DbError, DbDataType, DbData};
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

/// The container to hold a MySQL bind list with the reference to original data.
struct MySQLBindList<'a> {
    // keep this as a phanton type for track data lifetime
    _data: &'a Vec<Box<dyn DbData>>,
    // length list will be deallocated after the bind is finised
    length_list: Vec<u64>,
    binds: Vec<mysqlclient_sys::MYSQL_BIND>,
    is_null_placeholder: i8
}

impl<'a> MySQLBindList<'a> {
    fn from_boxed_db_data_list(data: &'a Vec<Box<dyn DbData>>) -> MySQLBindList<'a> {
        let length_list: Vec<u64> = data.iter().map(|x| x.db_data_len() as u64).collect();
        let mut binds = vec![];
        let mut return_value = MySQLBindList { _data: data, length_list: length_list, binds: vec![], is_null_placeholder: 1 };
        let length_list_ptr = return_value.length_list.as_mut_ptr();
        for i in 0..data.len() {
            unsafe {
                let mut bind: mysqlclient_sys::MYSQL_BIND = std::mem::zeroed();
                let data_item = &data[i];
                bind.buffer = data_item.db_data_ptr() as *mut std::ffi::c_void;
                bind.length = length_list_ptr.offset(i as isize);
                bind.buffer_type = match data_item.db_data_type() {
                    yoshino_core::db::DbDataType::Int | yoshino_core::db::DbDataType::NullableInt | yoshino_core::db::DbDataType::RowID => 
                        mysqlclient_sys::enum_field_types::MYSQL_TYPE_LONGLONG,
                    yoshino_core::db::DbDataType::Text | yoshino_core::db::DbDataType::NullableText => 
                        mysqlclient_sys::enum_field_types::MYSQL_TYPE_STRING,
                    yoshino_core::db::DbDataType::Float =>
                        mysqlclient_sys::enum_field_types::MYSQL_TYPE_DOUBLE,
                };
                bind.is_null = if data_item.db_data_ptr().is_null() {
                    &mut return_value.is_null_placeholder
                } else {
                    0 as *mut i8
                };
                binds.push(bind);
            }
        } 
        return_value.binds = binds;
        return_value
    }
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

    fn get_insert_value_stmt_code(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut fields_part = String::new();
        let mut fields_value_tokens = String::new();
        for i in 0..fields.len() {
            if i != 0 {
                fields_part = fields_part + ", ";
                fields_value_tokens = fields_value_tokens + ", ";
            }
            let (field_name, _) = fields.get(i).unwrap();
            fields_part = fields_part + field_name;
            fields_value_tokens = fields_value_tokens + "?";
        }
        format!("INSERT INTO {} ({}) VALUES ({});", schema_name, fields_part, fields_value_tokens)
    }

    fn get_query_clause_code(schema_name: &str, fields: &Vec<(String, DbDataType)>) -> String {
        let mut fields_str = String::new();
        for i in 0..fields.len() {
            if i != 0 {
                fields_str = fields_str + ", ";
            }
            let (field_name, _) = fields.get(i).unwrap();
            fields_str = fields_str + field_name;
        }
        format!("SELECT {} FROM {}", fields_str, schema_name)
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
        let insert_value_stmt = MySQLAdaptor::get_insert_value_stmt_code(&T::get_schema_name(), &T::get_fields());
        let stmt_cstring = CString::new(insert_value_stmt.as_str()).unwrap();
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
                    insert_value_stmt.len() as c_ulong
                )
            );
            let values = record.get_values();
            let mut bind_list = MySQLBindList::from_boxed_db_data_list(&values);
            let bind_array = bind_list.binds.as_mut_ptr();
            db_stmt_try!(
                stmt,
                mysqlclient_sys::mysql_stmt_bind_param(stmt, bind_array)
            );
            db_stmt_try!(stmt, mysqlclient_sys::mysql_stmt_execute(stmt));
            mysqlclient_sys::mysql_stmt_close(stmt);
        }
        Ok(())
    }

    fn query_all<T: yoshino_core::types::Schema>(&mut self) -> Result<yoshino_core::db::DbQueryResult<T>, yoshino_core::db::DbError> {
        let query_stmt = format!("{};", MySQLAdaptor::get_query_clause_code(&T::get_schema_name(), &T::get_fields()));
        let stmt_cstring = CString::new(query_stmt.as_str()).unwrap();
        unsafe {
            let stmt = mysqlclient_sys::mysql_stmt_init(self.handler);
            if stmt.is_null() {
                return Err(DbError(format!("Mysql database error: out of memory.")));
            }
            db_stmt_try!(
                stmt,
                mysqlclient_sys::mysql_stmt_prepare(stmt, stmt_cstring.as_ptr(), query_stmt.len() as c_ulong)
            );
            db_stmt_try!(
                stmt,
                mysqlclient_sys::mysql_stmt_execute(stmt)
            );
            let internal_iterator = MySQLResultIterator::new(stmt)?;
            Ok(yoshino_core::db::DbQueryResult {data_iter: Box::new(internal_iterator)})
        }
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

struct MySQLResultIterator<T> where T: yoshino_core::Schema {
    stmt: *mut mysqlclient_sys::st_mysql_stmt,
    fields: Vec<(String, DbDataType)>,
    bind_list: Vec<mysqlclient_sys::MYSQL_BIND>,
    length_list: Vec<u64>,
    is_null_list: Vec<i8>,
    phantom: PhantomData<T>
}

impl<T> MySQLResultIterator<T> where T: yoshino_core::Schema {
    fn new(stmt: *mut mysqlclient_sys::st_mysql_stmt) -> Result<MySQLResultIterator<T>, DbError>{
        let fields = T::get_fields();
        let mut length_list = vec![0u64; fields.len()];
        let mut is_null_list = vec![0;fields.len()];
        let mut bind_list: Vec<mysqlclient_sys::MYSQL_BIND> = fields.iter().map(
            |_| unsafe {
                std::mem::zeroed::<mysqlclient_sys::MYSQL_BIND>()
            }
        ).collect();
        let length_list_ptr = length_list.as_mut_ptr();
        let is_null_list_ptr = is_null_list.as_mut_ptr();
        for i in 0..bind_list.len() {
            unsafe {
                bind_list[i].length = length_list_ptr.offset(i as isize);
                bind_list[i].is_null = is_null_list_ptr.offset(i as isize);
            }
        }
        unsafe{
            db_stmt_try!(
                stmt,
                mysqlclient_sys::mysql_stmt_bind_result(stmt, bind_list.as_mut_ptr())
            );
        }
        Ok(MySQLResultIterator { stmt, fields, bind_list, length_list, is_null_list, phantom: PhantomData })
    }

    fn clear_binds(&mut self) {
        let fields_count = self.length_list.len();
        for i in 0..fields_count {
            self.length_list[i] = 0;
            self.bind_list[i].buffer_type = unsafe {std::mem::zeroed()};
        }
    }
}

impl<T> Iterator for MySQLResultIterator<T> where T:yoshino_core::Schema {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let status = unsafe {
            mysqlclient_sys::mysql_stmt_fetch(self.stmt)
        };
        if status == (mysqlclient_sys::MYSQL_NO_DATA as i32) || status == 1 {
            return None
        }
        let mut values :Vec<Box<dyn DbData>> = vec![];
        for i in 0..self.fields.len() {
            match self.fields[i].1 {
                yoshino_core::db::DbDataType::Int | yoshino_core::db::DbDataType::RowID =>{
                    self.bind_list[i].buffer_length = 1;
                    unsafe {
                        let mut buffer:i64 = std::mem::zeroed();
                        self.bind_list[i].buffer = (&mut buffer) as *mut i64 as *mut std::ffi::c_void;
                        self.bind_list[i].buffer_type = mysqlclient_sys::enum_field_types::MYSQL_TYPE_LONGLONG;
                        mysqlclient_sys::mysql_stmt_fetch_column(
                            self.stmt, 
                            self.bind_list.as_mut_ptr().offset(i as isize),
                            i as u32, 0);
                        values.push(Box::new(buffer));
                    }
                }
                yoshino_core::db::DbDataType::NullableInt =>{
                    if self.is_null_list[i] != 0 {
                        values.push(Box::<Option<i64>>::new(None));
                    } else {
                        self.bind_list[i].buffer_length = 1;
                        unsafe {
                            let mut buffer:i64 = std::mem::zeroed();
                            self.bind_list[i].buffer = (&mut buffer) as *mut i64 as *mut std::ffi::c_void;
                            self.bind_list[i].buffer_type = mysqlclient_sys::enum_field_types::MYSQL_TYPE_LONGLONG;
                            mysqlclient_sys::mysql_stmt_fetch_column(
                                self.stmt, 
                                self.bind_list.as_mut_ptr().offset(i as isize),
                                i as u32, 0);
                            values.push(Box::<Option<i64>>::new(Some(buffer)));
                        }
                    }
                }
                yoshino_core::db::DbDataType::Text => {
                    if self.length_list[i] == 0 {
                        values.push(Box::new("".to_string()));
                    } else {
                        let mut buffer: Vec<u8> = vec![0u8 ;self.length_list[i] as usize];
                        self.bind_list[i].buffer = buffer.as_mut_ptr() as *mut std::ffi::c_void;
                        self.bind_list[i].buffer_type = mysqlclient_sys::enum_field_types::MYSQL_TYPE_STRING;
                        self.bind_list[i].buffer_length = self.length_list[i];
                        unsafe {
                            mysqlclient_sys::mysql_stmt_fetch_column(
                                self.stmt, 
                                self.bind_list.as_mut_ptr().offset(i as isize),
                                i as u32, 0);
                        }
                        values.push(Box::new(String::from_utf8(buffer).unwrap()));
                    }
                }
                yoshino_core::db::DbDataType::NullableText => {
                    if self.is_null_list[i] != 0 {
                        values.push(Box::<Option<String>>::new(None));
                    } else {
                        let mut buffer: Vec<u8> = vec![0u8 ;self.length_list[i] as usize];
                        self.bind_list[i].buffer = buffer.as_mut_ptr() as *mut std::ffi::c_void;
                        self.bind_list[i].buffer_type = mysqlclient_sys::enum_field_types::MYSQL_TYPE_STRING;
                        self.bind_list[i].buffer_length = self.length_list[i];
                        unsafe {
                            mysqlclient_sys::mysql_stmt_fetch_column(
                                self.stmt, 
                                self.bind_list.as_mut_ptr().offset(i as isize),
                                i as u32, 0);
                        }
                        values.push(Box::new(Some(String::from_utf8(buffer).unwrap())));
                    }
                }
                yoshino_core::db::DbDataType::Float => {
                    let mut buffer = 0f64;
                    self.bind_list[i].buffer = (&mut buffer) as *mut f64 as *mut std::ffi::c_void;
                    self.bind_list[i].buffer_type = mysqlclient_sys::enum_field_types::MYSQL_TYPE_DOUBLE;
                    self.bind_list[i].buffer_length = self.length_list[i];
                    unsafe {
                        mysqlclient_sys::mysql_stmt_fetch_column(
                            self.stmt, 
                            self.bind_list.as_mut_ptr().offset(i as isize),
                            i as u32, 0);
                    }
                    values.push(Box::new(buffer));
                }
            }    
        }
        self.clear_binds();
        Some(T::create_with_values(values))
    }
}

impl<T> Drop for MySQLResultIterator<T> where T:yoshino_core::Schema{
    fn drop(&mut self) {
        unsafe{
            mysqlclient_sys::mysql_stmt_close(self.stmt);
        }
    }
}