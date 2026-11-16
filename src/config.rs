use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
#[derive(Default)]
pub struct Config {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub ldap: LdapConfig,
    pub stalwart: StalwartConfig,
    pub sieve: SieveConfig,
    pub calendar: ProtocolConfig,
    pub contacts: ProtocolConfig,
    pub files: FilesConfig,
    pub mail: MailConfig,
    pub jabber: JabberConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_owned(),
        }
    }
}


impl Config {
    pub fn load() -> Result<Self> {
        let conf_path = std::env::var_os("CONFIG")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::args()
                    .collect::<Vec<_>>()
                    .windows(2)
                    .find(|args| args[0] == "--config")
                    .map(|args| PathBuf::from(&args[1]))
            })
            .unwrap_or_else(|| PathBuf::from("freenit.toml"));
        if conf_path.exists() {
            let content = std::fs::read_to_string(&conf_path)
                .with_context(|| format!("failed to read {}", conf_path.display()))?;
            toml::from_str::<Self>(&content)
                .with_context(|| format!("failed to parse {} as TOML", conf_path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn args_without_config() -> Vec<String> {
        let mut filtered = Vec::new();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--config" {
                args.next();
            } else {
                filtered.push(arg);
            }
        }
        filtered
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct AuthConfig {
    pub backend: AuthBackend,
    pub secret: String,
    pub secure_cookies: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            backend: AuthBackend::Sql,
            secret: "SECRET".to_owned(),
            secure_cookies: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthBackend {
    #[default]
    Sql,
    Ldap,
}

impl AuthBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sql => "sql",
            Self::Ldap => "ldap",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:data.db".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct LdapConfig {
    pub url: String,
    pub starttls: bool,
    pub service_dn: String,
    pub service_password: String,
    pub user_base: String,
    pub role_base: String,
    pub user_classes: Vec<String>,
    pub group_classes: Vec<String>,
    pub role_classes: Vec<String>,
    pub role_member_attr: String,
    pub user_member_attr: String,
    pub user_omemo_attr: String,
    pub uid_next_dn: String,
    pub uid_next_class: String,
    pub uid_next_field: String,
    pub gid_next_dn: String,
    pub gid_next_class: String,
    pub gid_next_field: String,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            url: "ldap://127.0.0.1:389".to_owned(),
            starttls: false,
            service_dn: "cn=freenit,dc=service,dc=ldap".to_owned(),
            service_password: String::new(),
            user_base: "dc=account,dc=ldap".to_owned(),
            role_base: "dc=group,dc=ldap".to_owned(),
            user_classes: ["pilotPerson", "posixAccount", "omemoPerson"]
                .map(str::to_owned)
                .to_vec(),
            group_classes: vec!["posixGroup".to_owned()],
            role_classes: vec!["groupOfUniqueNames".to_owned()],
            role_member_attr: "uniqueMember".to_owned(),
            user_member_attr: "memberOf".to_owned(),
            user_omemo_attr: "omemoBundle".to_owned(),
            uid_next_dn: "cn=uidnext,dc=ldap".to_owned(),
            uid_next_class: "uidNext".to_owned(),
            uid_next_field: "uidNumber".to_owned(),
            gid_next_dn: "cn=gidnext,dc=ldap".to_owned(),
            gid_next_class: "gidNext".to_owned(),
            gid_next_field: "gidNumber".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StalwartConfig {
    pub url: String,
    pub admin: String,
    pub password: String,
}

impl Default for StalwartConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080".to_owned(),
            admin: "admin".to_owned(),
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct SieveConfig {
    pub port: u16,
    pub security: String,
}

impl Default for SieveConfig {
    fn default() -> Self {
        Self {
            port: 4190,
            security: "none".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct ProtocolConfig {
    pub protocol: PersonalDataProtocol,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
#[derive(Default)]
pub struct FilesConfig {}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            protocol: PersonalDataProtocol::Jmap,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PersonalDataProtocol {
    Dav,
    #[default]
    Jmap,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct MailConfig {
    pub server: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub tls: bool,
    pub from: String,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            server: "mail.example.com".to_owned(),
            port: 587,
            user: "user@example.com".to_owned(),
            password: String::new(),
            tls: true,
            from: "no-reply@mail.com".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct JabberConfig {
    pub ws_url: String,
}
