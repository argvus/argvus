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

`ARGVUS_LANGUAGE` in `$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`) selects the ARGVUS interface language from its installed catalogs. Without it, ARGVUS resolves the regional `LANG` value to a catalog and falls back to English (`en-US`). Values such as `pt_BR.UTF-8` resolve to Brazilian Portuguese (`pt-BR`); `pt-PT` explicitly selects European Portuguese.

The session keeps `LANG` as the regional locale for formatting and third-party applications. `ARGVUS_LANGUAGE` does not need a generated system locale and does not replace `LANG`. The Control Panel restarts after the interface language changes; log out and back in to apply it throughout session components. The separate Regional Locale setting changes `LANG` and requires the selected UTF-8 system locale to be generated. No preference file is created automatically.

`session.language`, `argvus/data/language` and `argvus/data/internal/language` are ignored legacy data. They are neither imported nor regenerated; existing files are preserved. `argvus config set` and `patch` reject new writes to `session.language` and direct callers to `locale.conf`.

For diagnosis, compare `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` and `systemctl --user show-environment`. `localectl status` describes the system, not the user's ARGVUS interface language. `ARGVUS_I18N_DIR` selects a catalog directory for development, not the language.
