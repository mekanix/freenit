use crate::{
    db::{LdapConfig, ldap_config},
    features::user::{class_filter, ldap::User},
};
use ldap3::{Mod, Scope, SearchEntry, ldap_escape};
use std::collections::HashSet;
use topcoat::{
    Result,
    context::Cx,
    router::error::{RouterErrorExt, bad_request, internal_server_error},
};

pub type GroupId = String;

#[derive(Debug)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub dn: String,
    pub members: Vec<String>,
}

impl Group {
    pub async fn list(cx: &Cx) -> Result<Vec<Group>> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let filter = format!("(|{})", class_filter(&config.group_classes));
        let (entries, _) = ldap
            .search(
                &config.role_base,
                Scope::Subtree,
                &filter,
                vec!["cn", "gidNumber", "memberUid"],
            )
            .await?
            .success()?;
        let mut groups = entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(Self::from_entry)
            .collect::<Result<Vec<_>>>()?;
        groups.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(groups)
    }

    pub async fn get(cx: &Cx, id: &GroupId) -> Result<Group> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let filter = format!("(|{})", class_filter(&config.group_classes));
        let (entries, _) = ldap
            .search(
                id,
                Scope::Base,
                &filter,
                vec!["cn", "gidNumber", "memberUid"],
            )
            .await?
            .success()?;
        if entries.len() > 1 {
            return Err(
                internal_server_error(std::io::Error::other("multiple LDAP groups found")).into(),
            );
        }
        Self::from_entry(SearchEntry::construct(
            entries.into_iter().next().ok_or_not_found()?,
        ))
    }

    pub async fn add(cx: &Cx, name: &str, domain: &str, user_id: u64) -> Result<()> {
        let name = name.trim();
        let domain = domain.trim();
        if name.is_empty() || domain.is_empty() {
            return Err(bad_request("group name and domain are required").into());
        }
        let user = User::get(cx, user_id).await?;
        let config = ldap_config(cx);
        let dn = config.group_dn(name, &format!("group@{domain}"));
        let mut ldap = config.connect().await?;
        let gid = next_gid(&mut ldap, &config).await?;
        ldap.add(
            &dn,
            vec![
                (
                    "objectClass".to_owned(),
                    config.group_classes.iter().cloned().collect::<HashSet<_>>(),
                ),
                ("cn".to_owned(), HashSet::from([name.to_owned()])),
                ("gidNumber".to_owned(), HashSet::from([gid.to_string()])),
                ("memberUid".to_owned(), HashSet::from([user.id.to_string()])),
            ],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: &GroupId) -> Result<()> {
        let group = Self::get(cx, id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.delete(&group.dn).await?.success()?;
        Ok(())
    }

    pub async fn users(cx: &Cx, group_id: &GroupId) -> Result<Vec<User>> {
        let group = Self::get(cx, group_id).await?;
        let mut users = Vec::with_capacity(group.members.len());
        for id in group.members {
            let id = id
                .parse()
                .map_err(|_| bad_request("group contains an invalid member UID"))?;
            users.push(User::get(cx, id).await?);
        }
        users.sort_by_key(|user| user.id);
        Ok(users)
    }

    pub async fn assign(cx: &Cx, group_id: &GroupId, user_id: u64) -> Result<()> {
        let group = Self::get(cx, group_id).await?;
        let uid = user_id.to_string();
        if group.members.contains(&uid) {
            return Ok(());
        }
        User::get(cx, user_id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.modify(
            &group.dn,
            vec![Mod::Add("memberUid", HashSet::from([uid.as_str()]))],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn unassign(cx: &Cx, group_id: &GroupId, user_id: u64) -> Result<()> {
        let group = Self::get(cx, group_id).await?;
        let uid = user_id.to_string();
        if !group.members.contains(&uid) {
            return Ok(());
        }
        if group.members.len() == 1 {
            return Err(bad_request("cannot remove the last group member").into());
        }
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.modify(
            &group.dn,
            vec![Mod::Delete("memberUid", HashSet::from([uid.as_str()]))],
        )
        .await?
        .success()?;
        Ok(())
    }

    fn from_entry(entry: SearchEntry) -> Result<Group> {
        let name = required(&entry, "cn")?.to_owned();
        required(&entry, "gidNumber")?
            .parse::<u64>()
            .map_err(internal_server_error)?;
        Ok(Group {
            id: entry.dn.clone(),
            name,
            dn: entry.dn,
            members: entry.attrs.get("memberUid").cloned().unwrap_or_default(),
        })
    }
}

fn required<'a>(entry: &'a SearchEntry, name: &str) -> Result<&'a str> {
    entry
        .attrs
        .get(name)
        .and_then(|values| values.first())
        .map(String::as_str)
        .ok_or_else(|| {
            internal_server_error(std::io::Error::other(format!(
                "LDAP group {} has no {name}",
                entry.dn
            )))
            .into()
        })
}

async fn next_gid(ldap: &mut ldap3::Ldap, config: &LdapConfig) -> Result<u64> {
    ldap.modify(
        &config.gid_next_dn,
        vec![Mod::Increment(
            config.gid_next_field.clone(),
            "1".to_owned(),
        )],
    )
    .await?
    .success()?;
    let filter = format!("(objectClass={})", ldap_escape(&config.gid_next_class));
    let (entries, _) = ldap
        .search(
            &config.gid_next_dn,
            Scope::Base,
            &filter,
            vec![&config.gid_next_field],
        )
        .await?
        .success()?;
    let entry = SearchEntry::construct(entries.into_iter().next().ok_or_not_found()?);
    Ok(required(&entry, &config.gid_next_field)?
        .parse()
        .map_err(internal_server_error)?)
}


impl From<Group> for super::Group {
    fn from(group: Group) -> Self {
        Self {
            id: super::GroupId::Ldap(group.dn),
            name: group.name,
        }
    }
}
