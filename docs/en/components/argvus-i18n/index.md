---
title: ARGVUS Internationalization
description: Multi-language support system for ARGVUS
---

# ARGVUS Internationalization (i18n)

The ARGVUS Internationalization module provides language support across all ARGVUS components and applications, enabling a truly multilingual desktop experience.

## Supported Languages

Currently supported languages include:
- English (en)
- Portuguese - Brazil (pt-BR)
- Additional languages coming soon

## Language Selection

Choose the user language in Control Center > Preferences > Language & Region > Language, or Settings > Language. Both save `LANG` in `$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`). Without a valid preference ARGVUS uses English. System locale variables and legacy ARGVUS language files do not override this file. Log out and back in after changing the language.

## For Contributors

To contribute translations or add new language support:

1. Visit the translation platform
2. Join as a translator
3. Submit translations for review
4. Translations are merged in each release

## Translation Files

Translation files use standard `.po` (Portable Object) format and are managed through:
- `po/` directory in ARGVUS repositories
- Translation memory system for consistency

## Localization

Beyond language, localization includes:
- Date and time formats
- Number formatting
- Currency display
- Keyboard layouts

For customization, check ARGVUS system localization settings.
