//! Rules of the ARGVUS network and Bluetooth commands: how the output of nmcli,
//! ip, iwgetid and bluetoothctl is read, which status a state means, and the
//! Waybar text of the Bluetooth module. Nothing here starts a process.

pub mod bluetooth;
pub mod nmcli;
pub mod status;
