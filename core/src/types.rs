//! Data type declarations for Yoshino.
//!
//! Only the types that implement a trait of "fields" can be used in Yoshino
//! schema struct. Now available field traits are:
//!
//! * `TextField` - nonnull text field.
//! * `NullableTextField` - nullable text field.
//! * `IntegerField` - nonnull 64-bit integer field.
//! * `NullableIntegerField` - nullable 64-bit integer field.
//! * `FloatField` - nonnull 64-bit floating point field.
//!
//! If you want to use a custom type in schema struct, you need to implement
//! one field trait for this custom type.
//!
//! All field traits declare the method to generate DbData object that can be
//! accepted by the Yoshino database interfaces.

use crate::db::{DbData, DbDataType};

/// It can be serialized as a String in Yoshino.
pub trait TextField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the string to be used by the Yoshino.
    fn to_db_data(&self) -> String;
    /// The `DbDataType` of this field. For all `TextField` objects, it's `DbDataType::Text`.
    fn db_field_type() -> DbDataType {
        DbDataType::Text
    }
}

/// It can be serialized as a nullable String in Yoshino.
pub trait NullableTextField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the string to be used by the Yoshino.
    fn to_db_data(&self) -> Option<String>;
    /// The `DbDataType` of this field. For all `NullableTextField` objects, it's `DbDataType::NullableText`.
    fn db_field_type() -> DbDataType {
        DbDataType::NullableText
    }
}

/// It can be serailized as a 64-bit integer in Yoshino.
pub trait IntegerField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the i64 to be used by the Yoshino.
    fn to_db_data(&self) -> i64;
    /// The `DbDataType` of this field. For all `IntegerField` objects, it's `DbDataType::Int`.
    fn db_field_type() -> DbDataType {
        DbDataType::Int
    }
}

/// It can be serailized as a nullable 64-bit integer in Yoshino.
pub trait NullableIntegerField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the i64 to be used by the Yoshino.
    fn to_db_data(&self) -> Option<i64>;
    /// The `DbDataType` of this field. For all `IntegerField` objects, it's `DbDataType::NullableInt`.
    fn db_field_type() -> DbDataType {
        DbDataType::NullableInt
    }
}

/// It can be serailized as 64-bit floating point numeric number in Yoshino.
pub trait FloatField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the f64 to be used by the Yoshino.
    fn to_db_data(&self) -> f64;
    /// The `DbDataType` of this field. For all `FloatField` objects, it's `DbDataType::Float`.
    fn db_field_type() -> DbDataType {
        DbDataType::Float
    }
}

/// A binary large object field for storing raw data.
pub trait BinaryField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the Vec<u8> to be used by the Yoshino.
    fn to_db_data(&self)-> Vec<u8>;
    /// The `DbDataType` of this field. For all `BinaryField` objects, it's `DbDataType::Binary`.
    fn db_field_type() -> DbDataType {
        DbDataType::Binary
    }
}

/// A nullable binary large object field for storing raw data.
pub trait NullableBinaryField: Sized {
    /// Create an instance from a boxed DbData trait object.
    fn from_db_data(data: &Box<dyn DbData>) -> Self;
    /// Create the Vec<u8> to be used by the Yoshino.
    fn to_db_data(&self)-> Option<Vec<u8>>;
    /// The `DbDataType` of this field. For all `NullableBinaryField` objects, it's `DbDataType::NullableBinary`.
    fn db_field_type() -> DbDataType {
        DbDataType::NullableBinary
    }
}

impl TextField for String {
    fn from_db_data(data: &Box<dyn DbData>) -> String {
        <String as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self) -> String {
        self.to_owned()
    }
}

impl NullableTextField for Option<String> {
    fn from_db_data(data: &Box<dyn DbData>) -> Option<String> {
        <Option<String> as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self) -> Option<String> {
        match self {
            None => None,
            Some(x) => Some(x.to_owned()),
        }
    }
}

impl IntegerField for i64 {
    fn from_db_data(data: &Box<dyn DbData>) -> Self {
        <i64 as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self) -> i64 {
        *self
    }
}

impl NullableIntegerField for Option<i64> {
    fn from_db_data(data: &Box<dyn DbData>) -> Self {
        <Option<i64> as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self) -> Option<i64> {
        *self
    }
}

impl FloatField for f64 {
    fn from_db_data(data: &Box<dyn DbData>) -> Self {
        <f64 as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self) -> f64 {
        *self
    }
}

impl BinaryField for Vec<u8> {
    fn from_db_data(data: &Box<dyn DbData>) -> Self {
        <Vec<u8> as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self)-> Vec<u8> {
        self.clone()
    }
}

impl NullableBinaryField for Option<Vec<u8>> {
    fn from_db_data(data: &Box<dyn DbData>) -> Self {
        <Option<Vec<u8>> as DbData>::from_boxed_db_data(data)
    }
    fn to_db_data(&self)-> Option<Vec<u8>> {
        self.clone()
    }
}

/// Auto increment row ID field. It will be represented as an integer primary key.
///
/// A schema can has at most one RowID field.
#[derive(Clone, Copy, Debug)]
pub enum RowID {
    /// A new created object so it doesn't have a row id yet.
    NEW,
    /// The row id retrieved from the database.
    ID(i64),
}

impl RowID {
    pub fn from_db_data(data: &Box<dyn DbData>) -> RowID {
        <RowID as DbData>::from_boxed_db_data(data)
    }
    pub fn to_db_data(&self) -> RowID {
        self.clone()
    }
    pub fn db_field_type() -> DbDataType {
        DbDataType::RowID
    }
}

/// Make the type a data schema in the relational database.
///
/// In most cases, you should only use the derive macro to implement this trait.
pub trait Schema: 'static {
    /// the schema name in database
    fn get_schema_name() -> String;
    /// the list of field names and types of this data struct
    fn get_fields() -> Vec<(String, DbDataType)>;
    /// the values of all fields in boxed DbData objects.
    fn get_values(&self) -> Vec<Box<dyn DbData>>;
    /// to create the struct with valeus of all fields in boxed DbData objects
    fn create_with_values(values: Vec<Box<dyn DbData>>) -> Self;

    /// get the name and value of the RowID field.
    /// Return `None` if there is no such field. Panic if there are more than one RowID field.
    fn get_row_id_field(&self) -> Option<(String, RowID)> {
        let fields = Self::get_fields();
        let values = Self::get_values(&self);
        let mut answer = None;
        for i in 0..fields.len() {
            let (field_name, field_type) = &fields.get(i).unwrap();
            if let DbDataType::RowID = field_type {
                if answer.is_none() {
                    let field_value = values.get(i).unwrap();
                    let answer_field_name = field_name.to_owned();
                    let answer_field_value = RowID::from_db_data(field_value);
                    answer = Some((answer_field_name, answer_field_value));
                } else {
                    panic!(
                        "Multiple Row ID fields found in {}",
                        Self::get_schema_name()
                    );
                }
            }
        }
        return answer;
    }
}
