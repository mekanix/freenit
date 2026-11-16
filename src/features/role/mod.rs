use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use topcoat::{Result, context::Cx, router::error::bad_request};

use crate::{
    config::{AuthBackend, Config},
    features::user::{User, UserId},
};

#[cfg(feature = "ldap")]
mod ldap;
#[cfg(feature = "sql")]
mod sql;

#[cfg(feature = "sql")]
pub use sql::UserRole;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RoleId {
    Sql(u64),
    Ldap(String),
}

impl fmt::Display for RoleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sql(id) => id.fmt(formatter),
            Self::Ldap(dn) => dn.fmt(formatter),
        }
    }
}

impl FromStr for RoleId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = value.parse::<u64>() {
            Ok(Self::Sql(id))
        } else if value.is_empty() {
            Err("role ID cannot be empty".to_owned())
        } else {
            Ok(Self::Ldap(value.to_owned()))
        }
    }
}

impl Serialize for RoleId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RoleId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
}

impl Role {
    pub async fn list(cx: &Cx) -> Result<Vec<Self>> {
        match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                return Ok(sql::Role::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                return Ok(ldap::Role::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        }

    }

    pub async fn get(cx: &Cx, id: &RoleId) -> Result<Self> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, RoleId::Sql(id)) => sql::Role::get(cx, id).await.map(Into::into),
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, RoleId::Ldap(dn)) => ldap::Role::get(cx, dn).await.map(Into::into),
            _ => Err(bad_request("role ID does not match the authentication backend").into()),
        }
    }

    pub async fn add(cx: &Cx, name: &str, user_id: &UserId) -> Result<()> {
        match (backend(cx), user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::Role::add(cx, name, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(_)) => {
                ldap::Role::add(cx, name, user_uid(cx, user_id).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn remove(cx: &Cx, id: &RoleId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, RoleId::Sql(id)) => sql::Role::remove(cx, id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, RoleId::Ldap(dn)) => ldap::Role::remove(cx, dn).await,
            _ => Err(bad_request("role ID does not match the authentication backend").into()),
        }
    }

    pub async fn for_user(cx: &Cx, user_id: &UserId) -> Result<Vec<Self>> {
        match (backend(cx), user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => Ok(sql::Role::for_user(cx, *id)
                .await?
                .into_iter()
                .map(Into::into)
                .collect()),
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(_)) => Ok(ldap::Role::for_user(
                cx,
                user_uid(cx, user_id).await?,
            )
            .await?
            .into_iter()
            .map(Into::into)
            .collect()),
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn users(cx: &Cx, id: &RoleId) -> Result<Vec<User>> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, RoleId::Sql(id)) => Ok(sql::Role::users(cx, id)
                .await?
                .into_iter()
                .map(Into::into)
                .collect()),
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, RoleId::Ldap(dn)) => Ok(ldap::Role::users(cx, dn)
                .await?
                .into_iter()
                .map(Into::into)
                .collect()),
            _ => Err(bad_request("role ID does not match the authentication backend").into()),
        }
    }

    pub async fn assign(cx: &Cx, id: &RoleId, user_id: &UserId) -> Result<()> {
        match (backend(cx), id, user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, RoleId::Sql(role_id), UserId::Sql(user_id)) => {
                sql::Role::assign(cx, role_id, *user_id).await
            }
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, RoleId::Ldap(dn), UserId::Ldap(_)) => {
                ldap::Role::assign(cx, dn, user_uid(cx, user_id).await?).await
            }
            _ => Err(bad_request("IDs do not match the authentication backend").into()),
        }
    }

    pub async fn unassign(cx: &Cx, id: &RoleId, user_id: &UserId) -> Result<()> {
        match (backend(cx), id, user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, RoleId::Sql(role_id), UserId::Sql(user_id)) => {
                sql::Role::unassign(cx, role_id, *user_id).await
            }
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, RoleId::Ldap(dn), UserId::Ldap(_)) => {
                ldap::Role::unassign(cx, dn, user_uid(cx, user_id).await?).await
            }
            _ => Err(bad_request("IDs do not match the authentication backend").into()),
        }
    }
}

#[cfg(feature = "ldap")]
async fn user_uid(cx: &Cx, id: &UserId) -> Result<u64> {
    User::get(cx, id)
        .await?
        .uid_number
        .ok_or_else(|| bad_request("LDAP user has no uidNumber").into())
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
