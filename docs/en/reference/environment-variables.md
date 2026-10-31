---
title: Environment variables
description: Environment contracts used by ARGVUS session components.
---

The session establishes XDG and Wayland environment integration and may export virtualization compatibility variables when detected. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` and `ARGVUS_BOOTSTRAP` are used by session and dispatcher helpers to resolve configuration roots.

Treat environment variables as runtime contracts: verify the installed helper's `--help` or source before adding a persistent override.

Derived state lives under the state root rather than the configuration root.
`XDG_STATE_HOME` (normally `~/.local/state`) receives `argvus/config-projection.json`,
the projection manifest `argvus-config` rebuilds and `argvus sessionctl reload`
applies, plus `argvus/session.log`. Neither is a source of truth.

## Language environment

The only general language preference is `LANG` in `$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`). Without a valid value, ARGVUS selects English (`en-US`), even when `/etc/locale.conf`, PAM or the login manager supplies Portuguese. Values such as `pt_BR.UTF-8` normalize to `pt-BR` for catalogs.

The session applies this preference before starting desktop components and importing the systemd user and D-Bus activation environments. It replaces inherited `LANG`, `LC_MESSAGES`, `LANGUAGE` and clears `LC_ALL`; regional `LC_*` settings in the same file are preserved. English uses `en_US.UTF-8` when generated, otherwise `C.UTF-8`. No preference file is created automatically. Log out and back in after changing the language.

`session.language`, `argvus/data/language` and `argvus/data/internal/language` are ignored legacy data. They are neither imported nor regenerated; existing files are preserved. `argvus config set` and `patch` reject new writes to `session.language` and direct callers to `locale.conf`.

For diagnosis, compare `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` and `systemctl --user show-environment`. `localectl status` describes the system, not the user's ARGVUS language. `ARGVUS_I18N_DIR` selects a catalog directory for development, not the language.
