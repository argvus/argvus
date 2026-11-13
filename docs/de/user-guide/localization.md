---
title: Lokalisierung
description: Verwenden Sie ARGVUS auf Englisch oder Portugiesisch.
---

`argvus-i18n` stellt die freigegebenen Kataloge bereit, die `argvus i18n validate` Befehl und die QML Modul `org.argvus.i18n`. ARGVUS Schiffe 18 Schnittstellensprachen: Arabisch, Bengalisch, Deutsch, Englisch, Spanisch, Französisch, Hindi, Indonesisch, Italienisch, Japanisch, Koreanisch, Brasilianisches Portugiesisch, Europäisches Portugiesisch, Russisch, Türkisch, Urdu, Vietnamesisch und Vereinfachtes Chinesisch.

Eingeboren ARGVUS Anwendungen und Shell-Komponenten verbrauchen die freigegebenen Kataloge. Ein übersetzter String kann daher von mehr als einem Repository verwendet werden. Arabisch und Urdu verwenden rechts nach links Layout in der unterstützten GTK, Quickshell und Terminal-Schnittstellen.

Verwenden Sie ** Control Center → Präferenzen → Sprache & Region → Sprache**, um die ARGVUS Schnittstellensprache. Dies schreibt `ARGVUS_LANGUAGE` bis `$XDG_CONFIG_HOME/locale.conf` (normalerweise) `~/.config/locale.conf`. Zum Beispiel: `ARGVUS_LANGUAGE=pt-PT` Wählen Sie europäisches Portugiesisch. Brasilianisches Portugiesisch`pt-BR`) ist eine separate Option. Der Sprachkatalog wird installiert mit ARGVUSEs hängt also nicht von `locale-gen`.

Die Systemsteuerung wird automatisch neu gestartet, nachdem sich die Schnittstellensprache geändert hat. Starten Sie eine neue Sitzung, um sie auf Sitzungskomponenten anzuwenden. Die separate **Regional Locale** Einstellung schreibt `LANG` und kontrolliert regionale Formate und Drittanwendungen. Es erfordert ein generiertes System locale; ermöglicht eine in `/etc/locale.gen` Läufe `locale-gen` mit Administratorautorisierung. Tastaturlayout, Variante und Konsolen-Keymap sind separate Einstellungen.

## Wenn Umgebungsvariablen die Änderung außer Kraft setzen

ARGVUS Anträge lesen `ARGVUS_LANGUAGE` vom Benutzer `locale.conf`Wenn sie fehlt, lösen sie die regionale `LANG` Wert für einen installierten Katalog. Vererbte Locale-Variablen überschreiben keine explizite `ARGVUS_LANGUAGE` Präferenz. Anwendungen von Drittanbietern verwenden weiterhin das regionale System.

Prüfen Sie die effektiven Werte mit:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Prüfen Sie: `~/.config/locale.conf` enthält eine gültige `ARGVUS_LANGUAGE=<catalog>` Wert. Um die regionale Formatierung oder die Anwendungssprache eines Drittanbieters zu ändern, verwenden Sie **Regional Locale** und wählen Sie ein generiertes Locale aus. Die Systemsteuerung wird nach den Einstellungen neu gestartet; melden Sie sich aus und in, wenn andere Sitzungskomponenten noch die alte Schnittstellensprache anzeigen. Siehe [Umweltvariablen](/docs/reference/environment-variables/) für die technischen Verträge und Vorrang.
