---
title: Localization
description: Use ARGVUS in English or Portuguese.
---

`argvus-i18n` provides the shared catalogs, the `argvus-i18n validate` command and the QML module `org.argvus.i18n`. ARGVUS currently ships English and Portuguese catalogs.

Native applications and shell components consume the shared catalogs. A translated string may therefore be used by more than one repository.

Use **Control Center → Locale & Region** to configure the language, regional locale, generated system locales, time zone, date/time and keyboard. The current ARGVUS catalogs provide English and Portuguese; this is separate from the list of system locales that the operating system has generated.

Changing the display language saves the ARGVUS preference and automatically restarts `argvus-control-panel.service`, so the Control Panel reflects the change immediately. Other applications and shell surfaces may require a session reload or a new session. Generating or changing `/etc/locale.gen` entries requires authorization and runs `locale-gen`; selecting a locale that is not generated is rejected. Keyboard layout, variant and console keymap are separate settings and should not be confused with the interface language.

## When environment variables override the change

ARGVUS applications detect the language from the persistent ARGVUS language file first, then `LC_ALL`, `LC_MESSAGES` and `LANG`. Local environment variables can therefore make the Control Center appear to ignore a language change, especially when a user service or graphical session retains an older environment.

Inspect the effective values with:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

If `~/.config/locale.conf` exists and defines `LANG`, `LC_ALL` or `LC_MESSAGES`, rename it temporarily so it no longer reintroduces those values. Keep a backup instead of deleting it:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

Apply the language again in **Control Center → Locale & Region**. The Control Panel is restarted automatically; if other components still show the previous language, log out and in so the graphical session and `systemd --user` receive the new environment. Repeat the diagnostic commands afterwards. See [Environment variables](../../reference/environment-variables/) for the technical contracts and precedence.
