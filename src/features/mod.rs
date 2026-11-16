//! Application features. Each module holds everything one feature owns, such as its
//! data and business rules, for the pages in `app` to use.

pub mod auth;
#[cfg(feature = "files")]
pub mod files;
#[cfg(feature = "calendar")]
pub mod calendar;
#[cfg(feature = "contacts")]
pub mod contacts;
pub mod group;
#[cfg(feature = "jabber")]
pub mod jabber;
#[cfg(feature = "kanban")]
pub mod kanban;
#[cfg(feature = "mail")]
pub mod mail;
pub mod role;
#[cfg(any(feature = "webmail", feature = "calendar", feature = "contacts", feature = "files"))]
pub mod stalwart;
pub mod theme;
pub mod user;
#[cfg(feature = "webmail")]
pub mod webmail;
