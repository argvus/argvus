---
title: Importieren und Exportieren von Designs
description: Exportieren Sie das aktive Thema als ein eigenes Thema, importieren Sie ein Theme-Archiv, wenden Sie es an und löschen Sie es.
---
Export und Import funktionieren mit **Themen**, im gleichen Format wie die offiziellen Themen. Das Exportieren speichert das aktive Thema mit Ihrer Hervorhebungsfarbe und Ihrem Hintergrundbild als ein neues eigenes Thema. Das Importieren installiert ein Themenarchiv unter Ihren Themen. Beide finden Sie in **Systemsteuerung → Erscheinungsbild → Themen**, und Ihre Themen werden unter **Benutzerdefinierte Themen** angezeigt.

## Was ein exportiertes Thema enthält

Das Archiv, `argvus-theme-<id>.tar.gz`, enthält ein Themenverzeichnis, das wie ein offizielles Thema aufgebaut ist:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```
Das Manifest basiert auf dem aktiven Thema, mit Ihrem Namen, einer neuen ID, Ihrer Hervorhebungsfarbe als Palette-Akzent und dem Hintergrundbild daneben. Das Erscheinungsprofil bewahrt die aktuell in ARGVUS angewendeten visuellen Einstellungen, einschließlich Fensterabstände, Ränder, Taskleistenränder, Paneldarstellung, Farben, Effekte und Schriftarten. Benutzerdefinierte Hintergrundbilder und Akzentfarben werden mit dem Thema gespeichert. Das Anwenden eines Themas wendet dessen komplettes Profil erneut an (oder die ARGVUS-Standardeinstellungen, wenn kein Profil existiert) und ersetzt unabhängige visuelle Änderungen, die seit der letzten Anwendung vorgenommen wurden. Das Umschalten zwischen Sticky oder Spaced wendet stets die Fenster- und Taskleistengeometrie dieses Modus erneut an; speichern Sie persönliche Geometrie in einem exportierten benutzerdefinierten Thema, um sie bei der Anwendung dieses Themas verfügbar zu halten. Nicht-visuelle Einstellungen wie Audio-, Tastatur- und Netzwerkeinstellungen sind kein Teil eines Themas.

Das angewandte Erscheinungsbild bleibt über Sitzungsneustarts und Neuladungen hinweg bestehen. Die Sitzungs-Vorbereitung erstellt die notwendigen Dateien aus der aktuellen Konfiguration neu, ohne die Geometrie des ursprünglichen Themas wiederherzustellen.

Nach dem Anwenden eines Themas geänderte Einstellungen sind bei der nächsten Themen- oder Modusänderung vorübergehend. Exportieren Sie das Thema über **Systemsteuerung → Erscheinungsbild → Themen → Thema exportieren**, um die aktuelle visuelle Konfiguration zu speichern und später durch Anwenden dieses exportierten Themas wiederherzustellen.

Ihre Themen sind in `~/.local/share/argvus/themes/<id>/` installiert. Das Entpacken eines Archivs dort installiert das Thema ebenfalls.

## Das aktive Thema exportieren

1. Öffnen Sie **Systemsteuerung → Erscheinungsbild → Themen**.
2. Wählen Sie **Thema exportieren** oder drücken Sie `e` auf einer Themenseite.
3. Geben Sie einen Namen für das Thema ein. Das Feld zeigt die Datei an, die geschrieben wird.
4. Drücken Sie `Enter`.

Die ID wird aus dem Namen gebildet: Kleinbuchstaben, Ziffern und Bindestriche. Wenn die ID bereits von einem installierten Theme oder von einem Archiv in Ihrem Home-Verzeichnis verwendet wird, werden `-2`, `-3` und so weiter hinzugefügt. Das Archiv wird in Ihr Home-Verzeichnis geschrieben, zum Beispiel `~/argvus-theme-my-theme.tar.gz`. ARGVUS überprüft das Theme vor dem Schreiben: Das Manifest muss geladen werden und jede Komponente muss in beiden Modi angezeigt werden können.

## Ein Theme importieren

1. Kopieren Sie das `.tar.gz`-Archiv in Ihr Home-Verzeichnis (`~`). ARGVUS listet dort die Archive auf, die ein Theme enthalten.
2. Öffnen Sie **Systemsteuerung → Erscheinungsbild → Themes → Theme importieren** oder drücken Sie `i` auf einer Themes-Seite.
3. Wählen Sie das Archiv aus und drücken Sie `Enter`.

ARGVUS überprüft das Archiv, bevor es installiert wird. Es muss ein einzelnes Verzeichnis mit einem `theme.toml` enthalten, nur relative Pfade (kein `..`, keine Links), höchstens 128 Einträge und eine Größenbegrenzung pro Datei; die ID muss mit dem Verzeichnis übereinstimmen, das Hintergrundbild muss im Archiv enthalten sein und jede Komponente muss in beiden Modi dargestellt werden. Ein enthaltenes Erscheinungsprofil muss unterstützte Abschnitte und gültiges JSON verwenden. Wenn eine Überprüfung fehlschlägt, wird nichts installiert.

Das Thema wird zu **Benutzerdefinierte Themen** hinzugefügt. Das Importieren wendet es nicht an: Sie wählen, wann.

### Namenskollisionen

- Wenn eines Ihrer Themen dieselbe ID hat, fragt ARGVUS **Ihr Thema mit derselben ID ersetzen?**. Wählen Sie **Ersetzen**, um es zu ersetzen, oder drücken Sie `Esc`, um Ihres zu behalten. Das ersetzte Thema wird als Backup aufbewahrt, bis das neue installiert ist.
- Ein Archiv mit der ID eines offiziellen Themas wird abgelehnt. Exportieren Sie es erneut unter einem anderen Namen.

## Ein benutzerdefiniertes Thema anwenden

1. Öffnen Sie **Systemsteuerung → Erscheinungsbild → Designs → Benutzerdefinierte Designs**.
2. Wählen Sie das Design aus und drücken Sie `Enter`.

Die vollständige exportierte Konfiguration des Designs wird angewendet, einschließlich Wallpaper, Abstände, Ränder, Akzentfarbe, Effekte, Schriftarten und Panelpräsentation. Die visuellen Werte ersetzen die aktuellen Einstellungen. ARGVUS startet anschließend die Taskleiste, den Hyprland-Reload-Dienst, das Wallpaper und die Systemsteuerung neu, sodass jede Oberfläche das benutzerdefinierte Design verwendet.

## Benutzerdefiniertes Design löschen

1. Öffnen Sie **Systemsteuerung → Erscheinungsbild → Designs → Benutzerdefinierte Designs**.
2. Wählen Sie das Design aus und drücken Sie `d`.
3. Bestätigen Sie mit `y` oder drücken Sie `n` oder `Esc` zum Abbrechen.

Das Designverzeichnis wird aus `~/.local/share/argvus/themes` entfernt. Wenn es das aktive Design war, wechselt ARGVUS zuerst zu **ARGVUS Dark**.

## Profile früherer Versionen

Frühere Versionen exportierten *Themenprofile* (`profile.tar.gz`), eine Kopie der Erscheinungseinstellungen. Das erste Mal, wenn eine Themenseite geöffnet wird, wird jedes Profil in `~/.local/share/argvus/themes/<name>/` in ein Thema umgewandelt, das aus seinem Basisthema, seiner Hervorhebungsfarbe und seinem Hintergrundbild besteht. Das ursprüngliche Profil und die alte `custom-themes.json`-Liste werden in `~/.config/argvus/data/backups/theme-profiles-<seconds>/` verschoben, und die Statusleiste meldet die Konvertierung. Ein Profil, dessen Basisthema nicht installiert ist, wird nur in das Backup verschoben.

## Ein Thema teilen

Ein exportiertes Thema ist ein normales Archiv. Kopieren Sie es auf einen anderen Rechner, der ARGVUS ausführt, und importieren Sie es dort.

## Fehlerbehebung

- **Das Archiv wird nicht aufgelistet.** Es muss direkt in Ihrem Home-Verzeichnis liegen, mit `.tar.gz` enden und `<id>/theme.toml` enthalten.
- **Der Import wird abgelehnt.** Die Statuszeile gibt den Grund an: ein offizielles Thema mit derselben ID, ein fehlendes Hintergrundbild, ein Manifest, das nicht geladen wird, oder eine Komponente, die nicht gerendert wird.
- **Der Export wird abgelehnt.** Der Name muss mindestens einen Buchstaben oder eine Ziffer enthalten, und das aktive Thema muss installiert sein.
