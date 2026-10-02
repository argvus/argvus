---
title: Environment variables
description: Environment contracts used by ARGVUS session components.
---

The session establishes XDG and Wayland environment integration and may export virtualization compatibility variables when detected. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` and `ARGVUS_BOOTSTRAP` are used by session and dispatcher helpers to resolve configuration roots.

Treat environment variables as runtime contracts: verify the installed helper's `--help` or source before adding a persistent override.

Derived state lives under the state root rather than the configuration root.
`XDG_STATE_HOME` (normally `~/.local/state`) receives `argvus/config-projection.json`,
the projection manifest `argvus-config` rebuilds and `argvus-sessionctl reload`
applies, plus `argvus/session.log`. Neither is a source of truth.

## Language environment

The shared `argvus-i18n` implementation selects a locale in this order:

1. `$XDG_CONFIG_HOME/argvus/data/internal/language` (normally `~/.config/argvus/data/internal/language`);
2. `LC_ALL`;
3. `LC_MESSAGES`;
4. `LANG`;
5. the `en-US` fallback.

Values such as `pt_BR.UTF-8` are normalized to `pt-BR`. `LANG`, `LC_ALL` and `LC_MESSAGES` are imported into the user systemd and D-Bus environments by the session controller. A stale user-manager environment can therefore affect applications launched after the Control Center changes the language.

For diagnosis, compare the process locale with the user-manager environment and the system locale configuration:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

If `~/.config/locale.conf` is present and contains conflicting locale assignments, preserve a backup and rename it, then apply the language again in Control Center and start a new graphical session:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

`ARGVUS_CONFIG_HOME` and `ARGVUS_SYSTEM_CONFIG` control configuration roots; they are separate from the language-selection file. `ARGVUS_I18N_DIR` can select an alternate catalog root for development or testing and should not normally be set as a user customization.
