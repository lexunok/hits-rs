use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveValue::Set, DbConn, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use entity::users;
use entity::users::Entity as User;
use crate::error::GlobalError;

pub async fn create_admin(db:DbConn, username: String, password: String) -> Result<(), GlobalError> {
    let user: Option<users::Model> = User::find_by_email(username.clone())
        .one(&db)
        .await
        .map_err(GlobalError::DbErr)?;

    if let None = user {
        
        let user = users::ActiveModel {
            first_name: Set("Живая".to_owned()),
            last_name: Set("Легенда".to_owned()),
            roles: Set(vec!["ADMIN".to_owned(), "INITIATOR".to_owned()]),
            email: Set(username),
            password: Set(hash_password(&password)?),
            ..Default::default()
        };

        user.insert(&db).await.map_err(GlobalError::DbErr)?;
    }

    Ok(())
}



pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}
impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub first_name: String,
    pub last_name: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: TokenType,
    pub roles: Vec<String>,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = GlobalError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let access_token = jar
            .get("access_token")
            .ok_or(GlobalError::WrongCredentials)?
            .value()
            .to_string();

        let token_data = decode::<Claims>(&access_token, &KEYS.decoding, &Validation::default())
            .map_err(|_| GlobalError::InvalidToken)?;

        if token_data.claims.token_type != TokenType::Access {
            return Err(GlobalError::InvalidToken);
        }

        Ok(token_data.claims)
    }
}

pub fn generate_tokens(
    sub: String,
    first_name: String,
    last_name: String,
    roles: Vec<String>,
) -> Result<CookieJar, GlobalError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(15)).timestamp() as usize;

    let claims = Claims {
        sub: sub.clone(),
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        iat,
        exp,
        token_type: TokenType::Access,
        roles: roles.clone(),
    };

    let access_token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| GlobalError::TokenCreation)?;

    let exp = (now + Duration::days(7)).timestamp() as usize;

    let claims = Claims {
        sub,
        first_name,
        last_name,
        iat,
        exp,
        token_type: TokenType::Refresh,
        roles,
    };

    let refresh_token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| GlobalError::TokenCreation)?;

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(time::Duration::minutes(30));

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(time::Duration::days(30));

    Ok(CookieJar::new().add(access_cookie).add(refresh_cookie))
}

pub fn hash_password(password: &str) -> Result<String, GlobalError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| GlobalError::InternalServerError)?;
    Ok(password_hash.to_string())
}
pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
