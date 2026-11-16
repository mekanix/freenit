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
pub use sql::UserGroup;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GroupId {
    Sql(u64),
    Ldap(String),
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sql(id) => id.fmt(formatter),
            Self::Ldap(dn) => dn.fmt(formatter),
        }
    }
}

impl FromStr for GroupId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = value.parse::<u64>() {
            Ok(Self::Sql(id))
        } else if value.is_empty() {
            Err("group ID cannot be empty".to_owned())
        } else {
            Ok(Self::Ldap(value.to_owned()))
        }
    }
}

impl Serialize for GroupId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GroupId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
}

impl Group {
    pub async fn list(cx: &Cx) -> Result<Vec<Self>> {
        match backend(cx) {
            AuthBackend::Sql => {
                #[cfg(feature = "sql")]
                return Ok(sql::Group::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "sql"))]
                return Err(backend_unavailable(cx).into());
            }
            AuthBackend::Ldap => {
                #[cfg(feature = "ldap")]
                return Ok(ldap::Group::list(cx)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect());
                #[cfg(not(feature = "ldap"))]
                return Err(backend_unavailable(cx).into());
            }
        }

    }

    pub async fn get(cx: &Cx, id: &GroupId) -> Result<Self> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, GroupId::Sql(id)) => sql::Group::get(cx, id).await.map(Into::into),
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, GroupId::Ldap(dn)) => {
                ldap::Group::get(cx, dn).await.map(Into::into)
            }
            _ => Err(bad_request("group ID does not match the authentication backend").into()),
        }
    }

    pub async fn add(cx: &Cx, name: &str, domain: &str, user_id: &UserId) -> Result<()> {
        match (backend(cx), user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, UserId::Sql(id)) => sql::Group::add(cx, name, domain, *id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, UserId::Ldap(_)) => {
                ldap::Group::add(cx, name, domain, user_uid(cx, user_id).await?).await
            }
            _ => Err(bad_request("user ID does not match the authentication backend").into()),
        }
    }

    pub async fn remove(cx: &Cx, id: &GroupId) -> Result<()> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, GroupId::Sql(id)) => sql::Group::remove(cx, id).await,
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, GroupId::Ldap(dn)) => ldap::Group::remove(cx, dn).await,
            _ => Err(bad_request("group ID does not match the authentication backend").into()),
        }
    }

    pub async fn users(cx: &Cx, id: &GroupId) -> Result<Vec<User>> {
        match (backend(cx), id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, GroupId::Sql(id)) => Ok(sql::Group::users(cx, id)
                .await?
                .into_iter()
                .map(Into::into)
                .collect()),
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, GroupId::Ldap(dn)) => Ok(ldap::Group::users(cx, dn)
                .await?
                .into_iter()
                .map(Into::into)
                .collect()),
            _ => Err(bad_request("group ID does not match the authentication backend").into()),
        }
    }

    pub async fn assign(cx: &Cx, id: &GroupId, user_id: &UserId) -> Result<()> {
        match (backend(cx), id, user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, GroupId::Sql(group_id), UserId::Sql(user_id)) => {
                sql::Group::assign(cx, group_id, *user_id).await
            }
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, GroupId::Ldap(dn), UserId::Ldap(_)) => {
                ldap::Group::assign(cx, dn, user_uid(cx, user_id).await?).await
            }
            _ => Err(bad_request("IDs do not match the authentication backend").into()),
        }
    }

    pub async fn unassign(cx: &Cx, id: &GroupId, user_id: &UserId) -> Result<()> {
        match (backend(cx), id, user_id) {
            #[cfg(feature = "sql")]
            (AuthBackend::Sql, GroupId::Sql(group_id), UserId::Sql(user_id)) => {
                sql::Group::unassign(cx, group_id, *user_id).await
            }
            #[cfg(feature = "ldap")]
            (AuthBackend::Ldap, GroupId::Ldap(dn), UserId::Ldap(_)) => {
                ldap::Group::unassign(cx, dn, user_uid(cx, user_id).await?).await
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
