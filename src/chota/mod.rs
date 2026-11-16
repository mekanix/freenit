//! [Chota](https://github.com/jenil/chota) support for Topcoat.
//!
//! Use [`stylesheet!`](stylesheet) to bundle Chota's minified stylesheet as a
//! Topcoat asset.

pub mod components;

mod stylesheet;

pub use crate::stylesheet;
