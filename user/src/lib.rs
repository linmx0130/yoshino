/**
 * User related data model.
 */

mod authentication;
pub use authentication::UserCredentialHashType as UserCredentialHashType;
use bytes::Bytes;
use yoshino_core::{TextField, RowID};
use yoshino_derive::Schema;

/// The user representation for login purpose
#[derive(Schema, Debug)]
pub struct User {
    id: RowID,
    pub user_name: String,
    login_credential: authentication::UserCredential,
}

impl User {
    pub fn new(user_name: String, password: String, hash_type: UserCredentialHashType) -> User {
        let login_credential = authentication::UserCredential::new(Bytes::from(password.to_owned()), hash_type);
        User {
            id: RowID::NEW,
            user_name,
            login_credential
        }
    }
}