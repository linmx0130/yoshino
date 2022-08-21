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

impl Cond {
    pub fn or(left: Cond, right: Cond) -> Cond{
        Cond::Or { left: Box::new(left), right: Box::new(right) }
    }

    pub fn and(left: Cond, right: Cond) -> Cond{
        Cond::And { left: Box::new(left), right: Box::new(right) }
    }

    pub fn is_null(field_name: &str) -> Cond {
        Cond::IsNull { field_name: field_name.to_string() }
    }
    
    pub fn is_not_null(field_name: &str) -> Cond {
        Cond::IsNotNull { field_name: field_name.to_string() }
    }

    pub fn integer_equal_to(field_name: &str, value: i64) -> Cond {
        Cond::IntegerEqualTo { field_name: field_name.to_string(), value}
    }
    
    pub fn integer_not_equal_to(field_name: &str, value: i64) -> Cond {
        Cond::IntegerNotEqualTo { field_name: field_name.to_string(), value}
    }
    
    pub fn text_equal_to(field_name: &str, value: &str) -> Cond {
        Cond::TextEqualTo { field_name: field_name.to_string(), value: value.to_string() }
    }
}