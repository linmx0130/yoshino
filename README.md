Yoshino
===
Simple structural data modeling for Rust.

Yoshino automatically manages how the structural data should be persisted in 
database, so you only need to focus on your business logic.

## Code Structure
* `prelude` - Prelude crate that contains core and derive exports
* `core` - Core data types/abstractions
* `derive` - Marcos for deriving code to implement schema trait
* `sqlite` - SQLite database adaptor
* `user` - User identity type

## Usage
Simply to derive Schema from yoshino:
```rust
use yoshino_prelude::*;

#[derive(Schema)]
struct Record {
  pub id: RowID,
  pub title: String,
  pub content: Option<String>,
  pub reader: i64
}
```

Then you can use a Yoshino database adapter to persist the data of this struct:
```rust
let record = Record::new(...);                 // create a new record
let mut db = 
  SQLiteAdaptor::open("example_db_file")
  .unwrap();                                   // open a SQLite db 
db.insert_record(record).unwrap();             // store the record
```

The data can be retrieved with:
```rust
for record in adaptor.query_all::<Record>().unwrap() {
    // use the data in record
}
```

For more usages, please refer to this document and the examples.

## Copyright and License
Copyright 2022-present Mengxiao Lin <<linmx0130@gmail.com>>.

This project is licensed under [MIT License](LICENSE).