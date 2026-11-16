use std::collections::HashSet;

use ldap3::{Mod, Scope, SearchEntry, ldap_escape};
use topcoat::{
    Result,
    context::Cx,
    router::error::{RouterErrorExt, bad_request, internal_server_error},
};

use crate::{
    db::{LdapConfig, ldap_config},
    features::user::{class_filter, ldap::User},
};

pub type RoleId = String;

#[derive(Debug)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub dn: String,
    pub members: Vec<String>,
}

impl Role {
    pub async fn list(cx: &Cx) -> Result<Vec<Role>> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let filter = format!("(|{})", class_filter(&config.role_classes));
        let (entries, _) = ldap
            .search(
                &config.role_base,
                Scope::Subtree,
                &filter,
                vec!["cn", config.role_member_attr.as_str()],
            )
            .await?
            .success()?;
        let mut roles = entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(|entry| Self::from_entry(entry, &config))
            .collect::<Result<Vec<_>>>()?;
        roles.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(roles)
    }

    pub async fn get(cx: &Cx, id: &RoleId) -> Result<Role> {
        let config = ldap_config(cx);
        let dn = config.role_dn(id);
        let mut ldap = config.connect().await?;
        let filter = format!("(|{})", class_filter(&config.role_classes));
        let (entries, _) = ldap
            .search(
                &dn,
                Scope::Base,
                &filter,
                vec!["cn", config.role_member_attr.as_str()],
            )
            .await?
            .success()?;
        if entries.len() > 1 {
            return Err(
                internal_server_error(std::io::Error::other("multiple LDAP roles found")).into(),
            );
        }
        let entry = entries.into_iter().next().ok_or_not_found()?;
        Self::from_entry(SearchEntry::construct(entry), &config)
    }

    pub async fn add(cx: &Cx, name: &str, user_id: u64) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(());
        }
        let user = User::get(cx, user_id).await?;
        let config = ldap_config(cx);
        let dn = config.role_dn(name);
        let mut ldap = config.connect().await?;
        let (existing, _) = ldap
            .search(&dn, Scope::Base, "(objectClass=*)", vec!["dn"])
            .await?
            .success()?;
        if !existing.is_empty() {
            return Ok(());
        }

        ldap.add(
            &dn,
            vec![
                (
                    "objectClass",
                    config
                        .role_classes
                        .iter()
                        .map(String::as_str)
                        .collect::<HashSet<_>>(),
                ),
                ("cn", HashSet::from([name])),
                (
                    config.role_member_attr.as_str(),
                    HashSet::from([user.dn.as_str()]),
                ),
            ],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: &RoleId) -> Result<()> {
        let role = Self::get(cx, id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.delete(&role.dn).await?.success()?;
        Ok(())
    }

    pub async fn for_user(cx: &Cx, user_id: u64) -> Result<Vec<Role>> {
        let user = User::get(cx, user_id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let filter = format!(
            "(&({}={}){})",
            config.role_member_attr,
            ldap_escape(&user.dn),
            class_filter(&config.role_classes)
        );
        let (entries, _) = ldap
            .search(
                &config.role_base,
                Scope::Subtree,
                &filter,
                vec!["cn", config.role_member_attr.as_str()],
            )
            .await?
            .success()?;
        let mut roles = entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(|entry| Self::from_entry(entry, &config))
            .collect::<Result<Vec<_>>>()?;
        roles.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(roles)
    }

    pub async fn users(cx: &Cx, role_id: &RoleId) -> Result<Vec<User>> {
        let role = Self::get(cx, role_id).await?;
        let mut users = Vec::with_capacity(role.members.len());
        for dn in role.members {
            users.push(User::get_by_dn(cx, &dn).await?);
        }
        users.sort_by_key(|user| user.id);
        Ok(users)
    }

    pub async fn assign(cx: &Cx, role_id: &RoleId, user_id: u64) -> Result<()> {
        let role = Self::get(cx, role_id).await?;
        let user = User::get(cx, user_id).await?;
        if role.members.contains(&user.dn) {
            return Ok(());
        }

        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.modify(
            &role.dn,
            vec![Mod::Add(
                config.role_member_attr.as_str(),
                HashSet::from([user.dn.as_str()]),
            )],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn unassign(cx: &Cx, role_id: &RoleId, user_id: u64) -> Result<()> {
        let role = Self::get(cx, role_id).await?;
        let user = User::get(cx, user_id).await?;
        if !role.members.contains(&user.dn) {
            return Ok(());
        }
        if role.members.len() == 1 {
            return Err(bad_request("cannot remove the last role member").into());
        }

        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.modify(
            &role.dn,
            vec![Mod::Delete(
                config.role_member_attr.as_str(),
                HashSet::from([user.dn.as_str()]),
            )],
        )
        .await?
        .success()?;
        Ok(())
    }

    fn from_entry(entry: SearchEntry, config: &LdapConfig) -> Result<Role> {
        let name = entry
            .attrs
            .get("cn")
            .and_then(|values| values.first())
            .ok_or_else(|| {
                internal_server_error(std::io::Error::other(format!(
                    "LDAP role {} has no cn attribute",
                    entry.dn
                )))
            })?
            .clone();
        Ok(Role {
            id: name.clone(),
            name,
            dn: entry.dn,
            members: entry
                .attrs
                .get(&config.role_member_attr)
                .cloned()
                .unwrap_or_default(),
        })
    }
}


impl From<Role> for super::Role {
    fn from(role: Role) -> Self {
        Self {
            id: super::RoleId::Ldap(role.dn),
            name: role.name,
        }
    }
}
