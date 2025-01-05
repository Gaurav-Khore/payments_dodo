use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"payments_dodo";

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}
pub fn encode_jwt(uid: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    println!("Helllo from the create_jwt");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .expect("Valid Timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid,
        exp: expiration as usize,
    };

    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
}

pub fn decode_jwt(token: String) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    match decode::<Claims>(&token, &DecodingKey::from_secret(JWT_SECRET), &validation) {
        Ok(v) => Ok(v.claims),
        Err(e) => return Err(e),
    }
}
