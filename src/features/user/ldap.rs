use std::collections::HashSet;

use ldap3::{Ldap, Mod, Scope, SearchEntry, exop::PasswordModify, ldap_escape};
use topcoat::{
    Result,
    context::Cx,
    router::error::{RouterErrorExt, bad_request, internal_server_error},
};

use crate::db::{LdapConfig, ldap_config};

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub dn: String,
    pub email: String,
    pub fullname: Option<String>,
    pub active: bool,
    pub admin: bool,
    pub omemo_bundle: Option<String>,
    pub user_class: Vec<String>,
}

impl User {
    pub async fn list(cx: &Cx) -> Result<Vec<User>> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let (entries, _) = ldap
            .search(
                &config.user_base,
                Scope::Subtree,
                "(objectClass=person)",
                vec!["*", config.user_member_attr.as_str()],
            )
            .await?
            .success()?;
        let mut users = entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(|entry| Self::from_entry(entry, &config))
            .collect::<Result<Vec<_>>>()?;
        users.sort_by_key(|user| user.id);
        Ok(users)
    }

    pub async fn get(cx: &Cx, id: u64) -> Result<User> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let filter = format!("(&(objectClass=person)(uidNumber={id}))");
        let (entries, _) = ldap
            .search(
                &config.user_base,
                Scope::Subtree,
                &filter,
                vec!["*", config.user_member_attr.as_str()],
            )
            .await?
            .success()?;
        let entry = one_entry(entries)?;
        Self::from_entry(entry, &config)
    }

    pub async fn get_by_dn(cx: &Cx, dn: &str) -> Result<User> {
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let (entries, _) = ldap
            .search(
                dn,
                Scope::Base,
                "(objectClass=person)",
                vec!["*", config.user_member_attr.as_str()],
            )
            .await?
            .success()?;
        let entry = one_entry(entries)?;
        Self::from_entry(entry, &config)
    }

    pub async fn find_by_email(cx: &Cx, email: &str) -> Result<Option<User>> {
        let config = ldap_config(cx);
        let dn = config.user_dn(&email.trim().to_lowercase());
        let mut ldap = config.connect().await?;
        let (entries, _) = ldap
            .search(&dn, Scope::Base, "(objectClass=person)", vec!["dn"])
            .await?
            .success()?;
        if entries.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self::get_by_dn(cx, &dn).await?))
    }

    pub async fn authenticate(cx: &Cx, email: &str, password: String) -> Result<Option<User>> {
        let Some(user) = Self::find_by_email(cx, email).await? else {
            return Ok(None);
        };
        let config = ldap_config(cx);
        let mut ldap = config.connect_unbound().await?;
        if ldap
            .simple_bind(&user.dn, &password)
            .await?
            .success()
            .is_err()
        {
            return Ok(None);
        }
        Ok(Some(user))
    }

    pub async fn from_token(cx: &Cx, dn: &str) -> Result<User> {
        Self::get_by_dn(cx, dn).await
    }

    pub fn token_id(&self) -> String {
        self.dn.clone()
    }

    #[cfg(feature = "jabber")]
    pub fn omemo_key(&self) -> String {
        self.dn.clone()
    }

    #[cfg(feature = "jabber")]
    pub async fn set_omemo_bundle(cx: &Cx, id: u64, bundle: Option<String>) -> Result<()> {
        let user = Self::get(cx, id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let values = if config.user_omemo_attr == "description" {
            let (entries, _) = ldap
                .search(
                    &user.dn,
                    Scope::Base,
                    "(objectClass=person)",
                    vec!["description"],
                )
                .await?
                .success()?;
            let entry = one_entry(entries)?;
            let mut descriptions = entry.attrs.get("description").cloned().unwrap_or_default();
            descriptions.retain(|description| !description.starts_with("__OMEMO__:"));
            if let Some(bundle) = bundle {
                descriptions.push(format!("__OMEMO__:{bundle}"));
            }
            descriptions.into_iter().collect::<HashSet<_>>()
        } else {
            bundle.into_iter().collect::<HashSet<_>>()
        };
        ldap.modify(
            &user.dn,
            vec![Mod::Replace(config.user_omemo_attr.clone(), values)],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn add(
        cx: &Cx,
        email: &str,
        password: String,
        fullname: &str,
        active: bool,
        admin: bool,
    ) -> Result<User> {
        let email = email.trim().to_lowercase();
        let Some((uid, _)) = email.split_once('@') else {
            return Err(bad_request("email must contain a domain").into());
        };
        if uid.is_empty() || password.is_empty() {
            return Err(bad_request("email and password are required").into());
        }

        let config = ldap_config(cx);
        let dn = config.user_dn(&email);
        let mut ldap = config.connect().await?;
        let (existing, _) = ldap
            .search(&dn, Scope::Base, "(objectClass=person)", vec!["dn"])
            .await?
            .success()?;
        if !existing.is_empty() {
            return Err(bad_request("user already exists").into());
        }

        let uid_number = next_number(
            &mut ldap,
            &config.uid_next_dn,
            &config.uid_next_class,
            &config.uid_next_field,
        )
        .await?;
        let gid_number = next_number(
            &mut ldap,
            &config.gid_next_dn,
            &config.gid_next_class,
            &config.gid_next_field,
        )
        .await?;
        let group_dn = config.group_dn(uid, &email);
        ldap.add(
            &group_dn,
            vec![
                attribute("objectClass", config.group_classes.iter().cloned()),
                attribute("cn", [uid.to_owned()]),
                attribute("gidNumber", [gid_number.to_string()]),
                attribute("memberUid", [uid_number.to_string()]),
            ],
        )
        .await?
        .success()?;

        let mut user_class = Vec::new();
        if active {
            user_class.push("active".to_owned());
        }
        if admin {
            user_class.push("admin".to_owned());
        }
        let common_name = fullname.trim();
        let common_name = if common_name.is_empty() {
            uid
        } else {
            common_name
        };
        let attrs = vec![
            attribute("objectClass", config.user_classes.iter().cloned()),
            attribute("uid", [uid.to_owned()]),
            attribute("cn", [common_name.to_owned()]),
            attribute("sn", [uid.to_owned()]),
            attribute("uidNumber", [uid_number.to_string()]),
            attribute("gidNumber", [gid_number.to_string()]),
            attribute("userClass", user_class),
            attribute("homeDirectory", [format!("/home/{uid}")]),
            attribute("mail", [email]),
        ];
        if let Err(error) = ldap.add(&dn, attrs).await?.success() {
            let _ = ldap.delete(&group_dn).await;
            return Err(error.into());
        }

        let password_result = ldap
            .extended(PasswordModify {
                user_id: Some(&dn),
                old_pass: None,
                new_pass: Some(&password),
            })
            .await?
            .success();
        if let Err(error) = password_result {
            let _ = ldap.delete(&dn).await;
            let _ = ldap.delete(&group_dn).await;
            return Err(error.into());
        }
        Self::get(cx, uid_number).await
    }

    pub async fn activate(cx: &Cx, id: u64) -> Result<()> {
        let user = Self::get(cx, id).await?;
        Self::set_flag(cx, &user, "active", true).await
    }

    pub async fn set_password(cx: &Cx, id: u64, password: String) -> Result<()> {
        if password.is_empty() {
            return Err(bad_request("password is required").into());
        }
        let user = Self::get(cx, id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.extended(PasswordModify {
            user_id: Some(&user.dn),
            old_pass: None,
            new_pass: Some(&password),
        })
        .await?
        .success()?;
        Ok(())
    }

    pub async fn toggle_active(cx: &Cx, id: u64) -> Result<()> {
        let user = Self::get(cx, id).await?;
        Self::set_flag(cx, &user, "active", !user.active).await
    }

    pub async fn set_admin(cx: &Cx, id: u64, admin: bool) -> Result<()> {
        let user = Self::get(cx, id).await?;
        Self::set_flag(cx, &user, "admin", admin).await
    }

    pub async fn toggle_admin(cx: &Cx, id: u64) -> Result<()> {
        let user = Self::get(cx, id).await?;
        Self::set_flag(cx, &user, "admin", !user.admin).await
    }

    async fn set_flag(cx: &Cx, user: &User, flag: &str, enabled: bool) -> Result<()> {
        let mut classes = user.user_class.clone();
        if user.active {
            classes.push("active".to_owned());
        }
        if user.admin {
            classes.push("admin".to_owned());
        }
        if enabled {
            if !classes.iter().any(|class| class == flag) {
                classes.push(flag.to_owned());
            }
        } else {
            classes.retain(|class| class != flag);
        }

        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        ldap.modify(
            &user.dn,
            vec![Mod::Replace(
                "userClass".to_owned(),
                classes.into_iter().collect::<HashSet<_>>(),
            )],
        )
        .await?
        .success()?;
        Ok(())
    }

    pub async fn remove(cx: &Cx, id: u64) -> Result<()> {
        let user = Self::get(cx, id).await?;
        let config = ldap_config(cx);
        let mut ldap = config.connect().await?;
        let role_classes = class_filter(&config.role_classes);
        let filter = format!(
            "(&({}={}){})",
            config.role_member_attr,
            ldap_escape(&user.dn),
            role_classes
        );
        let (roles, _) = ldap
            .search(
                &config.role_base,
                Scope::Subtree,
                &filter,
                vec!["cn", &config.role_member_attr],
            )
            .await?
            .success()?;
        for entry in roles.into_iter().map(SearchEntry::construct) {
            let members = entry
                .attrs
                .get(&config.role_member_attr)
                .map_or(0, Vec::len);
            if members == 1 {
                return Err(bad_request("cannot delete the only member of a role").into());
            }
            ldap.modify(
                &entry.dn,
                vec![Mod::Delete(
                    config.role_member_attr.clone(),
                    HashSet::from([user.dn.clone()]),
                )],
            )
            .await?
            .success()?;
        }

        let group_classes = class_filter(&config.group_classes);
        let filter = format!("(&(memberUid={id}){group_classes})");
        let (groups, _) = ldap
            .search(
                &config.role_base,
                Scope::Subtree,
                &filter,
                vec!["memberUid"],
            )
            .await?
            .success()?;
        for entry in groups.into_iter().map(SearchEntry::construct) {
            if entry.attrs.get("memberUid").map_or(0, Vec::len) == 1 {
                ldap.delete(&entry.dn).await?.success()?;
            }
        }
        ldap.delete(&user.dn).await?.success()?;
        Ok(())
    }

    fn from_entry(entry: SearchEntry, config: &LdapConfig) -> Result<User> {
        let mut user_class = entry.attrs.get("userClass").cloned().unwrap_or_default();
        let active = remove_class(&mut user_class, "active");
        let admin = remove_class(&mut user_class, "admin");
        let omemo_bundle = if config.user_omemo_attr == "description" {
            entry
                .attrs
                .get("description")
                .and_then(|descriptions| {
                    descriptions
                        .iter()
                        .find_map(|description| description.strip_prefix("__OMEMO__:"))
                })
                .map(str::to_owned)
        } else {
            entry
                .attrs
                .get(&config.user_omemo_attr)
                .and_then(|values| values.first())
                .cloned()
        };

        Ok(User {
            id: parse_attr(&entry, "uidNumber")?,
            dn: entry.dn.clone(),
            email: attr(&entry, "mail")?.to_owned(),
            fullname: Some(attr(&entry, "cn")?.to_owned()),
            active,
            admin,
            omemo_bundle,
            user_class,
        })
    }
}

async fn next_number(ldap: &mut Ldap, dn: &str, class: &str, field: &str) -> Result<u64> {
    ldap.modify(dn, vec![Mod::Increment(field, "1")])
        .await?
        .success()?;
    let filter = format!("(objectClass={})", ldap_escape(class));
    let (entries, _) = ldap
        .search(dn, Scope::Base, &filter, vec![field])
        .await?
        .success()?;
    parse_attr(&one_entry(entries)?, field)
}

fn one_entry(entries: Vec<ldap3::ResultEntry>) -> Result<SearchEntry> {
    if entries.len() > 1 {
        return Err(
            internal_server_error(std::io::Error::other("multiple LDAP entries found")).into(),
        );
    }
    let entry = entries.into_iter().next().ok_or_not_found()?;
    Ok(SearchEntry::construct(entry))
}

fn attr<'a>(entry: &'a SearchEntry, name: &str) -> Result<&'a str> {
    Ok(entry
        .attrs
        .get(name)
        .and_then(|values| values.first())
        .map(String::as_str)
        .ok_or_else(|| {
            internal_server_error(std::io::Error::other(format!(
                "LDAP entry {} has no {name} attribute",
                entry.dn
            )))
        })?)
}

fn parse_attr(entry: &SearchEntry, name: &str) -> Result<u64> {
    Ok(attr(entry, name)?.parse().map_err(internal_server_error)?)
}

fn attribute(
    name: impl Into<String>,
    values: impl IntoIterator<Item = String>,
) -> (String, HashSet<String>) {
    (name.into(), values.into_iter().collect())
}

pub fn class_filter(classes: &[String]) -> String {
    classes
        .iter()
        .map(|class| format!("(objectClass={})", ldap_escape(class)))
        .collect()
}

fn remove_class(classes: &mut Vec<String>, class: &str) -> bool {
    if let Some(index) = classes.iter().position(|candidate| candidate == class) {
        classes.remove(index);
        true
    } else {
        false
    }
}


impl From<User> for super::User {
    fn from(user: User) -> Self {
        Self {
            id: super::UserId::Ldap(user.dn),
            email: user.email,
            fullname: user.fullname,
            active: user.active,
            admin: user.admin,
            omemo_bundle: user.omemo_bundle,
            uid_number: Some(user.id),
        }
    }
}
