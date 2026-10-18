# Phase 3 architecture

Phase 3 introduces three independent crates:

- `argvus-control-center-network`: NetworkManager-oriented models and parsers;
- `argvus-control-center-audio`: PipeWire/WirePlumber `wpctl` backend;
- `argvus-control-center-bluetooth`: BlueZ `bluetoothctl` one-shot backend.

They reuse `argvus-control-center-core::ProcessRunner`, `JobManager`,
`Capabilities`, and terminal sanitization. Their views retain cached snapshots;
collection happens on page entry or with `r`.

## Network

`nmcli -t -f ...` is used with separate arguments and a parser for escaped
fields. Interfaces, Wi-Fi networks, VPN profiles, and connectivity are
normalized in the network backend. Duplicate Wi-Fi SSIDs are consolidated by
the strongest observed signal. DNS is read from `resolvectl` when advertised,
otherwise from `/etc/resolv.conf`; proxy variables are informational and
credentials in proxy URLs are redacted.

NetworkManager actions are not run through `sudo` or a shell. The current
iteration exposes backend methods for connection actions and validates names,
but does not yet expose every mutating action in the TUI. iwd is detected for
future fallback; a complete direct-iwd UI is not claimed here.

## Audio

The preferred backend is `wpctl` when `wpctl`/PipeWire is available. Device
status parsing and volume clamping are backend responsibilities. `pactl` is
detected for a future fallback, but this phase does not claim a complete
PulseAudio UI.

## Bluetooth

BlueZ availability is detected through the existing capability system. The
backend uses bounded, non-interactive `bluetoothctl` invocations for adapter
state, devices, scans, and explicit actions. It never opens an interactive
shell session. A D-Bus pairing Agent for PIN, passkey, and numeric comparison
is not implemented yet; pairing that requires agent interaction is therefore a
known limitation and is not represented as a completed feature.

## Security and refresh

External strings are passed through `terminal_text` before display. Commands
use `Command::new` through the shared runner, with separated arguments and
timeouts; no `sh -c`, `bash -c`, or `sudo` is used. Wi-Fi password input is
masked and kept only in the page state until dismissed. No debug formatting of
the page state is provided.

The current views use manual refresh and refresh-after-entry. D-Bus event
subscriptions are intentionally deferred; there is no aggressive polling loop.

## Optional dependencies

NetworkManager/nmcli, iwd/iwctl, resolvectl, `ip`, WireGuard tools, wpctl,
PipeWire, WirePlumber, pactl, and bluetoothctl are optional. Missing tools
produce unavailable/read-only states rather than panics. No new Rust dependency
was added for this phase.
