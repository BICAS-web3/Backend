use hmac::{Hmac, Mac};
pub use jwt::error::Error as JwtError;
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::str;

use serde::{Deserialize, Serialize};

type HS256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    /// issuer
    pub iss: Option<String>,
    /// user login
    pub sub: String,
    /// expiration time
    pub exp: u64,
    /// issue time
    pub iat: u64,
    /// audience
    pub aud: String,
}

pub fn generate_token(payload: &Payload, secret: &str) -> String {
    let key: HS256 = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    payload.sign_with_key(&key).unwrap()
}

pub fn verify_token(token: &str, secret: &str) -> Result<Payload, JwtError> {
    let key: HS256 = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    token.verify_with_key(&key)
}
