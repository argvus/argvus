//! Single vertical menu list shared by every Control Center page.
//!
//! A page builds a `Vec<Row<Id>>` each frame from its own state, keeps one
//! [`MenuState`], forwards keys to [`MenuState::handle`] and reacts to the
//! returned [`MenuEvent`]. Actions that used to be buttons are rows too, so
//! there is no separate button focus.

mod render;
mod row;
mod state;

pub use render::{MenuStyle, draw_menu};
pub use row::{Emphasis, Row, RowKind, draft_actions};
pub use state::{MenuEvent, MenuState};
