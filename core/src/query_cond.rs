/// Query conditions.
pub enum Cond {
    IsNull{field_name: String},
    IsNotNull {field_name: String},
    TextEqualTo{field_name: String, value: String},
    IntegerEqualTo{field_name: String, value: i64},
    IntegerNotEqualTo{field_name: String, value: i64},
    And {left: Box<Cond>, right: Box<Cond>},
    Or {left: Box<Cond>, right: Box<Cond>}
}