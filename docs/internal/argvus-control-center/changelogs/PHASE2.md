# Phase 2 architecture

## Hardware

`argvus-control-center-hardware` owns hardware models, probes, cached state and
the TUI view. Collection runs through `JobManager`; no `/proc`, `/sys`, DMI,
DRM, `lspci`, OpenGL or Vulkan probe runs during a frame. Sysfs is the primary
source for CPU frequency, DRM devices and batteries. `lspci`, `glxinfo` and
`vulkaninfo` are optional fallbacks/complements detected by `Capabilities`.

Power profiles are read from `powerprofilesctl` when available, with TLP and
cpupower reported as alternatives. Governor writes are restricted to known
governor values and are routed through the existing validated
`system-settings governor set` boundary. The UI does not change graphical
configuration for virtual machines or `vmwgfx`; it only reports the condition.

## Services and journal

`argvus-control-center-services` owns unit and journal models, command parsing,
actions, filtering and views. System and user lists are requested separately
with structured `systemctl` arguments. Unit actions are represented as an
allowlisted action plus a validated `.service` name; system actions use the
existing polkit/pkexec path and user actions use `systemctl --user`.

Journal entries are parsed as JSON, tolerate missing fields, and pass message
and unit text through `argvus-control-center-core::sanitize::terminal_text`.
The viewer loads a bounded set of entries asynchronously. `b` toggles current
and previous boot; `0` through `7` apply a journal priority filter. Opening
`Logs` from a service detail applies `-u` for that unit. Follow mode remains a
future extension because it requires a streaming job contract.

## Optional dependencies and limitations

Missing tools result in unavailable data rather than a crash. Phase 2 does not
implement NetworkManager, audio, Bluetooth, boot, package, storage or complete
diagnostic domains. It also does not expose destructive disk actions, service
masking, or graphical configuration changes.
