---
title: ARGVUS Rechner
description: Einfach · Wissenschaftlich · Programmierung · Finanzen · Einheitenumrechnung
---

# ARGVUS Rechner

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Wählen Sie ein Profil unter Einstellungen → Profil mit ↑/↓ und Enter oder per Klick. Profil und Optionen werden sofort gespeichert und beim nächsten Start wiederhergestellt. Jedes Profil behält während der Sitzung seinen Ausdruck. Enter oder = ersetzt den Ausdruck durch das Ergebnis ohne Auswahl; der Cursor bleibt am Ende.

Zinssätze sind Prozent pro Periode; verwenden Sie dieselbe Periode für Satz, Anzahl und Zahlung. Zahlungen erfolgen am Periodenende. Nullzinsen werden unterstützt. Einfache Zinsen und Zinseszinsen liefern den Gesamtbetrag. Umrechnung erfolgt offline ohne Währungen. Der Verlauf speichert bis zu 100 Einträge samt Profil, Optionen und Finanzfeldern; die Auswahl stellt den Kontext wieder her. Löschen leert auch die gespeicherte Datei.

## Einstellungen

- `s Einstellungen` → **Profil** → Einfach / Wissenschaftlich / Programmierung / Finanzen / Einheitenumrechnung.
- ↑ / ↓ · Enter · Escape / ←.
- **Ergebnis kopieren** · **Verlauf** · **Verlauf löschen**.
- **Einfach**: 370 × 420; **Wissenschaftlich**: 440 × 640.
- **Programmierung**: 520 × 640; **Finanzen**: 440 × 640; **Einheitenumrechnung**: 440 × 520.

## Einfach / Wissenschaftlich

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Winkeleinheit**: Grad / Bogenmaß. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 signifikante Stellen · 4096 Bytes · 64 Parser-Ebenen.

## Programmierung

Standard: dezimal, 64 Bit mit Vorzeichen. Überläufe werden auf die Wortbreite begrenzt; Rechtsverschiebung mit Vorzeichen erweitert das Vorzeichen. Ganzzahldivision rundet gegen null. Division durch null und Verschiebungen außerhalb [0, bits) sind Fehler. Ohne Präfix gilt die gewählte Basis. Ein Basiswechsel konvertiert den gültigen Ausdruck; Breite oder Vorzeichen interpretieren dieselben Bits neu.

- **Basis**: 2 / 8 / 10 / 16; **Wortbreite (Bits)**: 8 / 16 / 32 / 64.
- **Mit Vorzeichen** / **Ohne Vorzeichen** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Mit Vorzeichen); `-7/2 = -3`.

## Finanzen

Einfache Zinsen · Zinseszinsen · Feste Rate · Barwert · Endwert · Prozentuale Änderung.

- `P`: Anfangskapital; `FV`: Zukünftiger Betrag; `A`: Zahlung pro Periode.
- `r`: Zinssatz pro Periode (%) / 100; `n`: Anzahl der Perioden.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Einfache Zinsen: `P*(1+r*n)`; Zinseszinsen: `P*G`.
- Feste Rate: `P*G/H`; Barwert: `(FV+A*H)/G`; Endwert: `P*G+A*H`.
- Prozentuale Änderung: `(Endbetrag - Ausgangsbetrag) / Ausgangsbetrag * 100`.

## Einheitenumrechnung

Länge · Masse · Temperatur · Fläche · Volumen · Zeit · Geschwindigkeit · Datenspeicher.

**Von** / **Nach**: Einstellungen · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Verlauf

100 · Profil + `context` · `Verlauf löschen`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Anwendungstexte folgen Applications Font unter Control Center → Appearance → Desktop Style → Fonts. GTK-Anwendungen erben gtk-font-name; Größen für Zahlen und Überschriften bleiben erhalten. Der Launcher liest das erzeugte apps_family und verfolgt Änderungen.

Die Kopfzeile kombiniert das ARGVUS-Abzeichen in der Akzentfarbe mit dem Anwendungsnamen. Die anklickbare Fußzeile s Settings liegt auf einem durchgehenden Streifen mit eigener Themenfläche im Stil des Control Centers.
