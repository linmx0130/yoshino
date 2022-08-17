use yoshino_sqlite::{SQLiteAdaptor};
use yoshino_core::db::{DbAdaptor};
use yoshino_user::{User};

fn main() {
    let mut adaptor = SQLiteAdaptor::open("db1");
    adaptor.create_table_for_schema::<User>();
}
