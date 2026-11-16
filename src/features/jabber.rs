use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use fernet::Fernet;
use serde_json::Value;
use sha2::{Digest, Sha256};
use topcoat::{
    Result,
    context::{Cx, context},
    router::error::{bad_request, internal_server_error},
};

pub use crate::config::JabberConfig;
use crate::{config::Config, features::user::User};

pub fn decrypt_bundle(cx: &Cx, user: &User) -> Result<Option<Value>> {
    let Some(encrypted) = &user.omemo_bundle else {
        return Ok(None);
    };
    if encrypted.is_empty() {
        return Ok(None);
    }
    let decrypted = fernet(cx, user)
        .decrypt(encrypted)
        .map_err(internal_server_error)?;
    Ok(Some(
        serde_json::from_slice(&decrypted).map_err(internal_server_error)?,
    ))
}

pub fn encrypt_bundle(cx: &Cx, user: &User, input: &str) -> Result<Option<String>> {
    if input.trim().is_empty() {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(input)
        .map_err(|error| bad_request(format!("invalid OMEMO JSON: {error}")))?;
    Ok(Some(fernet(cx, user).encrypt(value.to_string().as_bytes())))
}

fn fernet(cx: &Cx, user: &User) -> Fernet {
    let config = context::<Config>(cx);
    let key_material = format!("{}:{}", config.auth.secret, user.omemo_key());
    let key = URL_SAFE_NO_PAD.encode(Sha256::digest(key_material.as_bytes()));
    Fernet::new(&key).expect("a SHA-256 digest is a valid Fernet key")
}
