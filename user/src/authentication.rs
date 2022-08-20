/// Internal code for authenticate a user.

use bytes::{Bytes, BytesMut, BufMut, Buf};
use sha2::{Sha256, Digest};
use yoshino_core::{TextField, db::DbData};

/// To indicate how the useer credential is hashed
#[derive(Clone, Debug)]
pub enum UserCredentialHashType {
    /// SHA256 hash with a salt
    Sha256WithSalt(Bytes)
} 

/// User credential type 
#[derive(Clone, Debug)]
pub struct UserCredential {
    data: Bytes,
    hash_type: UserCredentialHashType
}

impl UserCredential {
    /// Validate whether the plain credential matches this user.
    pub fn validate_credential(&self, credential_plain: Bytes) -> bool {
        match &self.hash_type {
            UserCredentialHashType::Sha256WithSalt(salt) => {
                let mut hasher = Sha256::new();
                hasher.update(credential_plain.as_ref());
                hasher.update(salt.as_ref());
                let result = hasher.finalize();
                let result_bytes = Bytes::from(result.to_vec());
                result_bytes.eq(&self.data)
            }
        }
    }

    /// Create a user credential with the plain text and the hash type.
    pub fn new(credential_plain: Bytes, hash_type: UserCredentialHashType)-> UserCredential {
        let data = match &hash_type {
            UserCredentialHashType::Sha256WithSalt(salt) => {
                let mut hasher = Sha256::new();
                hasher.update(credential_plain.as_ref());
                hasher.update(salt.as_ref());
                let result = hasher.finalize();
                Bytes::from(result.to_vec())
            }            
        };
        UserCredential { data, hash_type}
    }
}

impl TextField for UserCredential {
    fn to_db_data(&self) -> String {
        let mut buf = BytesMut::new();
        match &self.hash_type {
            UserCredentialHashType::Sha256WithSalt(salt) => {
                buf.put_i32(0x35A256);
                buf.put_u32(salt.len() as u32);
                buf.put_slice(salt);
                buf.put_u32(self.data.len() as u32);
                buf.put_slice(&self.data);
            }
        };
        base64::encode(buf)
    }
    fn from_db_data(data: &Box<dyn DbData>) -> UserCredential {
        let data_str = <String as DbData>::from_boxed_db_data(data);
        let mut buf = Bytes::from(base64::decode(data_str).unwrap());
        let magic_number = buf.get_i32();
        match magic_number {
            0x35A256 => {
                let salt_len = buf.get_u32() as usize;
                let mut salt = BytesMut::with_capacity(salt_len);
                for _i in 0..salt_len {
                    salt.put_u8(buf.get_u8())
                }
                let data_len = buf.get_u32() as usize;
                let mut data = BytesMut::with_capacity(data_len);
                for _i in 0..data_len {
                    data.put_u8(buf.get_u8())
                }
                UserCredential {
                    data: Bytes::from(data),
                    hash_type: UserCredentialHashType::Sha256WithSalt(Bytes::from(salt))
                }
            }
            _ => {
                panic!("Unsupported user credential type");
            }
        }
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use bytes::Bytes;
    #[test]
    fn test_sha256_user_credential_validation() {
        let plain_text = "this_is_the_pain_text";
        let ground_truth = UserCredential::new(
            Bytes::from(plain_text), 
            UserCredentialHashType::Sha256WithSalt(Bytes::from("salt"))
        );
        
        assert!(ground_truth.validate_credential(Bytes::from(plain_text)));
        assert!(!ground_truth.validate_credential(Bytes::from("this is not the plain text")))
    }

    #[test]
    fn test_sha256_user_base64_serialization() {
        let plain_text = "this_is_the_pain_text";
        let ground_truth = UserCredential::new(
            Bytes::from(plain_text), 
            UserCredentialHashType::Sha256WithSalt(Bytes::from("salt"))
        );
        
        let data = ground_truth.to_db_data();
        let new_user = UserCredential::from_db_data(&data).unwrap();
        assert!(new_user.validate_credential(Bytes::from(plain_text)));
    }
}