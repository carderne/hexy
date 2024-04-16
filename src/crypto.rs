use fernet::{Fernet, MultiFernet};
use std::env;

use crate::error;

pub struct Crypto {
    mf: MultiFernet,
}

impl Default for Crypto {
    fn default() -> Self {
        let keys = env::var("FERNET_KEYS").unwrap();
        Self::new(&keys)
    }
}

impl Crypto {
    pub fn new(keys: &str) -> Self {
        let keys: Vec<&str> = keys.split(',').collect();
        let fernets: Vec<Fernet> = keys.iter().map(|k| Fernet::new(k).unwrap()).collect();
        Self {
            mf: MultiFernet::new(fernets),
        }
    }

    pub fn encrypt(&self, data: &str) -> String {
        let data = data.as_bytes().to_vec();
        self.mf.encrypt(&data)
    }

    pub fn decrypt(&self, data: &str) -> Result<String, error::Error> {
        let out = self.mf.decrypt(data)?;
        let out = String::from_utf8(out)?;
        Ok(out)
    }

    /// Will simply return the data if decryption fails
    /// so that things continue to work during migration to encryption
    pub fn decrypt_fallback(&self, data: &str) -> String {
        let decrypted = self.decrypt(data);
        match decrypted {
            Ok(out) => out,
            Err(_) => data.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let k1 = Fernet::generate_key();
        let k2 = Fernet::generate_key();
        let keys = format!("{},{}", k1, k2);
        let c = Crypto::new(&keys);
        let want = "longrandombunchoftokenstuff12345ABCDEFG";
        let encrypted = c.encrypt(want);
        let got = c.decrypt(&encrypted).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn test_decrypt_fallback() {
        let k1 = Fernet::generate_key();
        let k2 = Fernet::generate_key();
        let keys = format!("{},{}", k1, k2);
        let c = Crypto::new(&keys);
        let want = "longrandombunchoftokenstuff12345ABCDEFG";
        let got = c.decrypt_fallback(want);
        assert_eq!(want, got);
    }
}
