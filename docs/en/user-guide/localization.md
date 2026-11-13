---
title: Localization
description: Use ARGVUS in English or Portuguese.
---

`argvus-i18n` provides the shared catalogs, the `argvus i18n validate` command and the QML module `org.argvus.i18n`. ARGVUS ships 18 interface languages: Arabic, Bengali, German, English, Spanish, French, Hindi, Indonesian, Italian, Japanese, Korean, Brazilian Portuguese, European Portuguese, Russian, Turkish, Urdu, Vietnamese and Simplified Chinese.

Native ARGVUS applications and shell components consume the shared catalogs. A translated string may therefore be used by more than one repository. Arabic and Urdu use right-to-left layout in the supported GTK, Quickshell and terminal interfaces.

Use **Control Center → Preferences → Language & Region → Language** to select the ARGVUS interface language. This writes `ARGVUS_LANGUAGE` to `$XDG_CONFIG_HOME/locale.conf` (normally `~/.config/locale.conf`). For example, `ARGVUS_LANGUAGE=pt-PT` selects European Portuguese. Brazilian Portuguese (`pt-BR`) is a separate option. The language catalog is installed with ARGVUS, so it does not depend on `locale-gen`.

The Control Panel restarts automatically after the interface language changes. Start a new session to apply it to session components. The separate **Regional Locale** setting writes `LANG` and controls regional formats and third-party applications. It requires a generated system locale; enabling one in `/etc/locale.gen` runs `locale-gen` with administrator authorization. Keyboard layout, variant and console keymap are separate settings.

## When environment variables override the change

ARGVUS applications read `ARGVUS_LANGUAGE` from the user's `locale.conf`. If it is absent, they resolve the regional `LANG` value to an installed catalog. Inherited locale variables do not override an explicit `ARGVUS_LANGUAGE` preference. Third-party applications continue to use the system regional locale.

Inspect the effective values with:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Check that `~/.config/locale.conf` contains a valid `ARGVUS_LANGUAGE=<catalog>` value. To change regional formatting or third-party application language, use **Regional Locale** and select a generated locale. The Control Panel is restarted after either preference changes; log out and in if other session components still show the old interface language. See [Environment variables](/docs/reference/environment-variables/) for the technical contracts and precedence.
