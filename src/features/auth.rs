use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::{Cx, context},
    cookie::{Cookie, Cookies, SameSite, cookies},
    router::error::{forbidden, unauthorized},
};

use crate::{config::Config, features::user::User};

const ACCESS_SECONDS: u64 = 60 * 60;
const REFRESH_SECONDS: u64 = 365 * 24 * 60 * 60;
const RECOVERY_SECONDS: u64 = 60 * 60;

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenPurpose {
    Access,
    Refresh,
    Verify,
    Reset,
}

#[derive(Deserialize, Serialize)]
struct Claims {
    pk: String,
    #[serde(rename = "type")]
    backend: String,
    jid: String,
    exp: u64,
    purpose: TokenPurpose,
}

pub fn issue_token(cx: &Cx, user: &User, purpose: TokenPurpose) -> Result<String> {
    let lifetime = match purpose {
        TokenPurpose::Access => ACCESS_SECONDS,
        TokenPurpose::Refresh => REFRESH_SECONDS,
        TokenPurpose::Verify | TokenPurpose::Reset => RECOVERY_SECONDS,
    };
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(topcoat::router::error::internal_server_error)?
        .as_secs()
        + lifetime;
    let claims = Claims {
        pk: user.token_id(),
        backend: context::<Config>(cx).auth.backend.as_str().to_owned(),
        jid: user.email.clone(),
        exp,
        purpose,
    };
    let config = &context::<Config>(cx).auth;
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )?)
}

pub async fn login(cx: &Cx, email: &str, password: String) -> Result<()> {
    let user = User::authenticate(cx, email, password)
        .await?
        .filter(|user| user.active)
        .ok_or_else(unauthorized)?;
    let access = issue_token(cx, &user, TokenPurpose::Access)?;
    let refresh = issue_token(cx, &user, TokenPurpose::Refresh)?;
    set_session(cx, access, refresh);
    Ok(())
}

pub async fn register(cx: &Cx, email: &str, password: String) -> Result<()> {
    let user = User::add(cx, email, password, "", !cfg!(feature = "mail"), false).await?;
    #[cfg(feature = "mail")]
    crate::features::mail::send_verification(cx, &user).await?;
    #[cfg(not(feature = "mail"))]
    let _ = user;
    Ok(())
}

pub async fn verify(cx: &Cx, token: &str) -> Result<()> {
    let user = user_for_token(cx, token, TokenPurpose::Verify).await?;
    User::activate(cx, &user.id).await
}

pub async fn request_reset(cx: &Cx, email: &str) -> Result<()> {
    if let Some(user) = User::find_by_email(cx, email).await? {
        #[cfg(feature = "mail")]
        crate::features::mail::send_recovery(cx, &user).await?;
        #[cfg(not(feature = "mail"))]
        let _ = user;
    }
    Ok(())
}

pub async fn reset_password(cx: &Cx, token: &str, password: String) -> Result<()> {
    let user = user_for_token(cx, token, TokenPurpose::Reset).await?;
    User::set_password(cx, &user.id, password).await
}

pub async fn current_user(cx: &Cx) -> Result<Option<User>> {
    let Some(token) = cookies(cx).get("access") else {
        return Ok(None);
    };
    let Ok(claims) = decode_token(cx, token.value()) else {
        return Ok(None);
    };
    if claims.purpose != TokenPurpose::Access {
        return Ok(None);
    }
    let user = User::from_token(cx, &claims.pk).await?;
    Ok(user.active.then_some(user))
}

pub async fn require_user(cx: &Cx) -> Result<User> {
    current_user(cx).await?.ok_or_else(|| unauthorized().into())
}

pub async fn require_admin(cx: &Cx) -> Result<User> {
    let user = require_user(cx).await?;
    if !user.admin {
        return Err(forbidden().into());
    }
    Ok(user)
}

pub async fn user_for_token(cx: &Cx, token: &str, purpose: TokenPurpose) -> Result<User> {
    let claims = decode_token(cx, token)?;
    if claims.purpose != purpose {
        return Err(unauthorized().into());
    }
    User::from_token(cx, &claims.pk).await
}

pub fn set_session(cx: &Cx, access: String, refresh: String) {
    let config = &context::<Config>(cx).auth;
    for (name, value, max_age) in [
        ("access", access, ACCESS_SECONDS),
        ("refresh", refresh, REFRESH_SECONDS),
    ] {
        cookies(cx).add(
            Cookie::build((name, value))
                .path("/")
                .http_only(true)
                .secure(config.secure_cookies)
                .same_site(SameSite::Lax)
                .max_age(topcoat::cookie::time::Duration::seconds(max_age as i64))
                .build(),
        );
    }
}

pub fn clear_session(cx: &Cx) {
    for name in ["access", "refresh"] {
        cookies(cx).remove(Cookie::build((name, "")).path("/").build());
    }
}

fn decode_token(cx: &Cx, token: &str) -> Result<Claims> {
    let config = &context::<Config>(cx).auth;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| unauthorized().into())
}
