use yoshino_mysql::MySQLAdaptor;
use yoshino_prelude::*;

#[derive(Schema, Debug, Clone)]
struct Counter {
    pub pid: RowID,
    pub name: Option<String>,
    pub stock: Option<i64>,
    pub bb: Option<Vec<u8>>
}

fn main() {
    let mut adaptor = MySQLAdaptor::connect("localhost", "mysql_user", "mysql_passwd", "test_db").unwrap();
    adaptor.create_table_for_schema::<Counter>().unwrap();
    let record = Counter {
        pid: RowID::NEW,
        name: Some("world".to_string()),
        stock: Some(7),
        bb: None
    };
    adaptor.insert_record(record).unwrap();
    
    for item in adaptor.query_all::<Counter>().unwrap() {
        println!("{:?}", item)
    }
}
