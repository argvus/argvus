---
title: Umgebungsvariablen
description: Umweltverträge, die von ARGVUS-Sitzungskomponenten verwendet werden.
---
Die Sitzung stellt die Integration der XDG- und Wayland-Umgebung her und kann, wenn erkannt, Variablen zur Virtualisierungskompatibilität exportieren. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` und `ARGVUS_BOOTSTRAP` werden von Sitzungs- und Dispatcher-Hilfsprogrammen verwendet, um Konfigurationswurzeln aufzulösen.

Behandeln Sie Umgebungsvariablen als Laufzeitvereinbarungen: Überprüfen Sie das `--help` des installierten Hilfsprogramms oder die Quelle, bevor Sie eine dauerhafte Überschreibung hinzufügen.

Abgeleiteter Zustand befindet sich unter der Zustandswurzel und nicht unter der Konfigurationswurzel. `XDG_STATE_HOME` (normalerweise `~/.local/state`) empfängt `argvus/config-projection.json`, das Projektionsmanifest `argvus-config` wird neu erstellt und `argvus sessionctl reload` angewendet, plus `argvus/session.log`. Keines davon ist eine Quelle der Wahrheit.

## Sprachumgebung

Die einzige allgemeine Sprachpräferenz ist `LANG` in `$XDG_CONFIG_HOME/locale.conf` (Standard `~/.config/locale.conf`). Ohne einen gültigen Wert wählt ARGVUS Englisch (`en-US`), selbst wenn `/etc/locale.conf`, PAM oder der Login-Manager Portugiesisch bereitstellen. Werte wie `pt_BR.UTF-8` normalisieren sich zu `pt-BR` für Kataloge.

Die Sitzung wendet diese Präferenz an, bevor Desktop-Komponenten gestartet und die systemd-Benutzer- sowie D-Bus-Aktivierungsumgebungen importiert werden. Sie ersetzt vererbte `LANG`, `LC_MESSAGES`, `LANGUAGE` und löscht `LC_ALL`; regionale `LC_*`-Einstellungen in derselben Datei bleiben erhalten. Englisch verwendet `en_US.UTF-8`, wenn es generiert wird, andernfalls `C.UTF-8`. Es wird keine Präferenzdatei automatisch erstellt. Nach dem Ändern der Sprache abmelden und erneut anmelden.

`session.language`, `argvus/data/language` und `argvus/data/internal/language` sind ignorierte alte Daten. Sie werden weder importiert noch neu generiert; vorhandene Dateien bleiben erhalten. `argvus config set` und `patch` lehnen neue Schreibvorgänge an `session.language` ab und verweisen direkte Aufrufer auf `locale.conf`.

Zur Diagnose vergleichen Sie `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` und `systemctl --user show-environment`. `localectl status` beschreibt das System, nicht die ARGVUS-Sprache des Benutzers. `ARGVUS_I18N_DIR` wählt ein Katalogverzeichnis für die Entwicklung, nicht die Sprache.
