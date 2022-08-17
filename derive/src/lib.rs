extern crate proc_macro;
use proc_macro::token_stream::IntoIter;
use proc_macro::TokenTree;
use proc_macro::TokenStream;
use proc_macro::TokenTree::{Group, Ident, Punct}; 

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
    fn create_table_stmt() -> String {{
        {}
    }}
    fn insert_value_stmt() -> String {{
        {}.to_owned()
    }}
    fn insert_value_params(&self) -> Vec<Box<dyn yoshino_core::db::DbData>> {{
        {}
    }}
}}", get_create_table_stmt_code(&struct_name, &fields),
     get_insert_value_stmt_code(&struct_name, &fields),
     get_insert_value_params_code(&fields));
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
                        current_field_name = ident.to_string();
                        state = 1;
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
    fields
}

fn get_create_table_stmt_code(struct_name: &str, fields: &Vec<(String, String)>) -> String {
    let mut s = format!("format!(\"CREATE TABLE IF NOT EXISTS y_{} (", struct_name);
    let mut type_str_buf = String::new();
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
            type_str_buf = type_str_buf + ", ";
        }
        let (field_name, field_type) = fields.get(i).unwrap();
        s = s + &field_name;
        type_str_buf = type_str_buf + format!("{}::db_field_type()", field_type).as_ref();
        s = s + " {}";
    }
    s = s + ");\", " + type_str_buf.as_ref() + ")";
    s
}

fn get_insert_value_stmt_code(struct_name:&str, fields: &Vec<(String, String)>) -> String {
    let mut s = format!("\"INSERT INTO y_{struct_name} (");
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
        }
        let (field_name, _) = fields.get(i).unwrap();
        s = s + &field_name;
    }
    s = s + ") VALUES(";
    for i in 0..fields.len() {
        if i != 0 {
            s = s + ", ";
        }
        s = s + format!("?{}", i+1).as_ref();
    }
    s = s + ");\"";
    s
}
fn get_insert_value_params_code(fields: &Vec<(String, String)>) -> String {
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