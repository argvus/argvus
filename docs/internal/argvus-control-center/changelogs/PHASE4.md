# Phase 4 architecture

`argvus-control-center-boot` owns boot inspection models, parsers and the
background-loaded view. It currently reads boot state without changing `/boot`
or `/etc`: firmware mode comes from the existing capability detector; systemd-
boot evidence comes from loader files and entries; GRUB evidence comes from
its configuration paths; kernels come from `/usr/lib/modules`; mkinitcpio and
Plymouth are inspected only when their optional files/tools exist.

The parsers never source or execute configuration files. Unknown loader keys,
GRUB keys and entry directives are ignored by the read-only model rather than
being interpreted as shell. All displayed external text is sanitized before it
reaches the TUI.

The current implementation exposes Summary, Kernel, Bootloader, Initramfs and
Plymouth routes and refreshes them through the shared `JobManager`. It supports
UEFI/Legacy reporting, conservative systemd-boot/GRUB detection, kernel image
and initramfs association, mkinitcpio fields, Plymouth theme discovery and
best-effort Secure Boot detection.

Changing boot defaults, GRUB values, timeout, initramfs or Plymouth is not yet
exposed as an action. Consequently no privileged boot helper or backup writer
was added prematurely. Future mutating work must add typed actions to the
existing privilege boundary, validate fixed target paths, create non-overwriting
backups, write atomically and model partial Plymouth/initramfs failure.

Known limitations are UKI/XBOOTLDR discovery beyond the conservative paths,
robust GRUB menu-entry parsing, package-derived kernel versions, and all
privileged mutations. These are intentionally visible limitations rather than
fake controls.
