---
title: ARGVUS Internationalisierung
description: Sprachkataloge, Auswahl und Unicode-Unterstützung.
---
# ARGVUS Internationalisierung

ARGVUS liefert Englisch (`en-US`), Brasilianisches Portugiesisch (`pt-BR`), Spanisch (`es-ES`), Französisch (`fr-FR`), Italienisch (`it-IT`), Deutsch (`de-DE`), Russisch (`ru-RU`), Vereinfachtes Chinesisch (`zh-CN`), Japanisch (`ja-JP`) und Koreanisch (`ko-KR`). Englisch ist der kanonische Katalog und die Fallback-Sprache. Die acht zusätzlichen Übersetzungen wurden mit maschineller Übersetzungshilfe und technischen Korrekturen erstellt; eine unabhängige Überprüfung durch Muttersprachler wurde nicht zertifiziert.

## Sprache auswählen

Öffnen Sie das Kontrollzentrum > Einstellungen > Sprache & Region > Sprache oder Einstellungen > Sprache. Die Seite erkennt installierte Kataloge und zeigt deren native Namen an. Sie speichert `LANG` in `$XDG_CONFIG_HOME/locale.conf` (Standard `~/.config/locale.conf`). Aktivieren Sie zuerst die entsprechende UTF-8-Systemgebietsschema in den Systemgebietsschemata. Beispiele sind `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` und `ko_KR.UTF-8`.

Die Einstellung ändert `/etc/locale.conf` nicht. Kommentare und regionale Kategorieseinstellungen werden beibehalten. Das Bedienfeld wird nach dem Speichern neu gestartet; melden Sie sich ab und wieder an, um die Sprache für die gesamte Sitzung anzuwenden. Nicht konfigurierte Benutzer verwenden Englisch. Vererbte Umgebungsvariablen und ältere ARGVUS-Sprachdateien ersetzen diese Einstellung nicht. Der Login-Greeter wird ausgeführt, bevor die Benutzereinstellung verfügbar ist, und behält seine eigene Dienstkonto-Sprache bei, standardmäßig Englisch.

Regionale Varianten verwenden den installierten Katalog für ihre Sprache, wenn ein exakter Katalog nicht vorhanden ist; zum Beispiel verwendet `es_MX.UTF-8` `es-ES`. `zh-Hans` wählt vereinfachtes Chinesisch. Traditionelles Chinesisch (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) wird nicht stillschweigend auf vereinfachtes Chinesisch abgebildet und fällt auf Englisch zurück.

## Kataloge und Beiträge

Die Quelle ist `resources/locales/<locale>/<domain>.json` im `argvus`-Repository. Installierte Kataloge befinden sich in `/usr/share/argvus/i18n/`. Sie werden im Master-`argvus`-Paket geliefert. `locale.json` stellt den Bezeichner sowie die nativen/Anzeigenamen bereit. Es gibt keinen separaten PO-Katalog oder eine erforderliche Online-Übersetzungsplattform.

Verwenden Sie semantische Schlüssel und bewahren Sie Platzhalternamen, Befehle, Optionen, Pfade und Tastenkombinationen. Übersetzen Sie komplette Meldungen, anstatt Sätze aus übersetzten Fragmenten zusammenzusetzen. Neue kanonische Schlüssel benötigen Übersetzungen in allen zehn Sprachen. Halten Sie die Terminologie innerhalb jedes Bereichs konsistent und überprüfen Sie Übersetzungen im Kontext, insbesondere bei Authentifizierung, Berechtigungen und destruktiven Aktionen.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```
`ARGVUS_I18N_DIR` wählt einen alternativen Katalogstamm für Entwicklungs- und gestufte Paketprüfungen aus. Die Validierungsprüfung überprüft Domains, Schlüssel, Zeichenfolgenwerte, Platzhalter, leere Übersetzungen und lokalspezifische ganzzahlige Pluralformen. Die Validierung belegt die strukturelle Integrität, nicht die sprachliche Genauigkeit.

## Unicode und Schriftarten

`argvus-fonts` benötigt Noto CJK Schriftarten zusätzlich zu den lateinischen, Symbol- und Emoji-Familien. Fontconfig stellt eine Fallback-Lösung für Chinesisch, Japanisch und Hangul bereit, ohne die vom Benutzer gewählte primäre Schriftart zu ersetzen. Anwendungen und Terminals müssen weiterhin die Schriftart-Fallback-Funktion aktiviert haben.

Gemeinsame Ratatui-Texthelfer messen Terminalzellen, bewahren erweiterte Graphemcluster während der Kürzung und umbrechen breiten Text. Die Texteditoren verschieben und löschen ganze Grapheme. Extrem schmale Bereiche können ein Zwei-Zellen-Glyphe nicht in einer Zelle darstellen; der Hard-Wrap-Helfer verwendet in diesem Fall ein Ersatzzeichen. Emoji und Zeichen mit uneindeutiger Breite können je nach Terminal-/Schriftartenimplementierung weiterhin unterschiedlich sein. Die Breitenrichtlinie des Terminals sollte mit der normalen Unicode-Breite von Ratatui übereinstimmen.

Die Sprachauswahl installiert keine Eingabemethode und ändert nicht das Tastaturlayout. Verwenden Sie eine Wayland-kompatible Eingabemethode, wenn Sie Chinesisch, Japanisch oder Koreanisch eingeben.
