---
title: Abnehmbare Geräte
description: Wechselbare Speicher von ARGVUS bereitstellen und verwalten.
---
`argvus-removable-devices` überwacht UDisks2 und stellt das ARGVUS-Taskleisten-Speichermodul sowie dessen Gerätemenü bereit. Das Menü ist auch als separate Rofi-ähnliche Oberfläche verfügbar, sodass dieselben Geräteaktionen sowohl über die Taskleiste als auch über einen Launcher-Workflow verwendet werden können.

Die verfügbaren Aktionen sind `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` und `copy`. Die Befehle `list`, `watch` und `menu` stellen den entsprechenden Status, den Taskleisten-Stream und die Menüoberflächen für Integrationen zur Verfügung.

## Taskleistenmenü

Mit `"mode": "gui"` (dem Systemsstandard) öffnet ein Klick auf das Speicher-Symbol in der Taskleiste ein GTK-Menü, das an die ARGVUS-Taskleiste angehängt ist, ähnlich wie das Kalender-Popup: unterhalb der Leiste, wenn die Taskleiste oben auf dem Bildschirm ist, und oberhalb der Leiste, wenn die Taskleiste unten ist. Jede Gerätezeile hat ein Aktions-Untermenü mit nur den Aktionen, die ihr Status erlaubt.
- **Zeiger:** Das Überfahren oder Klicken einer Gerätezeile öffnet ihr Untermenü. Das Untermenü bleibt geöffnet, während der Zeiger die Menügrenzen überschreitet oder die Menüs verlässt, und es folgt einer anderen Zeile erst, nachdem der Zeiger dort kurz verweilt hat, sodass ein diagonaler Weg zum Untermenü es nicht schließt. Ein Klick außerhalb beider Menüs schließt das Popup.
- **Tastatur:** `Up`/`Down` wählt eine Zeile aus, `Right`, `Enter` oder `Space` öffnet das Untermenü (oder führt **Aktualisieren**/**Beenden** aus), `Enter` oder `Space` führt die ausgewählte Aktion aus, `Left` oder `Escape` kehrt zur Geräteliste zurück, und `Escape` schließt dort das Popup.

**Pfad kopieren** legt den Einhängepunkt über `copy_command` (Standard `wl-copy`, von `wl-clipboard`) in die Zwischenablage, sodass der Pfad nach dem Schließen des Menüs weiterhin vorhanden ist; eine Benachrichtigung bestätigt dies, wenn `show_notification` aktiviert ist. **Sperren** wird für jedes entsperrte verschlüsselte Laufwerk angeboten und wird bei Bedarf zuerst ausgehängt.

Die Standardaktion des Datei-Managers folgt dem im ARGVUS-Kontrollzentrum ausgewählten Datei-Manager. Dies sorgt dafür, dass Aktionen für Wechseldatenträger mit den Standardanwendungen des Benutzers konsistent sind.

Die Systemkonfiguration ist `/etc/argvus/removable-devices/config.json`. Die ARGVUS-verwaltete benutzerspezifische Projektion ist `~/.config/argvus/data/removable-devices/config.json`, mit einem passenden `theme.css` daneben. Dieses Stylesheet wird auch von der kanonischen Vorlage, Akzentfarbe und Modus nach `~/.config/argvus/data/generated/removable-devices/theme.css` projiziert, mit einer Familien-/Varianten-Fallback-Lösung für Designs, die keinen speziellen Wechseldatenträger-Asset enthalten, sodass ein Themenwechsel das Modul konsistent hält, ohne dass ein manueller Schritt erforderlich ist. Alte Komponentenpfade sind Migrationsinputs; spätere Benutzerkonfigurationen überschreiben die Systemstandards.

Nützliche Einstellungen umfassen, ob das Modul ausgeblendet wird, wenn kein Gerät vorhanden ist (`hide_when_empty`), ob Namen und Kapazitäten angezeigt werden, die Sortierreihenfolge der Geräte, das Benachrichtigungsverhalten, das Menü-Backend und Statussymbole. Sein Stylesheet wird aus dem aktiven ARGVUS-Theme gerendert (`removable-devices/theme.css` des Theme-Builds) und bei jeder Theme- oder Akzentänderung in das `theme.css` des Panels kopiert.

Die Schnittstelle verwendet die Benutzersprache aus `LANG` in `$XDG_CONFIG_HOME/locale.conf`, wobei Englisch die Standardoption für einen nicht konfigurierten Benutzer ist. Vererbte Locale-Variablen wählen die Schnittstellensprache nicht aus.

## Geteilte Sprachkataloge

Die Schnittstelle verwendet die geteilten `argvus-i18n`-Kataloge für alle zehn Sprachen. Texte werden in `resources/locales/<locale>/removable-devices.json` im Master-Arbeitsbereich gespeichert und von `argvus` installiert. Englisch ist die Ausweichsprache; neue Sprachen erfordern nicht mehr unbedingt ein lokales zweisprachiges Wörterbuch.
