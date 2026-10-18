//! Rules of the TUI terminal launcher: which font profile a window uses, the
//! font values it takes from the generated fonts file, the background opacity
//! of the Control Center, and the Kitty configuration that the launcher writes.
//! Nothing here runs a process or reads a file; the binary does that.

pub mod font;
pub mod kitty;
pub mod opacity;
pub mod profile;
