---
title: ARGVUS Festplattennutzungs-Analyzer
description: Durchsuchen Sie die Festplattennutzung interaktiv mit ncdu in einem thematisierten Kitty-Fenster.
---
Öffnen Sie **ARGVUS Disk Usage Analyzer** über den Anwendungsstarter oder führen Sie aus:

```sh
argvus du
```
Der Analyzer öffnet ein dediziertes, zentriertes Kitty-Fenster und bettet die echte ncdu-Oberfläche in seinen ARGVUS-Rahmen ein. Er startet in `$HOME`. Bearbeiten Sie den Verzeichnispfad oder wählen Sie **Durchsuchen**, um den konfigurierten Dateimanager zu öffnen. Mit Superfile navigieren Sie zu einem Verzeichnis und drücken **Leertaste**, um es auszuwählen; `q` oder `Esc` bricht ab. Andere konfigurierte Dateimanager öffnen sich normal, geben jedoch möglicherweise kein ausgewähltes Verzeichnis zurück, daher geben Sie den Pfad im Analyzer ein oder fügen ihn ein.

Verwenden Sie Tab oder die Pfeiltasten, um zwischen dem Verzeichnis, den Optionen und den Aktionen zu wechseln. Das Diagramm, der Prozentsatz, das dunkle Farbschema und die Verzeichnisse-zuerst-Anordnung sind standardmäßig aktiviert. Die übrigen Scan- und Anzeigeoptionen können vor der Auswahl von **Analysieren** umgeschaltet werden. In den Ergebnissen verwenden Sie die Tastatursteuerung von ncdu; `q` kehrt zum Formular zurück und `Esc` stoppt den Scan. Drücken Sie `d`, um die ausgewählte Datei oder das ausgewählte Verzeichnis zu löschen. ncdu fragt nach einer Bestätigung; das Löschen ist dauerhaft und erfolgt mit den Berechtigungen des aktuellen Benutzers. Keine erhöhten Privilegien werden angefordert.

Die äußere Schnittstelle folgt dem aktiven ARGVUS-Thema und der Spracheinstellung. Kitty und ncdu sind erforderlich; das `argvus-devtools`-Paket stellt ncdu bereit. Wenn Kitty oder ncdu fehlen, meldet die Anwendung die fehlende Abhängigkeit.
