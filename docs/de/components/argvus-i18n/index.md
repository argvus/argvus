---
title: ARGVUS Internationalisierung
description: Sprachkataloge, Auswahl und Unicode-Unterstützung.
---

# ARGVUS Internationalisierung

ARGVUS Schiffe 18 Schnittstellensprachen: Arabisch`ar`), Bengalisch (`bn-BD`), Deutsch (`de-DE`,
Englisch ()`en-US`), Spanisch (`es-ES`), Französisch (`fr-FR`), Hindi (`hi-IN`), indonesisch
()`id-ID`), italienisch (`it-IT`), Japanisch (`ja-JP`), Koreanisch (`ko-KR`), Brasilianisches Portugiesisch
()`pt-BR`), Europäisches Portugiesisch (`pt-PT`), Russisch (`ru-RU`), Türkisch (`tr-TR`), Urdu
()`ur-PK`), Vietnamesisch (`vi-VN`) und vereinfachtes Chinesisch (`zh-CN`. Englisch ist die
kanonischer Katalog und Fallback. Die Übersetzungen wurden erstellt mit AI Unterstützung und
technische Korrekturen; unabhängige muttersprachliche Überprüfung wurde nicht zertifiziert.

## Sprache auswählen

Open Control Center > Präferenzen > Sprache & Region > Sprache. Die Seite entdeckt
installiert Kataloge und zeigt ihre nativen Namen. Es spart `ARGVUS_LANGUAGE` in
`$XDG_CONFIG_HOME/locale.conf` (Standard) `~/.config/locale.conf`, unabhängig von
`LANG`Erzeugen eines Systemlocals ist nicht erforderlich. Brasilianisches Portugiesisch`pt-BR`) und
Europäisches Portugiesisch`pt-PT`) sind getrennte Entscheidungen.
Die Präferenz ändert sich nicht `/etc/locale.conf` oder die regionale `LANG` Einstellung.
Kommentare und regionale Kategorieeinstellungen bleiben erhalten. Das Control Panel wird neu gestartet
nach dem Speichern; Starten einer neuen Sitzung, um die Sprache in allen Sitzungskomponenten anzuwenden.
Unkonfigurierte Benutzer verwenden Englisch. Die separaten Regional Locale Präferenzkontrollen
`LANG`, regionale Formatierung und Anwendungen von Drittanbietern. Der Login Greet läuft
bevor die Präferenz des Benutzers verfügbar ist und ein eigenes Service-Konto behält
Sprache, Englisch standardmäßig.
Regionale Varianten verwenden den installierten Katalog für ihre Sprache, wenn eine genaue
Katalog fehlt; zum Beispiel `es_MX.UTF-8` Verwendung `es-ES`. `zh-Hans` auswählt
Vereinfachtes Chinesisch. Traditionelles Chinesisch`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) ist
nicht still auf vereinfachtes Chinesisch abgebildet und fällt auf Englisch zurück.

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
