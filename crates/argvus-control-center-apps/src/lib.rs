#![allow(dead_code)]
//! Root module of crate `argvus control center apps`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.

pub mod apply;
pub mod catalog;
pub mod cli;
pub mod detect;
pub mod state;
