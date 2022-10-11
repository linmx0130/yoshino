//! This is the main API export crate for Yoshino.
//! ## Usage
//! Simply to derive Schema from yoshino:
//! ```
//! use yoshino_prelude::*;
//! 
//! #[derive(Schema)]
//! struct Record {
//!   pub id: RowID,
//!   pub title: String,
//!   pub content: Option<String>,
//!   pub reader: i64
//! }
//! ```
//! 
//! Then you can use a Yoshino database adapter to persist the data of this struct:
//! ```text
//! let record = Record::new(...);                 // create a new record
//! let mut db = 
//!   SQLiteAdaptor::open("example_db_file")
//!   .unwrap();                                   // open a SQLite db 
//! db.insert_record(record).unwrap();             // store the record
//! ```
//! 
//! The data can be retrieved with:
//! ```text
//! for record in adaptor.query_all::<Record>().unwrap() {
//!     // use the data in record
//! }
//! ```
//! 
//! For more usages, please refer to this document and the examples.

pub use yoshino_core;
pub use yoshino_core::Cond;
pub use yoshino_core::Schema;
pub use yoshino_core::{RowID, NullableIntegerField, NullableTextField, IntegerField, TextField, FloatField, BinaryField, NullableBinaryField};
pub use yoshino_core::db::{DbAdaptor, DbData, DbError, DbQueryResult, DbDataType};
pub use yoshino_derive::Schema;