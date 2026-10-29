//! GTK 4 pieces shared by ARGVUS Focus and ARGVUS Clock.
//!
//! - [`sound`]: plays the packaged sounds (or a user file) through
//!   `GtkMediaFile`, and reports playback failures instead of hiding them.
//! - [`theme`]: the ARGVUS theme colours (the `time` theme consumer) plus each
//!   application's own stylesheet, reloaded when the theme changes.
//! - [`text`]: translation catalogs that never fail to load, and localized
//!   times and dates built from them.

pub mod sound;
pub mod text;
pub mod theme;
