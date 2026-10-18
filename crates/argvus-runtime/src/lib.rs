//! What the ARGVUS runtime commands share: the translated messages of
//! argvus-i18n and the programs they run. The rules of each command live in its
//! core crate; this crate is the only place that starts processes for them.

pub mod messages;
pub mod process;
