---
title: ARGVUS Internationalization
description: Language catalogs, selection and Unicode support.
---

# ARGVUS Internationalization

ARGVUS ships 18 interface languages: Arabic (`ar`), Bengali (`bn-BD`), German (`de-DE`),
English (`en-US`), Spanish (`es-ES`), French (`fr-FR`), Hindi (`hi-IN`), Indonesian
(`id-ID`), Italian (`it-IT`), Japanese (`ja-JP`), Korean (`ko-KR`), Brazilian Portuguese
(`pt-BR`), European Portuguese (`pt-PT`), Russian (`ru-RU`), Turkish (`tr-TR`), Urdu
(`ur-PK`), Vietnamese (`vi-VN`) and Simplified Chinese (`zh-CN`). English is the
canonical catalog and fallback. The translations were produced with AI assistance and
technical corrections; independent native-speaker review has not been certified.

## Select a language

Open Control Center > Preferences > Language & Region > Language. The page discovers
installed catalogs and shows their native names. It saves `ARGVUS_LANGUAGE` in
`$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`), independently of
`LANG`; generating a system locale is not required. Brazilian Portuguese (`pt-BR`) and
European Portuguese (`pt-PT`) are separate choices.

The preference does not modify `/etc/locale.conf` or the regional `LANG` setting.
Comments and regional category settings are preserved. The Control Panel is restarted
after saving; start a new session to apply the language throughout session components.
Unconfigured users use English. The separate Regional Locale preference controls
`LANG`, regional formatting, and third-party applications. The login greeter runs
before the user's preference is available and retains its own service-account
language, English by default.

Regional variants use the installed catalog for their language when an exact
catalog is absent; for example `es_MX.UTF-8` uses `es-ES`. `zh-Hans` selects
Simplified Chinese. Traditional Chinese (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) is
not silently mapped to Simplified Chinese and falls back to English.

## Catalogs and contributions

The source is `resources/locales/<locale>/<domain>.json` in the `argvus` repository.
Installed catalogs live in `/usr/share/argvus/i18n/`. They ship in the master
`argvus` package. `locale.json` supplies the identifier and native/display names.
There is no separate PO catalog or required online translation platform.

Use semantic keys and preserve placeholder names, commands, options, paths and
keyboard shortcuts. Translate complete messages instead of assembling sentences
from translated fragments. New canonical keys need translations in all ten
languages. Keep terminology consistent within each domain and review translations
in context, especially authentication, permissions and destructive actions.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR` selects an alternative catalog root for development and staged
package checks. Validation checks domains, keys, string values, placeholders,
empty translations and locale-specific integer plural forms. Validation proves
structural integrity, not linguistic accuracy.

## Unicode and fonts

`argvus-fonts` requires Noto CJK fonts in addition to the Latin, symbol and emoji
families. Fontconfig provides fallback for Chinese, Japanese and Hangul without
replacing the user's chosen primary family. Applications and terminals still
need font fallback enabled.

Shared Ratatui text helpers measure terminal cells, preserve extended grapheme
clusters during truncation and wrap wide text. The text editors move and delete
whole graphemes. Extremely narrow areas cannot display a two-cell glyph in one
cell; the hard-wrap helper uses a replacement character in that case. Emoji and
ambiguous-width characters can still differ between terminal/font implementations.
The terminal's width policy should agree with Ratatui's normal Unicode width.

Language selection does not install an input method or change the keyboard layout.
Use a Wayland-compatible input method when entering Chinese, Japanese or Korean.
