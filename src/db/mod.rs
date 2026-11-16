#[cfg(feature = "ldap")]
mod ldap;
#[cfg(feature = "sql")]
mod sql;

#[cfg(feature = "ldap")]
pub use ldap::*;
#[cfg(feature = "sql")]
pub use sql::*;

use crate::config::{AuthBackend, Config};

pub struct Connection {
    #[cfg(feature = "sql")]
    pub sql: Option<toasty::Db>,
}
pub async fn connect(config: &Config) -> Result<Connection, String> {
    match config.auth.backend {
        AuthBackend::Sql if !cfg!(feature = "sql") => {
            return Err("SQL authentication requires the `sql` feature".to_owned());
        }
        AuthBackend::Ldap if !cfg!(feature = "ldap") => {
            return Err("LDAP authentication requires the `ldap` feature".to_owned());
        }
        _ => {}
    }

    #[cfg(feature = "sql")]
    let sql = if config.auth.backend == AuthBackend::Sql || cfg!(feature = "kanban") {
        Some(
            sql::connect(config)
                .await
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };

    Ok(Connection {
        #[cfg(feature = "sql")]
        sql,
    })
}
