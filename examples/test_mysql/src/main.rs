use yoshino_mysql::MySQLAdaptor;
use yoshino_prelude::*;

#[derive(Schema)]
struct Counter {
    pub pid: RowID,
    pub name: String,
    pub stock: Option<i64>,
}

fn main() {
    let mut adaptor = MySQLAdaptor::connect("localhost", "mysql_user", "mysql_passwd", "test_db").unwrap();
    adaptor.create_table_for_schema::<Counter>().unwrap();
    let record = Counter {
        pid: RowID::NEW,
        name: "adam".to_string(),
        stock: Some(4)
    };
    adaptor.insert_record(record).unwrap();
}
