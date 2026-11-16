use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, dn_escape};
use topcoat::context::{Cx, context};

pub use crate::config::LdapConfig;
use crate::config::Config;

impl LdapConfig {
    pub async fn connect_unbound(&self) -> ldap3::result::Result<Ldap> {
        let settings = LdapConnSettings::new().set_starttls(self.starttls);
        let (conn, ldap) = LdapConnAsync::with_settings(settings, &self.url).await?;
        ldap3::drive!(conn);
        Ok(ldap)
    }

    pub async fn connect(&self) -> ldap3::result::Result<Ldap> {
        let mut ldap = self.connect_unbound().await?;
        ldap.simple_bind(&self.service_dn, &self.service_password)
            .await?
            .success()?;
        Ok(ldap)
    }

    pub fn user_dn(&self, email: &str) -> String {
        let (username, domain) = email.split_once('@').unwrap_or((email, ""));
        format!(
            "uid={},ou={},{}",
            dn_escape(username),
            dn_escape(domain),
            self.user_base
        )
    }

    pub fn group_dn(&self, uid: &str, email: &str) -> String {
        let domain = email.split_once('@').map_or("", |(_, domain)| domain);
        format!(
            "cn={},ou={},{}",
            dn_escape(uid),
            dn_escape(domain),
            self.role_base
        )
    }

    pub fn role_dn(&self, name: &str) -> String {
        format!("cn={},{}", dn_escape(name), self.role_base)
    }
}

pub fn ldap_config(cx: &Cx) -> LdapConfig {
    context::<Config>(cx).ldap.clone()
}
