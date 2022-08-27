use yoshino_prelude::*;
use yoshino_sqlite::{SQLiteAdaptor};
use yoshino_user::{User, UserCredential};
use bytes::Bytes;

#[derive(Schema)]
struct Counter {
    pub name: String,
    pub stock: Option<i64>,
}

fn main() {
    let mut adaptor = SQLiteAdaptor::open("db1").unwrap();
    adaptor.create_table_for_schema::<User>().unwrap();
    let new_user = User::new(
        "admin".to_string(), 
        "this_is_admin".to_string(), 
        yoshino_user::UserCredentialHashType::Sha256WithSalt(Bytes::from("salt")));
    adaptor.insert_record(new_user).unwrap();
    let query_result = adaptor.query_all::<User>().unwrap();
    for user in query_result {
        println!("user: {:?}", user);
        let mut new_user = user.clone();
        new_user.login_credential = UserCredential::new(
            Bytes::from("new_password"), 
            yoshino_user::UserCredentialHashType::Sha256WithSalt(Bytes::from("salt2"))
        );
        adaptor.update_with_cond(Cond::text_equal_to("user_name", "admin"), new_user).unwrap();
    }
    println!(">> New users");
    for user in adaptor.query_all::<User>().unwrap() {
        println!("user: {:?}", user);
    }

    adaptor.create_table_for_schema::<Counter>().unwrap();
    let p1 = Counter {name:"milk".to_string(), stock: Some(20)};
    let p2 = Counter{name:"cream".to_string(), stock: None};
    let p3 = Counter{name:"apple".to_string(), stock: Some(30)};
    adaptor.insert_record(p1).unwrap();
    adaptor.insert_record(p2).unwrap();
    adaptor.insert_record(p3).unwrap();
    let cond = Cond::or(
        Cond::is_null("stock"),
        Cond::integer_equal_to("stock", 20)
    );
    let query_result = adaptor.query_with_cond::<Counter>(cond).unwrap();
    for p in query_result {
        println!("Product: {}, stock = {:?}", p.name, p.stock);
    }
    adaptor.delete_with_cond::<User>(Cond::text_equal_to("user_name", "admin")).unwrap();

}
