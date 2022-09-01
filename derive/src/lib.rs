//! Yoshino derive macro for Schema trait.

extern crate proc_macro;

use proc_macro::token_stream::IntoIter;
use proc_macro::TokenTree;
use proc_macro::TokenStream;
use proc_macro::TokenTree::{Group, Ident, Punct}; 

/// Derive macro for implementing `yoshino_core::Schema` trait.
#[proc_macro_derive(Schema)]
pub fn derive_schema_fn(src: TokenStream) -> TokenStream {
    let mut src_tokens = src.into_iter();
    // get struct name
    let struct_name = get_next_struct_name(&mut src_tokens).unwrap().to_string();
    let mut derived_code = String::new();
    for it in src_tokens {    
        match it {
            Group(g) => {
                if g.delimiter() == proc_macro::Delimiter::Brace {
                    let fields = get_struct_fields_from_stream(g.stream());
                    derived_code = format!("impl yoshino_core::Schema for {struct_name} {{
    fn get_schema_name() -> String {{
        \"y_{}\".to_owned()
    }}
    fn get_fields() -> Vec<(String, yoshino_core::db::DbDataType)> {{
        {}
    }}
    fn get_values(&self) -> Vec<Box<dyn yoshino_core::db::DbData>> {{
        {}
    }}
    fn create_with_values(values: Vec<Box<dyn yoshino_core::db::DbData>>) -> {struct_name} {{
        {}
    }}
}}",
        struct_name.to_lowercase(),
        get_fields_vec_code(&fields),
        get_values_vec_code(&fields),
        get_create_with_values_code(&struct_name, &fields));
                } else {
                    panic!("Only StructStruct can be derived as schemas.")
                }
            }
            _ => {
                //ignore
            }
        }
    }
    derived_code.parse().unwrap()
}

fn get_next_struct_name(src_iter: &mut IntoIter) -> Option<TokenTree> {
    loop {
        let token = src_iter.next();
        match token {
            None => {break}
            Some(Ident(ident)) => {
                if ident.to_string() == "struct" {
                    return src_iter.next();
                }
            }
            _ => {}
        }
    }
    None
}

fn get_struct_fields_from_stream(src: TokenStream) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut state = 0;
    let mut current_field_name = String::new();
    let mut current_field_type = String::new();

    for it in src.into_iter() {
        match state {
            0 => {
                // wait for field name
                match &it {
                    Ident(ident) => {
                        if ident.to_string() != "pub"{
                            current_field_name = ident.to_string();
                            state = 1;
                        }
                    }
                    _ => {
                        //ignore
                    }
                }
            }
            1 => {
                // wait for punct ':'
                match &it {
                    Punct(punct) => {
                        if punct.as_char() == ':' {
                            state = 2;
                        }
                    } 
                    _ => {}
                }
            }
            2 => {
                // wait for field type
                match &it {
                    Ident(ident) => {
                        current_field_type = current_field_type + &ident.to_string();
                    }
                    Punct(punct) => {
                        match punct.as_char(){
                            ',' => {
                                fields.push((current_field_name.to_owned(), current_field_type.to_owned()));
                                current_field_name = String::new();
                                current_field_type = String::new();
                                state = 0;
                            }
                            '<' => {
                                current_field_type = current_field_type + "::<";
                            }
                            c => {
                                current_field_type = current_field_type + &c.to_string();
                            }
                        }
                    }
                    _=> {}
                }
            }
            _ => {}
        }
    }
    
    // end with state 2 -> there is a last field without ',' in the end
    if state == 2 {
        fields.push((current_field_name.to_owned(), current_field_type.to_owned()));
    }
    fields
}

fn get_fields_vec_code(fields: &Vec<(String, String)>) -> String {
    let mut s = "vec![".to_owned();
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
        }
        let (field_name, field_type) = fields.get(i).unwrap();
        s = s + format!("(\"{}\".to_string(), {}::db_field_type())", field_name, field_type).as_ref(); 
    }
    s = s + "]";
    return s
}

fn get_values_vec_code(fields: &Vec<(String, String)>) -> String {
    let mut s = "vec![".to_string();
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
        }
        let (field_name, _) = fields.get(i).unwrap();
        s = s + format!("Box::new(self.{}.to_db_data())", field_name).as_ref();
    }
    s = s + "]";
    s
}

fn get_create_with_values_code(struct_name: &str, fields: &Vec<(String, String)>) -> String {
    let mut s = struct_name.to_owned() + "{";
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
        }
        let (field_name, field_type) = fields.get(i).unwrap();
        s = s + format!("{}: {}::from_db_data(&values[{}])", field_name, field_type, i).as_ref();
    }
    s = s + "}";
    s
}