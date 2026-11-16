#[cfg(not(any(feature = "sql", feature = "ldap")))]
compile_error!("enable an auth backend: `sql` or `ldap`");

pub mod config;
pub mod db;
pub mod features;
