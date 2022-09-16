use yoshino_mysql::MySQLAdaptor;
fn main() {
    MySQLAdaptor::connect("localhost", "mysql_user", "mysql_passwd", "test_db").unwrap();
}
