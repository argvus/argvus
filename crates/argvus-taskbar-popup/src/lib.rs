//! Popups that open next to the ARGVUS taskbar.
//!
//! The taskbar calendar, ARGVUS Focus and ARGVUS Clock share this crate so they
//! open at the same place and close the same way:
//!
//! - [`geometry`]: where the popup goes. The click position comes from the
//!   Waybar launcher (immutable root coordinates); the bar geometry and the
//!   monitor come from `hyprctl`. The popup sits [`geometry::POPUP_GAP_Y`]
//!   pixels below a top bar or above a bottom bar, aligned to the bar's right
//!   edge.
//! - [`controller`]: the layer-shell surface (namespace, overlay layer,
//!   keyboard mode), focus-loss dismissal and the guard against a click that
//!   closes the popup from reopening it.

pub mod controller;
pub mod geometry;

pub use controller::PopupController;
pub use geometry::{PopupAnchor, PopupBounds, PopupSize};
