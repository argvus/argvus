//! Rules of the telemetry widget: the blocks and their order, the blocks file,
//! the edit of the Waybar JSONC that shows or hides each block, and the window
//! of blocks that fits the scroll budget. Nothing here runs a process; the
//! binary reads and writes the files and talks to systemd and Hyprland.

pub mod blocks;
pub mod cpu;
pub mod gpu;
pub mod jsonc;
pub mod machine;
pub mod memory;
pub mod network;
pub mod output;
pub mod ports;
pub mod processes;
pub mod report;
pub mod scroll;
pub mod sensors;
pub mod storage;
