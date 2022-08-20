use yoshino_sqlite::{SQLiteAdaptor};
use yoshino_core::db::{DbAdaptor};
use yoshino_user::{User};
use bytes::Bytes;


fn main() {
    let mut adaptor = SQLiteAdaptor::open("db1");
    adaptor.create_table_for_schema::<User>().unwrap();
    let new_user = User::new(
        "admin".to_string(), 
        "this_is_admin".to_string(), 
        yoshino_user::UserCredentialHashType::Sha256WithSalt(Bytes::from("salt")));
    adaptor.insert_record(new_user).unwrap();
    let query_result = adaptor.query_all::<User>().unwrap();
    for user in query_result {
        println!("user: {:?}", user);
    }
}