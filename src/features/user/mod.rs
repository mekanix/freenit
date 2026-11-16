use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use topcoat::{Result, context::Cx, router::error::bad_request};

use crate::config::{AuthBackend, Config};

#[cfg(feature = "ldap")]
pub(crate) mod ldap;
#[cfg(feature = "sql")]
pub(crate) mod sql;

#[cfg(feature = "ldap")]
pub use ldap::class_filter;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UserId {
    Sql(u64),
    Ldap(String),
}

impl UserId {
    pub fn as_sql(&self) -> Option<u64> {
        match self {
            Self::Sql(id) => Some(*id),
            Self::Ldap(_) => None,
        }
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sql(id) => id.fmt(formatter),
            Self::Ldap(dn) => dn.fmt(formatter),
        }
    }
}

impl FromStr for UserId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = value.parse::<u64>() {
            Ok(Self::Sql(id))
        } else if value.is_empty() {
            Err("user ID cannot be empty".to_owned())
        } else {
            Ok(Self::Ldap(value.to_owned()))
        }
    }
}

impl Serialize for UserId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub fullname: Option<String>,
    pub active: bool,
    pub admin: bool,
    pub omemo_bundle: Option<String>,
    pub uid_number: Option<u64>,
}

impl User {
    pub async fn list(cx: &Cx) -> Result<Vec<Self>> {
        match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                return Ok(sql::User::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                return Ok(ldap::User::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        }

    }

    pub async fn get(cx: &Cx, id: &UserId) -> Result<Self> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => {
                sql::User::get(cx, *id).await.map(Into::into)
            }
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::get_by_dn(cx, dn).await.map(Into::into)
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn find_by_email(cx: &Cx, email: &str) -> Result<Option<Self>> {
        let user = match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                {
                    sql::User::find_by_email(cx, email)
                        .await?
                        .map(Into::into)
                }
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                {
                    ldap::User::find_by_email(cx, email)
                        .await?
                        .map(Into::into)
                }
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        };
        Ok(user)
    }

    pub async fn authenticate(cx: &Cx, email: &str, password: String) -> Result<Option<Self>> {
        let user = match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                {
                    sql::User::authenticate(cx, email, password)
                        .await?
                        .map(Into::into)
                }
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                {
                    ldap::User::authenticate(cx, email, password)
                        .await?
                        .map(Into::into)
                }
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        };
        Ok(user)
    }

    pub async fn from_token(cx: &Cx, id: &str) -> Result<Self> {
        let id = id
            .parse()
            .map_err(|_| topcoat::router::error::unauthorized())?;
        Self::get(cx, &id).await
    }

    pub fn token_id(&self) -> String {
        self.id.to_string()
    }

    pub async fn add(
        cx: &Cx,
        email: &str,
        password: String,
        fullname: &str,
        active: bool,
        admin: bool,
    ) -> Result<Self> {
        match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                return sql::User::add(cx, email, password, fullname, active, admin)
                    .await
                    .map(Into::into);
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                return ldap::User::add(cx, email, password, fullname, active, admin)
                    .await
                    .map(Into::into);
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        }
    }

    pub async fn activate(cx: &Cx, id: &UserId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::activate(cx, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::activate(cx, ldap_uid(cx, dn).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn set_password(cx: &Cx, id: &UserId, password: String) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::set_password(cx, *id, password).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::set_password(cx, ldap_uid(cx, dn).await?, password).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn toggle_active(cx: &Cx, id: &UserId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::toggle_active(cx, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::toggle_active(cx, ldap_uid(cx, dn).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn set_admin(cx: &Cx, id: &UserId, admin: bool) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::set_admin(cx, *id, admin).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::set_admin(cx, ldap_uid(cx, dn).await?, admin).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn toggle_admin(cx: &Cx, id: &UserId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::toggle_admin(cx, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::toggle_admin(cx, ldap_uid(cx, dn).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    #[cfg(feature = "jabber")]
    pub fn omemo_key(&self) -> String {
        self.id.to_string()
    }

    #[cfg(feature = "jabber")]
    pub async fn set_omemo_bundle(cx: &Cx, id: &UserId, bundle: Option<String>) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::set_omemo_bundle(cx, *id, bundle).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::set_omemo_bundle(cx, ldap_uid(cx, dn).await?, bundle).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn remove(cx: &Cx, id: &UserId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::User::remove(cx, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(dn)) => {
                ldap::User::remove(cx, ldap_uid(cx, dn).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }
}

#[cfg(feature = "ldap")]
async fn ldap_uid(cx: &Cx, dn: &str) -> Result<u64> {
    Ok(ldap::User::get_by_dn(cx, dn).await?.id)
}

fn backend(cx: &Cx) -> AuthBackend {
    topcoat::context::context::<Config>(cx).auth.backend
}

#[cfg(any(not(feature = "sql"), not(feature = "ldap")))]
fn backend_unavailable(cx: &Cx) -> topcoat::Error {
    bad_request(format!(
        "authentication backend `{}` is not enabled",
        backend(cx).as_str()
    ))
    .into()
}
