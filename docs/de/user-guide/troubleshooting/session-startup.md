---
title: Fehlerbehebung beim Sitzungsstart
description: Fehler in Diagnose-Greeter, Hyprland und systemd-Benutzersitzung erkennen.
---
Lauf:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
Überprüfen Sie den Greeter, `argvus start`, Hyprland und die relevante Benutzerunit separat. Ein erfolgreicher Paketbau beweist nicht, dass die installierte Nutzlast oder die aktive Benutzerüberschreibung aktuell ist.

Verwenden Sie die Überprüfungen in dieser Reihenfolge:

1. Bestätigen Sie, dass der Greeter das installierte ARGVUS greetd-Setup verwendet.
2. Lesen Sie das Sitzungsprotokoll und suchen Sie nach einer fehlenden oder ungültigen Hyprland Lua-Konfiguration.
3. Überprüfen Sie die Bereitschaft und den Zielzustand mit `systemctl --user status argvus-session.target`.
4. Untersuchen Sie die fehlerhafte Komponente, anstatt alle Dienste gleichzeitig neu zu starten.

Wenn `argvus-session.target` inaktiv ist, während das Ladeoverlay aktiv ist, untersuchen Sie den Projektionsfehler, bevor Sie Dienste neu starten:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` ist der einzige Schreiber unter `data/generated/`, daher ist `project` die autoritative Methode, um eine Generation bei der Diagnose zu reproduzieren. Der Sitzungsstart führt sie aus, weil noch kein verbindlicher Plan existiert; der Plan, den sie in `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` schreibt, wird später von `argvus sessionctl reload` angewendet, ohne erneut projiziert zu werden.

Die Sitzung beendet die Ladeanzeige, wenn die fatale Vorbereitung oder der Start des Ziels fehlschlägt. Ein Fehler bei einem visuellen Verbraucher sollte entweder das dokumentierte Fallback verwenden oder explizit gemeldet werden; er darf nicht als erfolgreiche Sitzung verborgen werden.

## Langsames Einloggen

Wenn `greeter.handoff-takeover-failed` ohne `handoff.master-acquired` erscheint, überprüfen Sie `handoff.master-unavailable card=<name> errno=<n>` auf den Erwerbsfehler. Ein Beobachter, der geöffnet wurde, während ein anderer Kompositor Master war, kann errno 13 (`EACCES`) zurückgeben, selbst nachdem dieser Kompositor Master freigegeben hat. Der Helfer öffnet dieselbe GPU erneut, um Master implizit zu erwerben; `handoff.master-reopened` protokolliert diese Wiederherstellung, und ein neuer Framebuffer wird auf diesem Deskriptor vorbereitet. Der Begrüßer bricht den fehlgeschlagenen Helfer (`greeter.handoff-cancelled`) ab, sodass er keinen weiteren Warteschritt hinzufügen kann, bevor der Benutzerkompositor startet. Eine Sitzung, die `release` sendet, während der Erwerb aussteht, erhält `handoff.release-before-master`: der Helfer schließt seine DRM-Deskriptoren, bevor er dies bestätigt. Ein Absturz des Begrüßer-Kompositors in Hyprland/Aquamarine ist eine separate Feststellung; das Abbrechen des Helfers zeigt nicht, dass das Treiber- oder Kompositorproblem behoben wurde.

`greeter.vt-switch-returned active=<tty>` zeichnet das aktive VT auf, wenn der Wechselaufruf von logind zurückkehrt. Ein Zeichnungs-Timeout zeichnet ebenfalls `active-vt=<tty>` in seinem Fehler auf. Dies sind Momentaufnahmen, kein Beweis für eine abgeschlossene DRM-Übergabe.

Für einen Vorher-/Nachher-Vergleich speichern Sie die Arbeit und verwenden fünf Logins pro Version auf derselben Maschine, mit demselben Thema und Monitorkonfiguration. Vor jedem folgenden Logout kopieren Sie `/run/argvus-greeter/timing.log`, das Sitzungsprotokoll und das Boot-Journal in ein separates Probenverzeichnis unter `/tmp/argvus/`; der nächste Greeter-Durchlauf ersetzt sein Zeitprotokoll. Notieren Sie die Boot-ID mit jeder Probe und vergleichen Sie die Zeitstempel nur innerhalb dieses Boots. Messen Sie die Zeit von der Authentifizierung bis zur Übergabe des ersten Frames, den Eintritt in den Session-Wrapper, den finalen Start von Hyprland und das Stoppen des Lade-Spinners. Vergleichen Sie die mittleren und schlechtesten Zeiten von Authentifizierung bis Spinner-Stopp und prüfen Sie visuell auf schwarze Frames, Flackern oder hängende Spinner. Allein die Dauer der Desktop-Oberflächen ist nicht die gesamte Login-Zeit. Wiederholen Sie Fehlerschlacht- und Wiederherstellungstests in einer temporären Sitzung, ohne eine Sitzung mit ungespeicherter Arbeit zu beenden.

Jeder Startschritt zeichnet seine Dauer im Sitzungsprotokoll auf. Um die Schritte des letzten Logins zu sehen, zuerst die langsamsten:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
Um den Wechsel vom Begrüßungsbildschirm zum Desktop zu timen, lesen Sie die Marks des Begrüßers und der Übergabe zusammen mit dem Journal. Alle verwenden `CLOCK_MONOTONIC`, sodass sie übereinstimmen:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` enthält nur den letzten Login: den Begrüßungsbildschirm, die DRM-Übergabeoberfläche (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) und `greeter.*`-Marken. `handoff.crtc`-Zeilen protokollieren jede CRTC-Änderung, während die Übergabe auf den DRM-Master wartet; eine Zeile mit `fb=0` oder `mode_valid=false`, oder `handoff.output-found ... framebuffer=0`, bedeutet, dass der Begrüßungs-Compositor die Ausgabe ausgeschaltet hat, bevor die Übergabe sie übernommen hat, sodass der Monitor sich erneut synchronisieren musste. `greeter.watchdog-stop-requested` bedeutet, dass die Begrüßungssitzung endete, ohne auf ihren Compositor zu warten, und `greeter.compositor-exit-deferred` zeigt, dass der Compositor aufgefordert wird, zu beenden, sobald die Übergabeoberfläche verschwunden ist; `greeter.compositor-exit-requested` allein bedeutet, dass er sofort aufgefordert wurde zu beenden. Wenn die greetd- und logind-Zeilen im Journal eine Lücke von etwa einer halben Sekunde zwischen dem Schließen der Begrüßungssitzung und der Benutzersitzung zeigen Beim Start wurde der Abbau des Begrüßungs-Compositors während der Anmeldung ausgeführt. Das wird erwartet, während die Markierung `compositor-in-foreground=true` anzeigt: das Beenden des Terminals des Begrüßers beendet den Compositor vor dem verzögerten Beenden. Das passiert, wenn der Begrüßer sein VT nicht verlassen konnte, bevor sein Compositor beendet wurde: suchen Sie nach `greeter.vt-switch-requested` gefolgt von `greeter.handoff-drawing`; wenn der zweite fehlt, sagt `greeter.handoff-takeover-failed error=<reason>`, warum. `handoff.page-flip` bedeutet, dass die Übergabe den letzten Frame des Begrüßers ohne Moduswechsel ersetzt hat; `handoff.page-flip-failed errno=<n>` gefolgt von `handoff.modeset` bedeutet, dass der Kernel den Seitenwechsel verweigert hat. `handoff.overlay plane=<id>` und dann `handoff.retired` bedeuten, dass das Overlay Hyprlands erste Frames verborgen hat, bis der Sitzungslader auf dem Bildschirm war; `handoff.overlay-unavailable` bedeutet, dass das CRTC keine freie Overlay-Ebene hatte, und `handoff.retire-timeout` bedeutet, dass der Sitzungslader niemals gesendet wurde. `retire`. Der Sitzungsspinner (`argvus-greeter-handoff`) schreibt `first layer-shell frame callback` und `stop requested` in das Journal.

Ein `project.theme-switch` oder ein ungleich null `prepare.startup-theme` Eintrag wird nach einer Erscheinungsänderung, geänderten Themen-Dateien, einem neuen Akzent oder der Migration eines älteren Startstempels erwartet. Die Neuinstallation identischer Dateien bewahrt den inhaltbasierten Startcache. Wenn es bei jeder Anmeldung erscheint, fehlen die materialisierten Themen-Dateien; führen Sie `argvus theme-switch <theme>` einmal aus.

Das Sitzungsprotokoll wird unter `$XDG_STATE_HOME/argvus/session.log` (normalerweise `~/.local/state/argvus/session.log`) geführt. Nach der Änderung einer Benutzer-Hyprland-Überschreibung, melden Sie sich ab und wieder an, damit der Sitzungseinstiegspunkt sie erneut validiert und auswählt. Siehe [Graphische Sitzung](/docs/argvus-session/graphical-session/) für den normalen Startablauf.
