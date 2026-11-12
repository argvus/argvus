---
title: Kontrollzentrum
description: Konfigurieren Sie die persistenten Teile Ihres ARGVUS-Desktops.
---
Das Kontrollzentrum ist ARGVUS' Tastatur-zuerst Einstellungsanwendung. Verwenden Sie es, wenn Sie den Desktop konfigurieren möchten, anstatt eine einmalige Aktion auszuführen.

## Kontrollzentrum und Systemsteuerung

| Verwenden… | Für… |
| --- | --- |
| **Systemsteuerung** | Themen, Hintergrundbilder, Effekte, Layout, Taskleistenverhalten, Schriftarten, Standardanwendungen, Tastatur- und Eingabeeinstellungen, Sprache, Region und Systemkonfiguration. |
| **Kontrollfeld** | Aktueller Status und häufige Aktionen wie Lautstärke, Helligkeit, Netzwerk, Benachrichtigungen, Energie- und Sitzungskontrollen. |

Das Bedienfeld kann Schnellsteuerungen für eine Einstellung anzeigen, die dauerhaft im Kontrollzentrum konfiguriert ist. Verwenden Sie beispielsweise das Bedienfeld für eine schnelle Erscheinungs- oder Anzeigeaktion und das Kontrollzentrum, wenn Sie die dahinter liegende Desktop-Konfiguration ändern möchten. Siehe Bedienfeld und [Wo man Dinge konfiguriert](/docs/user-guide/where-to-configure/).

## Hauptbereiche

Das aktuelle Startmenü des Kontrollzentrums bietet diese Bereiche:

Die Startseite ist ein Katalog von Reisezielen, die in sechs Karten gruppiert sind. Jedes Reiseziel öffnet seine Seite direkt, und die Suche findet es unter jeder Karte.
- **Aussehen** — Passe den Desktop an: Desktop-Stil (Thema, Erscheinungsmodus, Akzentfarbe, Hintergrundbilder, Symbole und Schriftarten), Anwendungsstil (Transparenz von Terminal und Launcher), Taskleiste, Systemsteuerung und Widget-Telemetrie.
- **Desktop & Eingabe** — den Arbeitsbereich konfigurieren: Hyprland (Windows: Regeln, Bereiche und Modi, Animationen, Unschärfe, Rahmen, Schatten und die installierte Version), Displays, Tastatur & Tastenkombinationen und Maus & Touchpad.
- **Einstellungen** — persönliche und regionale Einstellungen: Sprache & Region (die Sprache, Zeitzone, Formate und Tastaturlayout-Zentrale), Datum & Uhrzeit, Standardanwendungen, Energie und Sitzung.
- **Hardware & Konnektivität** — Geräte und Verbindungen: Hardware-Übersicht, Netzwerk & WLAN, Bluetooth, Audio und Speicher.
- **Systemadministration** — das Betriebssystem verwalten: Pakete, Dienste, Start, Hostname, Benutzer & Gruppen, Firewall und Benachrichtigungen (Nicht stören).
- **Einstellungen** — das Kontrollzentrum selbst, nach den Karten aufgeführt: Sprache (die Benutzersprache) und Stil (Dekorative Symbole und Transparenz). Drücken Sie `s`, um es zu öffnen.
- **Werkzeuge & Diagnosen** — Entwickler- und Wartungstools: ARGVUS Willkommen, Projekte, Snippets (Text gespeichert für `argvus snippets`, eingegeben mit `SUPER + ALT + N`), SSH-Schlüssel (Schlüsselpaar in `~/.ssh` verwaltet von `argvus keys`), GPG-Schlüssel (der Schlüsselbund verwaltet von `argvus gpg`; gpg-agent verwendet pinentry-gtk), Entwicklerdienste, Systemdiagnose und Über ARGVUS.

Wenn 'Nicht stören' aktiviert ist, werden Benachrichtigungs-Pop-ups geschlossen und neue Benachrichtigungen für später aufbewahrt. Das Bedienfeld verbirgt seine Benachrichtigungshistorie, bis 'Nicht stören' deaktiviert wird.

## ARGVUS Willkommen

Wählen Sie **Open ARGVUS Welcome** unter **Tools & Diagnostics**, suchen Sie nach `welcome` oder führen Sie `argvus welcome` aus. Welcome ist in das Control Center integriert und verwendet dessen Einzel-Fenster. Enter öffnet die ausgewählte Einstellungsseite; nach dem Beenden kehren Sie mit Esc zum Leitfaden zurück. F6 Next markiert den aktuellen Schritt als erledigt und öffnet den nächsten im selben Fenster. Esc geht nur zurück, und Enter in der Checkliste öffnet immer die ausgewählte Seite. O überprüft den Schritt erneut, S überspringt ihn, ohne ihn abzuhaken, und F beendet. Der Leitfaden auch mit ausstehenden optionalen Schritten. Abbrechen oder Zurückkehren beendet keinen Schritt. Bestehender Fortschritt wird gespeichert.

Der Leitfaden behandelt Sprache und Region, Bildschirme, Hardware und Treiber, Netzwerk, Audio, Bluetooth, Erscheinungsbild, Energie, Standardanwendungen und Sitzungsstart. Er installiert niemals Treiber oder Pakete und nimmt keine Systemänderungen selbst vor. Der Fortschritt wird in `$XDG_STATE_HOME/argvus/welcome/progress.json` (oder `~/.local/state/argvus/welcome/progress.json`) gespeichert.

Nachdem der Desktop bereit ist, führt die Sitzung `argvus welcome --if-needed` für TTY, ARGVUS Greeter und andere Anmelde-Manager aus. Es öffnet Willkommen, es sei denn, es liegen pro Benutzer Fortschrittsaufzeichnungen `opened`, Fertigstellung oder Schließen vor. Die erste erfolgreiche Darstellung speichert `opened` in `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, wodurch ein erneutes Öffnen bei späteren Anmeldungen verhindert wird. Es hängt nicht von `/etc/skel` ab. Bestehende Konten ohne diese Aufzeichnung erhalten ebenfalls eine automatische Öffnung. Fehler vor der Darstellung eine weitere Anmeldung beim nächsten Login zulassen. Manuelles `argvus welcome` bleibt verfügbar.

Die genauen Anbieterseiten können von den installierten Paketen und Berechtigungen abhängen. Gehen Sie nicht davon aus, dass ein fehlendes optionales Domain ein defektes Control Center ist.

Wenn das Raster mehr Einträge hat, als auf den Bildschirm passen, erscheint am rechten Rand eine Bildlaufleiste, um zu zeigen, dass unten noch mehr vorhanden ist.

## Eine Einstellung finden

Der Startbildschirm registriert Einstellungen nach Titel, Kategorie, Kennung und Schlüsselwörtern. Die Suche ist akzentunempfindlich und akzeptiert praktische Begriffe wie `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` und `DPI`. Exakte Titelübereinstimmungen werden zuerst angezeigt, gefolgt von einem Titelfortsatz, jedem Wort des Titels oder der Kategorie, das den eingegebenen Text enthält, und schließlich Schlüsselwortübereinstimmungen — so kann das Eingeben eines beliebigen Teils eines Wortes (nicht nur seines Anfangs) trotzdem das Ziel finden.

Das sichtbare Menü ist fähigkeitsbewusst. Eine Route kann fehlen, wenn ihr Anbieter nicht installiert ist, die Hardware nicht verfügbar ist oder die aktuelle Sitzung die erforderliche Fähigkeit nicht bereitstellen kann. Die Suche ist daher auch ein nützliches Mittel, um zu entdecken, was diese Installation konfigurieren kann.

Der aktuelle Haupteinstellungsbaum umfasst derzeit:

| Bereich | Aktuelle Ziele |
| --- | --- |
| Erscheinungsbild | Desktop-Stil: Thema, Erscheinungsmodus, Akzentfarbe, Hintergrundbilder, Symbole und Schriftarten; Anwendungsstil: Terminal- und Launcher-Transparenz; Taskleiste; Systemsteuerung; Widget-Telemetrie |
| Desktop & Eingabe | Hyprland (Windows: Regeln, Bereiche und Modus, Animationen, Unschärfe, Rahmen und Schatten), Anzeigen, Tastatur & Verknüpfungen, Maus & Touchpad |
| Einstellungen | Sprache & Region (Sprache, Zeitzone, Formate, System-Lokale und Tastaturlayout), Datum & Uhrzeit, Standardanwendungen, Energie, Sitzung |
| Hardware & Konnektivität | Hardwareübersicht (Zusammenfassung, CPU, GPU, Speicher, Geräte), Netzwerk & WLAN, Bluetooth, Audio, Speicher |
| Systemverwaltung | Pakete, Dienste, Start, Hostname, Benutzer & Gruppen, Firewall, Benachrichtigungen |
| Einstellungen | Sprache (Benutzersprache), Stil (Symbole und Transparenz) |
| Werkzeuge & Diagnosen | Projekte, Snippets, SSH-Schlüssel, GPG-Schlüssel, Entwicklerdienste, Systemdoktor, Über ARGVUS |

Einige davon sind vollständige Seiten und einige Unterseiten, die über die Suche oder eine Domainzusammenfassung erreicht werden. Die genaue Liste wird durch die installierten ARGVUS-Anbieter bestimmt.

Jede Seite zeigt ihren Pfad in der Kopfzeile an, zum Beispiel `Control Center > Hardware & Connectivity > Audio`. Der Pfad beginnt mit der Startseite, die die Seite auflistet, sodass dieselbe Seite von der Startseite, aus der Suche und von der CLI aus gleich gelesen wird. Die Wurzel- und Kartennamen sind übersetzt: Die portugiesische Oberfläche zeigt `Central de Controle > Hardware e conectividade > Áudio`.

## Wofür jeder Bereich gedacht ist

### Erscheinungsbild

Aussehen ist der Hauptanpassungsbereich. Die Desktop-Gruppe enthält das Thema (mit seinem Sticky- oder Spaced-Modus), den Erscheinungsmodus, die Akzentfarbe, die Hintergrundbilder, das Symbolthema und die Schriftseite, und die Anwendungsstil-Gruppe enthält die Terminal- und Starter-Transparenz. Taskleiste, Systemsteuerung und Widget-Telemetrie sind direkt unter Aussehen auf der Startseite aufgeführt und verwenden die gemeinsame Transparenzseite von Aussehen für ihre eigenen Oberflächen. Blur ist eine globale Einstellung unter Hyprland > Blur, und Es gilt für jede ARGVUS-Oberfläche, bei der Unschärfe aktiviert ist. Diese Seite bearbeitet ihren Aktivieren-Schalter und die Hyprland-Unschärfewerte (Größe, Durchläufe, Helligkeit, Rauschen, Kontrast, Lebendigkeit und Lebendigkeitsdunkelheit) als Entwurf, und nichts ändert sich, bis **Anwenden** ausgewählt wird. Animationen funktionieren auf die gleiche Weise unter Hyprland > Animationen: ihr Aktivieren-Schalter, die benutzerdefinierten Kurven (Bézier, Spring, Linear und Ease-out) und die fünf Animationskategorien bilden einen Entwurf, der zusammen mit **Anwenden** übernommen wird. Bézier und Federn haben editierbare Parameter (die Bézier-Punkte sowie Masse, Steifigkeit und Dämpfung der Feder); Linear und Ease-out sind feste Kurven, daher listet ihre Seite nur ihre Kontrollpunkte auf und kann nicht bearbeitet werden. Die Stiloptionen folgen dem, was Hyprland 0.56.2 zeichnet: Öffnen und Schließen bieten Slide, Popin und Gnomed, Layer-Eintritt und -Austritt bieten Slide, Popin und Fade, und die Arbeitsbereich-Verläufe bieten Slide, Fade, Slidefade, Slidevert und Slidefadevert. Diese Steuerelemente koordinieren mit dem Erscheinungsbild und Sitzungskomponenten anstatt nur das Control-Center-Fenster zu ändern.

### Taskleiste

Die Taskleiste konfiguriert die Position des Panels, den Abstand um es und seine Fenster, seine Höhe und Grenze, die Gruppe der Dienstprogramm-Symbole sowie die Datums-/Uhrzeitblöcke, zusätzlich zu ihrer eigenen Transparenz. Sie ist unter **Erscheinungsbild** auf der Startseite aufgeführt. Im Sticky-Modus ist **Spaces > Bottom** standardmäßig auf `2` eingestellt; Spaced behält seine `18`-Pixel-Basislinie bei.

**Höhe** wählt 24, 28 (standardmäßig), 32 oder 36 Pixel. **Rahmen** hat einen **Aktivieren**-Schalter und eine **Dicke** von 1 bis 3 Pixeln, wobei 1 der Standardwert ist. Wenn der Rahmen deaktiviert ist, haben das Taskleistenfenster, die rechte Gruppe und der MPRIS-Player keinen Rahmen. Diese Änderungen sind ein Entwurf, bis **Übernehmen** ausgewählt wird. Die übernommene Höhe und der Rahmen werden beibehalten, wenn das Thema gewechselt wird; bis sie übernommen werden, entscheidet das Thema darüber.

Die **Symbole**-Seite öffnet **Optionale Module**, **Arbeitsbereichspräsentation**, **Starter** und **Dienstprogramme**. Optionale Module können einzeln angezeigt oder ausgeblendet werden: Audioplayer, Focus, Starter, Netzwerk, Energieprofil, Tastaturlayout, Speicher, CPU- und Temperaturanzeigen, Datum, Uhrzeit, Batterie, Systemleiste, Benachrichtigungen, Wechselmedien, Aufnahme, Bluetooth, Audioeingang/-ausgang und Aktivhaltung. Arbeitsbereiche bleiben verfügbar und können ihre Namen (die Standardeinstellung) oder Anwendungsicons anzeigen. Der Titel des fokussierten Fensters, Einstellungen und Energieaktionen sind immer verfügbar. Die Sichtbarkeit des Launchers wird in den optionalen Modulen gesteuert; sein Untermenü ändert das benutzerdefinierte Symbol. Änderungen werden nach **Übernehmen** wirksam, was die Taskleiste neu startet.

### Systemsteuerung

Die Systemsteuerung konfiguriert die Sichtbarkeit und Reihenfolge ihrer Schnellaktionskarten sowie ihre eigenen Sitzungen und die Transparenz.

Die Seite öffnet sich und akzeptiert Eingaben, sobald die Karteneinstellungen gelesen wurden. Die Karten, die von der Hardware abhängen (Bluetooth und Helligkeit), werden im Hintergrund überprüft: bis diese Überprüfung abgeschlossen ist, behalten sie die zuletzt in diesem Fenster gesehene Verfügbarkeit, und die Liste aktualisiert sich von selbst, wenn das Ergebnis eintrifft.

### Widget-Telemetrie

Die Widget-Telemetrie konfiguriert die Sitzungen und die Transparenz des Systeminformations-Widgets.

### Schriftarten

Schriften werden über **Erscheinungsbild > Desktop-Stil > Schriften** erreicht. Es ermöglicht Ihnen, eine Familie und Größe für die Taskleiste, Telemetrie/Systeminformationen, die Systemsteuerung, die ARGVUS-Systemoberfläche, Anwendungen, das Terminal und den Browser auszuwählen. Es bietet auch Einstellungen zur Schriftwiedergabe wie Kantenglättung, Hinting, Subpixel-Modus und DPI. Die standardmäßige primäre Familie ist IBM Plex Mono; einzelne Ziele können unabhängig voneinander geändert und von der Schriften-Seite aus wiederhergestellt werden.

### Standardanwendungen

Standardanwendungen wählen installierte Programme für Rollen wie Terminal, Dateimanager, Texteditor, Terminaleditor, Browser, Bildbetrachter, PDF-Betrachter, Videoplayer, Audioplayer, Archivwerkzeug und Launcher aus. Die Auswahl wird von ARGVUS gespeichert und kann die standardmäßigen XDG-Zuordnungen aktualisieren. TUI-Dateimanager verwenden den ausgewählten Terminaleditor beim Öffnen von Textdateien; grafische Texteditoren bleiben die Standardanwendung für Desktop-Dateizuordnungen.

### Ausschnitte

Snippets ist die Seite für den Text, den `argvus snippets` mit `SUPER + ALT + N` eingibt. Sie befindet sich unter **Tools & Diagnostics → Snippets** (suche `snippets` oder führe `argvus control-center snippets` aus).
- **Neuer Schnipsel** — gib einen **Namen** und einen **Inhalt** ein, dann **Schnipsel speichern**. Ein Name, der bereits existiert, ersetzt den Inhalt dieses Schnipsels.
- **Gespeicherte Schnipsel** — aufgelistet mit ihrer Position (`1  email`) und dem Inhalt in einer Zeile. Drücke Enter bei den Inhaltsänderungen. **Typ** gibt den Schnipsel ein, und **Entfernen** fragt nach Bestätigung.
- **Auswahl öffnen** — öffnet denselben Auswahldialog wie `SUPER + ALT + N`.

Die Eingabe beginnt 3 Sekunden nachdem Sie die Eingabetaste drücken, da das Kontrollzentrum selbst in diesem Moment den Fokus hat. Klicken Sie in der Zwischenzeit auf das Feld, das den Text erhalten soll. Die Seite bearbeitet nur einzeilige Inhalte; ein mehrzeiliger Ausschnitt kann in einem Terminal mit `argvus snippets add` erstellt werden.

Die Seite liest die Einträge aus `argvus-config` und schreibt sie über `argvus snippets`, sodass der Launcher die Validierung beibehält. Wenn `argvus snippets` nicht installiert ist, meldet die Seite dies.

### SSH-Schlüssel

SSH-Schlüssel ist die Seite für die Schlüsselpaar, die `argvus keys` verwaltet. Sie befindet sich unter **Tools & Diagnostics → SSH keys** (suche `ssh keys` oder führe `argvus control-center ssh-keys` aus).

Die Seite hat vier Teile, von oben nach unten:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```
#### Wie man es benutzt

1. Öffnen Sie **Systemsteuerung → Werkzeuge & Diagnostik → SSH-Schlüssel**. Vom Startbildschirm aus können Sie auch nach `ssh keys` suchen.
2. **Einen Schlüssel finden:** Drücken Sie `/`, und geben Sie dann ein, um nur die Paare anzuzeigen, deren Name oder Kommentar übereinstimmt. `Enter` beendet die Suche und führt die markierte Zeile aus, wie in den anderen Listen, also überprüfen Sie die Zeile, bevor Sie sie drücken; `Esc` löscht die Suche.
3. **Ein Paar erstellen:** Wählen Sie in **Erzeugen** **Name** aus, drücken Sie `Enter`, geben Sie den Namen ein und drücken Sie erneut `Enter`. Wählen Sie **Typ** aus und drücken Sie `Enter`, um zwischen `ed25519` und `rsa` zu wechseln. Wählen Sie **Schlüsselpaar erzeugen** und drücken Sie `Enter`. Das neue Paar erscheint sofort unter **Schlüssel**, ohne dass eine Passphrase erforderlich ist.
4. **Öffnen Sie den öffentlichen Schlüssel:** Im Block des Paares wähle **Öffentlichen Schlüssel kopieren** und drücke `Enter`. Der Inhalt der `.pub`-Datei gelangt in die Wayland-Zwischenablage, bereit, in eine `authorized_keys` eines Servers oder eines Git-Hosts eingefügt zu werden. Der private Schlüssel wird niemals kopiert. Dies erfordert `wl-clipboard` (`wl-copy`); ohne ihn zeigt die Seite an, dass der Schlüssel nicht kopiert wurde.
5. **Ein Paar in den Agenten laden:** Wählen Sie im Block des Paares **In Agenten laden** und drücken Sie `Enter`. Ein Schlüssel ohne Passphrase wird sofort geladen, ohne Fenster. Ein Schlüssel mit Passphrase öffnet ein ARGVUS-Terminal, in dem OpenSSH danach fragt; schließen Sie das Terminal mit `Enter`, wenn angezeigt wird, dass der Schlüssel geladen ist. Die Seite überprüft den Agenten alle paar Sekunden, solange sie geöffnet ist, sodass die Zeile von selbst zu **Aus Agenten entladen** wechselt. Um sofort zu aktualisieren, verwenden Sie **Aktualisieren** unter **Aktionen**.
6. **Einen Schlüssel aus dem Agent entfernen:** Wählen Sie im Block des Paares **Vom Agent entladen**. Um jeden Schlüssel zu entfernen, verwenden Sie **Alle Schlüssel aus dem Agent entladen** unter **Aktionen**. Die Schlüsseldateien bleiben auf der Festplatte.
7. **Ein Paar löschen:** Wählen Sie im Block des Paares **Löschen** und drücken Sie `Enter`. Die Bestätigung fokussiert **Abbrechen**, sodass ein zufälliges `Enter` nichts bewirkt. Drücken Sie `y`, um zu bestätigen, oder `n`/`Esc`, um abzubrechen. Ein im Agent geladenes Schlüssel kann nicht gelöscht werden; laden Sie ihn zuerst aus.

Bewegen: `↑`/`↓` oder `j`/`k` bewegen sich durch die Liste, `Enter` aktiviert eine Zeile, `Esc` geht zurück und `?` zeigt Hilfe. `Tab` verschiebt den Fokus auf **Aktionen**, wobei `←`/`→` eine Schaltfläche auswählen und `Enter` sie ausführen. `↑`/`↓` bringen den Fokus zurück zur Liste.

Passwörter werden niemals im Kontrollzentrum eingegeben und niemals als Befehlsargumente übergeben. Die Seite liest die Liste mit `argvus keys list` und schreibt nur über `argvus keys`. Wenn `argvus-keys` nicht installiert ist, zeigt die Seite dies an.

#### Fehlerbehebung

| Symptom | Ursache und Lösung |
| --- | --- |
| `Could not open a connection to your authentication agent` nach **In Agent laden** | Das Terminal hat den Agenten-Socket nicht empfangen. Hyprland behält die Umgebung, mit der es gestartet wurde, und der Agent könnte später gestartet worden sein. Aktualisieren Sie `argvus-launcher`, das auf `$XDG_RUNTIME_DIR/ssh-agent.socket` zurückfällt, und überprüfen Sie `systemctl --user status argvus-ssh-agent.service`. |
| **Agent** zeigt `Agent unavailable` | Der Launcher kann den Agenten nicht erreichen. Überprüfen Sie den Dienst wie oben beschrieben. |
| Die Seite zeigt `argvus-keys is not installed` | Aktualisieren Sie das `argvus`-Paket: `argvus keys` ist ein Unterbefehl davon. |
| **Löschen** zeigt einen Fehler | Der Schlüssel ist im Agenten geladen und wird aufbewahrt. Laden Sie ihn zuerst aus, und löschen Sie ihn dann erneut. |

### GPG-Schlüssel

GPG-Schlüssel ist die Seite für den Schlüsselbund, den `argvus gpg` verwaltet. Sie befindet sich unter **Werkzeuge & Diagnose → GPG-Schlüssel** (suche `gpg keys` oder führe `argvus control-center gpg-keys` aus).

Die Seite hat diese Abschnitte von oben nach unten:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```
`▾` markiert einen erweiterten Abschnitt und `▸` einen zusammengeklappten. `Enter` oder `Space` erweitert oder klappt ihn zusammen.

#### So verwenden Sie es

1. Öffnen Sie **Control Center → Tools & Diagnostics → GPG-Schlüssel**. Vom Home aus können Sie auch nach `gpg keys` suchen.
2. **Pinentry** ist auf `pinentry-gtk` festgelegt. Die Seite zeigt nur dies an. Das erste Mal, wenn ein Schlüssel generiert wird, schreibt ARGVUS diesen Pinentry in `gpg-agent.conf`, zwischen seine eigenen Markierungslinien, und behält Ihre anderen Zeilen bei. Wenn Sie bereits eine `pinentry-program`-Zeile außerhalb dieser Marker geschrieben haben, stoppt die Generierung mit einem Fehler, damit Sie diese Zeile entfernen können.
3. **Erstellen Sie einen Schlüssel:** Geben Sie in **Schlüssel erzeugen ▾** **Vollständiger Name** und **E-Mail** ein. Beide sind erforderlich, und **Schlüssel erzeugen** bleibt deaktiviert, bis beide Text enthalten. **Kommentar** ist optional. Die Benutzer-ID wird `Name (comment) <e-mail>`.
- **Algorithmus** wechselt zwischen **RSA** und **DSA ElGamal**. Ein DSA-Schlüssel erhält einen ElGamal-Verschlüsselungsunter­schlüssel; ein RSA-Schlüssel erhält einen RSA-Unter­schlüssel.
- **Hauptstärke** durchläuft die Größen des Algorithmus: RSA 1024, 2048 und 4096; DSA 1024, 2048 und 3072. gpg erstellt keine DSA-Schlüssel über 3072 Bits, daher wird 4096 für DSA nicht angeboten.
- **Ablaufdatum** öffnet eine eigene Seite. Geben Sie das Datum im Layout Ihrer Sprache ein: `DD/MM/YYYY` in Portugiesisch, Spanisch, Französisch, Italienisch, Deutsch und Russisch; `YYYY/MM/DD` in Englisch, Chinesisch, Japanisch und Koreanisch. Das Datum muss nach dem heutigen Tag liegen. Schalten Sie **Nie ablaufen** ein, um den Schlüssel dauerhaft zu machen; das Datumsfeld wird dann deaktiviert und nicht verwendet.
- **Passphrase** öffnet seine eigene Seite. **Mit einer Passphrase schützen** ist standardmäßig aktiviert; geben Sie die **Passphrase** ein und dann die **Passphrase bestätigen**. Beide müssen übereinstimmen. Das Deaktivieren des Schutzes löscht beide Felder, und die Seite fragt nach einer Bestätigung, bevor ein ungeschützter Schlüssel erstellt wird.
4. Wählen Sie **Schlüssel generieren** und drücken Sie `Enter`. Der Schlüssel erhält einen Signaturschlüssel und einen Verschlüsselungs-Unterschlüssel. Die Passphrase wird über die Standardausgabe an den Launcher gesendet, niemals als Befehlsargument. Wenn die Generierung fehlschlägt, bleiben die anderen Felder bestehen, aber die Passphrasen werden gelöscht und müssen erneut eingegeben werden.
5. **Abbrechen** löscht jedes Feld des Formulars und setzt die Daten und Größen auf ihre Standardwerte zurück.
6. **Importieren:** Wählen Sie in **Import ▾** **Öffentlichen Schlüssel importieren…** oder **Privaten Schlüssel importieren…**. Der Dateimanager öffnet sich, damit Sie die Datei auswählen können. Ein privater Schlüssel verlangt zuerst eine Bestätigung, und sein Passwort wird von gpg abgefragt, wenn der Schlüssel verwendet wird. Wenn der Dateimanager grafisch ist, kann er die Datei nicht an die Seite melden; die Seite öffnet dann ein Feld, in dem Sie den Pfad eingeben.
7. **Schlüssel ›** öffnet den Schlüsselbund. Jeder Schlüssel zeigt seine Identität, Fingerabdruck, Typ, Ablaufdatum und ob sein privater Schlüssel im Schlüsselbund ist. Jeder Schlüssel hat folgende Aktionen:
   - **Schlüssel bearbeiten ›** öffnet die eigene Seite des Schlüssels (unten beschrieben).
- **Öffentlichen Schlüssel exportieren…** fragt nach einem Dateipfad, vorgeschlagen als `~/<fingerprint>.asc`. Die Datei wird mit dem Modus `600` erstellt. Eine vorhandene Datei wird niemals ersetzt.
- **Privaten Schlüssel exportieren…** erscheint nur bei Schlüsseln mit einem privaten Teil. Es fragt nach Bestätigung und dann nach einem Dateipfad, vorgeschlagen als `~/<fingerprint>-private.asc`. Die Datei wird mit dem Modus `600` erstellt.
- **Öffentlichen Schlüssel kopieren** und **Fingerabdruck kopieren** kopieren in die Wayland-Zwischenablage mit `wl-copy`.
- **Schlüssel löschen…** fragt nach Bestätigung, mit dem Fokus auf Abbrechen. Es entfernt den Schlüssel und seinen privaten Teil.
8. **Finde einen Schlüssel:** Auf der **Schlüssel**-Seite filtert die Suchleiste oben die Schlüssel. Tippe, um die Liste einzugrenzen; die Suche stimmt mit der Identität (die den Kommentar einschließt) und dem Fingerabdruck überein. `Esc` löscht die Suche.
9. **Schlüssel bearbeiten ›** zeigt die Details des Schlüssels an, dann diese Aktionen:
   - **Ablaufdatum ›** ändert das Ablaufdatum des Schlüssels und jedes seiner Unterschlüssel. Verwende **Nie läuft ab**, um das Ablaufdatum zu entfernen. **Anwenden** speichert die Änderung.
- **Passphrase ändern** fragt im Pinentry-Fenster nach der alten und der neuen Passphrase.
- **Key mit › signieren** zertifiziert diesen Schlüssel mit einem Ihrer anderen Schlüssel, der über einen privaten Teil verfügt. Die Passphrase des Signierenden wird abgefragt. Die Seite fragt zuerst nach einer Bestätigung.
- **Schlüssel widerrufen ›** listet die Gründe auf: kein Grund, Schlüssel kompromittiert, Schlüssel ersetzt, Schlüssel wird nicht mehr verwendet. Jeder Grund fragt nach einer Bestätigung. Ein widerrufener Schlüssel kann nicht mehr verwendet werden.
- **Vertrauen** legt fest, wie sehr Sie dem Schlüssel vertrauen: nicht gesetzt, marginal, voll oder ultimativer Grad.
- **An Keyserver senden…** sendet den öffentlichen Schlüssel nach Bestätigung an den in `gpg.conf` festgelegten Keyserver. **Vom Keyserver abrufen** lädt den Schlüssel von diesem Keyserver herunter.

Navigation: `↑`/`↓` oder `j`/`k` bewegen sich durch die Liste, `Enter` aktiviert eine Zeile, `Esc` geht zurück und `?` zeigt die Hilfe an.

Passphrasen werden nur in die maskierten Felder der Passphrase-Seite oder in das Pinentry-Fenster eingegeben. Die Seite sendet die generierte Passphrase an `argvus gpg` über die Standardeingabe, und der Launcher übergibt sie an gpg über einen Dateideskriptor, sodass sie niemals in der Prozessliste, in einem Befehlsargument oder in den Protokollen erscheint. Schlüsseländerungen laufen im Hintergrund; während eine läuft, warten die anderen. Die Seite liest den Schlüsselbund mit `argvus gpg list` und schreibt nur über `argvus gpg`. Wenn `argvus gpg` nicht installiert, die Seite sagt es.

Noch nicht auf der Seite: Suche auf dem Keyserver nach einem Schlüssel per Name oder E-Mail. Verwenden Sie **Von Keyserver abrufen** mit einem bekannten Fingerabdruck.

#### Fehlerbehebung

| Symptom | Ursache und Lösung |
| --- | --- |
| Die Seite sagt `argvus gpg is not available` | Das installierte `argvus-launcher` ist älter als der GPG-Befehl. Führen Sie `argvus gpg list` im Terminal aus: `unknown command` bedeutet, dass das Paket neu gebaut und installiert werden muss. Schlüssel müssen ebenfalls `gnupg`. |
| **Schlüssel erzeugen** bleibt deaktiviert | Geben Sie sowohl **Vollständiger Name** als auch **E-Mail** ein. |
| Eine Statuszeile zeigt `argvus-gpg: pinentry-gtk is not installed` | Installieren Sie das Paket `pinentry`, das `pinentry-gtk` bereitstellt. |
| Die Erzeugung stoppt mit einem `pinentry-program`-Fehler | `gpg-agent.conf` setzt ein Pinentry außerhalb des ARGVUS-Blocks. Entfernen Sie diese Zeile; ARGVUS schreibt dann sein eigenes. |
| **Pinentry** zeigt `Not set` | gpg-agent verwendet seinen integrierten Standard, bis der erste Schlüssel von der Seite erzeugt wird. |
| Die Seite sagt, dass die installierte `argvus-launcher` älter ist als das Control Center | Installieren Sie `argvus-launcher` neu: Führen Sie `make build` und dann `make install` in seinem Repository aus und öffnen Sie die Seite erneut. Die Schlüsselaktionen bleiben deaktiviert, bis die Versionen übereinstimmen. |
| Der Dateimanager öffnet sich, aber die Seite fragt nach einem Pfad | Der Dateimanager ist grafisch und kann die ausgewählte Datei nicht melden. Geben Sie den Pfad in das Feld ein. |
| **Öffentlichen Schlüssel kopieren** sagt, dass er nicht kopiert werden konnte | `wl-copy` ist nicht installiert oder die Wayland-Sitzung ist nicht verfügbar. Installiere `wl-clipboard`. |
| **Schlüssel bearbeiten ›** sagt, dass der Schlüssel nicht mehr im Schlüsselbund ist | Der Schlüsselbund hat sich geändert, seit die Seite geöffnet wurde. Gehe zurück zu **Schlüssel ›** und aktualisiere. |
| Eine Statuszeile zeigt `argvus-gpg: ...` | Der Launcher verweigerte die Aktion, und der Text gibt den Grund an, wie ein abgelaufenes Datum, eine bereits vorhandene Datei oder ein fehlendes `--yes`. |
| Eine Aktion wartet und es passiert nichts | Das Pinentry-Fenster ist verborgen oder wurde nicht geöffnet. Überprüfen Sie, ob die Sitzung `WAYLAND_DISPLAY` oder `DISPLAY` freigibt. |
| Der Import wird verweigert | Die Datei enthält keinen Schlüssel der gewählten Art. Öffentliche Schlüssel werden über **Öffentlichen Schlüssel importieren…**; private Schlüssel über **Privaten Schlüssel importieren…** importiert. |

### Sprache & Region

Diese Karte listet Sprache (das regionale Gebietsschema), Zeitzone, Datum & Uhrzeit, System-Gebietsschemas und Tastaturlayout auf, und jedes davon lässt sich direkt vom Startbildschirm aus öffnen. Die Tastaturseiten enthalten auch die Variante und das Konsolentastaturlayout. Die Startbildsuchen zeigen außerdem Tastenkombinationen und Maus-/Touchpad-Einstellungen als direkte Einstellungsrouten an.

Die Sprachliste erkennt installierte Kataloge und zeigt die einheimischen Namen für alle zehn Sprachen. Aktivieren Sie das entsprechende UTF-8-Systemgebietsschema, bevor Sie es auswählen.

Sowohl **Preferences > Language & Region > Language** als auch **Settings > Language** speichern dieselbe Benutzereinstellung: `LANG` in `$XDG_CONFIG_HOME/locale.conf` (Standard `~/.config/locale.conf`), zum Beispiel `LANG=en_US.UTF-8`. Kommentare und regionale Variablen werden beibehalten. Keine globale Datei oder parallele ARGVUS-Spracheneinstellung wird geschrieben. Ein nicht konfigurierter Benutzer erhält Englisch; alte `session.language`- und `data/language`-Dateien werden ignoriert. Melden Sie sich ab und wieder an, um die Sprache auf die gesamte Sitzung anzuwenden und Anwendungen.

**System-Lokale** ändern `/etc/locale.gen` und führen `locale-gen` aus, daher erfordert deren Anwendung Administratorberechtigung. Eine dauerhafte **[WARN]**-Meldung über der Fußzeile kennzeichnet Seiten mit administrativen Einstellungen. Dieselbe Meldung gilt für Zeitzone, Datum/Uhrzeit, Hostname, Konsolen-Tastaturbelegung, Benutzer- und Gruppenbearbeitung, Firewall, Systemdienste, CPU-Governor, Energieprofile und -richtlinien, Boot-Konfiguration, Paketoperationen, Snapshots und geschützte Netzwerkänderungen. Persönliche Einstellungen, Benutzer Dienste und Nur-Lese-Seiten erhalten diese Benachrichtigung nicht. Die Benachrichtigung ist informativ; eine Autorisierung wird nur angefordert, wenn eine Handlung dies erfordert.

### Hardware und Displays

Wenn die entsprechenden Funktionen installiert sind, kann das Kontrollzentrum Hardwareinformationen wie CPU, GPU, Speicher, Stromversorgung und Geräte anzeigen. Die Eingabeseite konfiguriert das Verhalten von Maus und Touchpad; der Anzeigenbereich verwaltet Monitorauflösung, Bildwiederholrate, Skalierung, Position, Ausrichtung, Hauptanzeige, VRR und HDR, soweit von der laufenden Sitzung unterstützt. Anzeigeänderungen werden sofort angewendet. Nach einer Änderung, die den Bildschirm unbrauchbar machen kann (Auflösung oder Bildwiederholrate, Position, Spiegelung, Deaktivieren eines Monitors, Farbtiefe) oder nach **Übernehmen** fragt das Kontrollzentrum, ob es beibehalten werden soll, wobei der Fokus auf **Zurücksetzen** liegt: Drücken Sie `y`, um es beizubehalten; `n`, `Esc` oder `Enter` setzen es zurück, und ohne eine Antwort innerhalb von 15 Sekunden wird die vorherige Konfiguration automatisch wiederhergestellt. Das Entfernen der gespeicherten Konfiguration eines getrennten Monitors und das Löschen eines Profils erfordert eine Bestätigung; Profile öffnen ihre eigene Seite mit **Profil übernehmen**, **Umbenennen** und **Profil löschen**. Das Layout wird gespeichert durch `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` und `monitor-remove`, sodass der kanonische `/displays` Abschnitt von `config.json` die einzige Quelle des Monitorlayouts ist. Nur der Profilkatalog bleibt in `display/config.toml`.

### Konnektivität und Audio

Der Netzwerkbereich zeigt den Status und die Verbindungsseiten des installierten Netzwerkdienstanbieters an, einschließlich WLAN, Ethernet, VPN, DNS- und Proxy-Routen, sofern verfügbar. Die Firewall wird unter **Systemverwaltung** aufgeführt. Bluetooth ist abhängig von der Unterstützung durch Bluetooth. Der Audiobereich wird bereitgestellt, wenn seine optionale Funktion aktiviert ist, und integriert sich in den installierten Audiodienst.

Die **Netzwerk & Wi-Fi**-Startseite und ihre **Status**-Seite werden ausgefüllt, sobald jede NetworkManager-Abfrage beantwortet wird, anstatt nach der langsamsten: zuerst die Geräteliste, dann das Funkmodul, die Adressen und DNS der verbundenen Schnittstellen, die gespeicherten Verbindungen und die Wi-Fi-Netzwerke, die NetworkManager bereits kennt (ein neuer Scan wird nur von der Wi-Fi-Seite mit `r` gestartet). Bis die Geräteliste eintrifft, zeigen die Zeilen *Wird geladen...* an, und der Wi-Fi-Schalter wird nur angeboten, wenn der Computer über eine Wi-Fi-Karte verfügt. Das Starten einer Aktion (der Wi-Fi-Schalter, verbinden, DNS) während eine Seite aktualisiert wird, ersetzt das Aktualisieren, und die Seite aktualisiert sich erneut, wenn die Aktion endet.

#### Verbindungen, inspiriert von nmtui

**Netzwerk & Wi-Fi > Verbindungen** listet jedes NetworkManager-Verbindungsprofil auf, gruppiert nach Typ, mit dem Gerät, auf dem es aktiv ist (`★ Active · eno1`), ob es ein Port eines Bridges, Bonds oder Teams ist und ob es schreibgeschützt ist. `Enter` öffnet das Profil im Editor, `a` verbindet oder trennt es, `/` filtert nach Name und `r` aktualisiert. **Eine Verbindung aktivieren** ist dieselbe Liste, bei der `Enter` ein Profil hoch- oder runterfährt, wie in `nmtui`. Die Detailseiten einer Schnittstelle, eines gespeicherten Wi-Fi-Netzwerks und ein VPN haben eine **Verbindung bearbeiten**-Zeile, die ihr Profil öffnet.

**Verbindung hinzufügen** erstellt Ethernet-, Wi-Fi-, Hotspot-, DSL-(PPPoE)-, Bridge-, Bond-, Team-, VLAN-, IP-Tunnel-, Loopback- und WireGuard-Profile sowie ein VPN-Profil pro installiertem NetworkManager-VPN-Plugin (OpenVPN hat sein eigenes Formular; andere Plugins bearbeiten ihre Einstellungen als `key=value`-Einträge). Es importiert auch eine WireGuard `.conf` oder eine Plugin-Datei wie eine `.ovpn` (`nmcli connection import`). Team benötigt das `libteam`-Paket, DSL benötigt `ppp` und ein VPN benötigt sein Plugin (zum Beispiel `networkmanager-openvpn`); Ohne sie ist die Zeile deaktiviert und gibt an, was installiert werden soll. WEP, InfiniBand, MACsec und Open vSwitch werden nicht angeboten.

Der Editor ist eine Liste von einklappbaren Abschnitten: **Allgemein** (Name, Oberfläche, automatisch verbinden, Priorität, für alle Benutzer verfügbar, gemessen), die eigenen Einstellungen des Typs, **IPv4** und **IPv6** (Methode, Adressen, Gateway, DNS-Server, Suchdomänen, Routen, automatische DNS oder Routen ignorieren, niemals als Standardroute verwenden, Routemetrik, Datenschutzerweiterungen, Adressgenerierung). Auswahlen und Zahlen ändern sich mit `←/→`; `Enter` gibt einen Wert ein; Listen (Adressen, DNS, Routen, erlaubte IPs) öffnen ihre eigenen. Seite, auf der `Enter` einen Eintrag bearbeitet; das Löschen entfernt ihn, und **Eintrag hinzufügen** fügt einen hinzu. Felder, die für die gewählte Methode nicht zutreffen, werden deaktiviert angezeigt. Eine Bridge, ein Bond oder ein Team wählt seine Ethernet-**Ports** auf einer Unterseite; WireGuard bearbeitet seine **Peers** auf einer Unterseite und kann ein **Schlüsselpaar erzeugen** (mit `wg`, aus `wireguard-tools`, einer Abhängigkeit des `argvus`-Pakets); der öffentliche Schlüssel des privaten Schlüssels wird darunter angezeigt.

Nichts erreicht NetworkManager, bis auf **Speichern**, das deaktiviert bleibt, solange sich nichts geändert hat oder ein Feld ungültig ist, und dann zeigt, welches Feld und warum. Es werden nur die geänderten Einstellungen geschrieben. Das Speichern einer aktiven Verbindung fragt zuerst und verbindet sie dann wieder, sodass die neuen Einstellungen sofort wirksam werden (dies kann das Netzwerk kurz unterbrechen). `Esc` mit nicht gespeicherten Änderungen fragt, bevor sie verworfen werden. **Verbindung löschen**, in der Gefahrenzone, fordert zur Bestätigung auf, wobei der Fokus auf Abbrechen liegt; das Löschen Eine Brücke, ein Bond oder Team löscht auch seine Ports.

Passwörter, private Schlüssel und Preshared Keys werden niemals zurückgelesen: Ihre Zeilen zeigen *Unverändert*, bis ein neuer Wert eingegeben wird, und der neue Wert wird maskiert angezeigt. Sie werden über die Standardeingabe von `nmcli connection edit` geschrieben, niemals als Befehlsargumente. Änderungen werden als Sitzungsbenutzer ausgeführt: lokale Mitglieder von `wheel` sind durch die Polkit-Regel von NetworkManager erlaubt, und andere Benutzer werden vom Polkit-Agenten der Sitzung nach dem Administratorpasswort gefragt.

### Leistung und Sitzung

Strom liefert die Systemleistungssteuerungen und Richtlinienseiten, die für die Installation verfügbar sind, einschließlich des Umschalters „Keep Awake“. Wenn aktiviert, verhindert Keep Awake, dass die ARGVUS-Leerlaufrichtlinie automatische Bildschirmsperr- und Display-Stromsparer-Timer startet. Sitzung zeigt den ARGVUS-Sitzungsstatus, Komponenten, Autostart, Diagnosen und Protokolle an. Unter **Strom** enthält die **Aktionen**-Leiste unter der Liste **Suspendieren**, **Ruhezustand** und **Aktualisieren** (`r` aktualisiert ebenfalls): `Tab` oder `←/→` wechselt dorthin, `Enter` führt das fokussierte aus. Schaltfläche, `↑`/`↓` bringt den Fokus zurück zur Liste, und `Esc` geht zurück. Suspendieren und Ruhezustand fragen zuerst nach einer Bestätigung. Diese Seiten können Systemberechtigungen erfordern und sollten von den Schnellaktionen im Bedienfeld unterschieden werden.

### Systemwerkzeuge

Der Systembereich kann Startinformationen, Pakete, Dienste, Speicher, Diagnosen, Benutzer-/Gruppenverwaltung und Systeminformationen anzeigen. Dies sind administrative oder diagnostische Werkzeuge; eine Seite kann schreibgeschützt sein oder eine Autorisierung erfordern, abhängig von der Operation.

**Dienste** hat vier Seiten: Systemeinheiten, Benutzereinheiten, fehlgeschlagene Einheiten und das Journal (Protokolle). Jede Einheitenliste zeigt eine `Filter:`-Zeile, die `f` wechselt, und ein umrahmtes `Search:`-Feld, das mit `/` geöffnet wird. Jede Einheit zeigt ihren aktiven und aktivierten Status als gefüllte oder leere Punkte an. In den Protokollen wechselt `b` zwischen dem aktuellen und dem vorherigen Start, `u` wechselt die Einheit und `p` wechselt die Priorität; die `Filter:`-Zeile zeigt die verwendeten Werte. `Enter` öffnet eine Einheit oder einen Protokolleintrag.

**Dev Services** meldet drei entwicklungsorientierte Quellen auf einer schreibgeschützten Seite: die systemd-Dienste des aktuellen Benutzers, die aktuell lauschenden TCP-Ports (`ss`) und die laufenden Container eines installierten Container-Runtimes (Podman, Docker). Sie erscheint nur im Home-Raster, wenn mindestens eine dieser Quellen auf dem System verfügbar ist — ein Rechner ohne systemd-Benutzersitzung, ohne Podman und ohne Docker zeigt den Eintrag nicht an. `r` aktualisiert die drei Quellen unabhängig voneinander, sodass eine langsame oder fehlerhafte Quelle (für Ein Beispiel: eine Container-Laufzeit, die aufgehört hat zu reagieren, blockiert die anderen nicht daran, ihre Daten anzuzeigen.

**Pakete** sind nach Quelle gruppiert. **Offizielle Repositories** und, wenn ein AUR-Helfer (`paru` oder `yay`) installiert ist, **AUR (Arch User Repository)** öffnen jeweils **Installieren**, **Entfernen** und **Aktualisieren**; **Cache**, **Verlauf**, **Downgrade** und **Spiegel** folgen. Die drei Aufgaben teilen sich einen Bildschirm: das **Suchfeld**, oben fixiert, nimmt die von Ihnen eingegebenen Buchstaben auf, und die Pakete werden darunter im **Ergebnis**-Block aufgelistet (**Updates** bei Aktualisieren), wobei eine leere Liste den Grund angibt (zum Beispiel **Keine** Ergebnisse**, **Keine passenden Elemente** oder **Keine ausstehenden Updates**). **Installieren** durchsucht die Repositorien (`pacman -Ss`) oder das AUR, wenn Sie `Enter` in der oberen Suchleiste drücken; **Entfernen** listet die installierten Pakete auf (die AUR-Liste zeigt die außerhalb der Repositorien erstellten Pakete, `pacman -Qm`, und während das AUR-Menü angezeigt wird, verlassen sie die offizielle Liste); **Aktualisieren** listet die ausstehenden Updates auf (`pacman -Qu` oder `paru -Qua`/`yay -Qua` für das AUR). Bei einem Paket führt `Enter` die Aufgabe des Bildschirms aus und `→` öffnet die Paketseite, mit **Installieren** (oder **Neu installieren**) und in seiner **Gefahrenzone** **Entfernen**; `←` oder `Esc` gehen zurück. **Offizielle Repositories → Entfernen** beginnt mit **Waisen**, wo `Space` Pakete markiert (`[x]`), `Enter` öffnet eines und **Markierte entfernen** entfernt sie zusammen. Aus der Update-Ergebnisliste `Tab` bewegt sich in die **Aktionen**-Leiste unter der Liste (**Alle aktualisieren** und **Datenbank aktualisieren**, oder **Alle aktualisieren (AUR)** im AUR), `←`/`→` bewegen sich zwischen seinen Schaltflächen, `Enter` führt aus der fokussierte und `↑`/`↓` (oder Tippen) bringen den Fokus zurück zur Liste; **Alle aktualisieren** ist ausgegraut und wird übersprungen, solange kein Update aufgeführt ist. Die Breadcrumbs und die globale Suche folgen demselben Pfad, zum Beispiel **Pakete > Offizielle Repositories > Entfernen > Waisen**. Die Cache-Bereinigungen befinden sich in der Gefahrenzone der Cache-Seite, und **Spiegel → Spiegel konfigurieren** legt die Reflektor-Optionen fest, bevor **Vorschau generieren**. Jede Paketoperation zeigt ihren Plan an, oder für das AUR die PKGBUILD-Warnung. und fragt vor der Ausführung um Bestätigung (`y` bestätigt, `n` oder `Esc` bricht ab), und seine Ausgabe wird während der Ausführung angezeigt. AUR-Operationen führen den Helfer als Ihr Benutzer aus (`-S <package>`, um ein Paket zu installieren oder zu aktualisieren, `-Sua`, um alle zu aktualisieren); AUR-Entfernungen verwenden denselben `pacman -Rns` wie die offiziellen Pakete. Wenn andere installierte Pakete von dem Paket abhängen, das Sie entfernen (offiziell oder AUR), nennt die Bestätigung sie und bietet zwei Möglichkeiten: **Nur entfernen** das ausgewählte Paket (`pacman -Rdd`, das überspringt das Abhängigkeitsprüfungen, sodass die Pakete, die es benötigen, möglicherweise aufhören zu funktionieren) oder **Mit Abhängigkeiten entfernen** (`pacman -Rcns`, mit der Anzahl der Pakete, die entfernt werden). Der Fokus beginnt auf **Abbrechen**, was jedes Paket behält; `Enter` führt die fokussierte Zeile aus und es gibt in dieser Auswahl keine `y`-Verknüpfung.

Die obere **Such**leiste besitzt die Paketabfrage. `/` fokussiert sie, ohne ihren Text zu löschen; das Tippen verschiebt ebenfalls den Fokus dorthin. Bearbeite mit den Pfeiltasten, `Home`, `End`, `Backspace`, `Delete` oder füge ein. In **Installieren** sendet `Enter` eine Repository-/AUR-Abfrage, selbst wenn vorherige Ergebnisse vorhanden sind, und verschiebt dann den Fokus zu den Ergebnissen; offizielle Suchen erfordern eine nicht-leere Abfrage und AUR-Suchen mindestens zwei Zeichen. Das Bearbeiten einer Installationsabfrage bricht die ausstehende Anfrage ab und löscht ihre alten Ergebnisse; eine neue Die Anfrage wartet auf `Enter`. In **Entfernen**, **Aktualisieren** und **Verwaiste** filtert die Bearbeitung die geladene Liste sofort, ohne eine weitere Repository-Abfrage. `↓` wechselt von Suche zu Ergebnissen, `↑` beim ersten Ergebnis kehrt zur Suche zurück, und `Esc` in der Suche kehrt zur Liste zurück, während die Abfrage beibehalten wird. `Enter` bei einem Ergebnis führt die Aktion dieses Pakets aus. Die Aktionsleiste Aktualisieren bleibt über `Tab` erreichbar, und `/` kehrt von dort zur Suche zurück.

Alle Operationen in **Pakete** öffnen ein integriertes interaktives Terminal: offizielle Pakete, AUR, Cache-Bereinigung, Spiegel und Änderungen von Repository/Schlüssel. Beantworten Sie die Aufforderungen des Programms, indem Sie tippen und `Enter` drücken; pacman läuft ohne `--noconfirm` in diesem Terminal. `Esc` blendet die Ansicht aus und hält den Prozess am Laufen. `F8` öffnet dieselbe Sitzung von jeder Pakete-Seite erneut; **Terminal anzeigen** ist ebenfalls in den Menüs verfügbar. Die Fußzeile zeigt an, ob die Transaktion läuft, abgeschlossen oder fehlgeschlagen ist. Nach einem Erfolgreicher Befehl, `Done!` erscheint im Terminal nach seiner endgültigen Ausgabe, auch wenn das Terminal ausgeblendet war. `Shift+PageUp` / `Shift+PageDown` blättern durch die Historie, `Shift+End` kehrt zur Live-Ausgabe zurück und `Ctrl+C` sendet einen Interrupt an den Kindprozess. Während das Terminal fokussiert ist, werden `q` und `?` als Eingabe und nicht als globale Tastenkombinationen behandelt. Ausgabe, ANSI-Farben und der Cursor werden beibehalten, und das Ändern der Größe aktualisiert das Kindterminal. Die Historie speichert bis zu 10.000 Zeilen bis zum nächsten Operation oder die Anwendung wird beendet. Es läuft immer nur eine Transaktion gleichzeitig; das Verstecken ihres Terminals erlaubt der Anwendung nicht, während der Ausführung zu beenden. Es wird keine freie Shell geöffnet.

Die Upgrade-Vorschau liest die bestehenden Sync-Datenbanken (`pacman -Su --print`), ohne sie zu ändern oder Root zu benötigen. Nach der Bestätigung führt das vollständige Upgrade `pacman -Syyu` über den privilegierten Helfer mit `pkexec` aus; die Synchronisation kann zusätzliche Updates anzeigen. Paketinstallation, Entfernung, Datenbankaktualisierung, Cache-Bereinigung, Downgrade, Repository-/Schlüsseländerungen und Mirror-Schreibvorgänge nutzen ebenfalls diesen privilegierten Helfer. AUR-Helfer bauen weiterhin als Ihr Benutzer.

**Offizielle Repositories → Repositories** fügt inoffizielle pacman-Repositories hinzu und verwaltet sie. **Hinzufügen** ist ein Entwurf: Wählen Sie die **Quelle** der Konfiguration (**URL** einer `.conf`-Datei, **Text**, der in einen Mehrzeilen-Editor eingegeben oder eingefügt wird, oder **Formular** mit **Name**, **Server** und **SigLevel**, geschrieben im Arch-Wiki-Layout) und unter **Signaturschlüssel** einen **Schlüssel** von einer **URL** oder **Text** (oder **Keine**). Die Auswahlfelder bewegen sich mit `←`/`→` oder `Enter`; Textfelder öffnen einen Editor, in dem `Enter` eine einzelne Zeile speichert. `Ctrl+S` spart mehrere Zeilen, Einfügen funktioniert und `Esc` bricht ab. Downloads akzeptieren nur `https://`-Adressen. **Überprüfen und hinzufügen** lädt herunter und prüft alles als Ihr Benutzer und fragt dann nach Bestätigung, listet die Repositories, Server und SigLevel sowie die Benutzer-ID und den Fingerabdruck des Schlüssels in Vierergruppen auf, um sie mit dem vom Repository veröffentlichten zu vergleichen. Bestätigung importiert den Schlüssel und signiert ihn lokal (`pacman-key --add`, `--lsign-key`), schreibt `/etc/pacman.d/<name>.conf` (benannt nach seinem erste Abschnitt) und fügt `Include = /etc/pacman.d/<name>.conf` zu `/etc/pacman.conf` hinzu (eine bestehende auskommentierte Zeile für dieselbe Datei wird stattdessen aktiviert). Danach bietet das Control Center `pacman -Syyu` an, sodass die neuen Datenbanken ohne ein partielles Upgrade synchronisiert werden. Das Verlassen von **Hinzufügen** mit einem Entwurf fragt, bevor es verworfen wird. **Liste** zeigt jedes inoffizielle Repository, das in `pacman.conf` selbst geschrieben oder von `/etc/pacman.d/` übernommen wurde, aktiviert oder deaktiviert; offizielle Abschnitte werden nie aufgelistet. Ein Die Repository-Seite zeigt ihre Datei, SigLevel, Server und den aufgezeichneten Schlüssel, einen **Aktiviert**-Schalter und in der **Gefahrenzone** **Entfernen**. Deaktivieren kommentiert die `Include`-Zeile aus oder fügt jedem Zeilenanfang eines Inline-Abschnitts `#argvus-off ` hinzu; Entfernen löscht die Zeile oder den Abschnitt sowie die eingebundene Datei. Repositories derselben eingebundenen Datei werden zusammen aktiviert, deaktiviert und entfernt. Wenn das Control Center den Schlüssel importiert hat, bietet **Entfernen** auch die Möglichkeit, ihn aus dem pacman-Schlüsselbund (`pacman-key --delete`) zu entfernen. --delete`/etc/pacman.conf`/etc/pacman.conf`pacman.conf.argvus-backup-…`pacman.conf.argvus-Backup-…`), und installierte Pakete werden niemals entfernt.

### Benutzer und Gruppen

Wenn der Kontenanbieter verfügbar ist, öffnen Sie **Systemsteuerung → Systemverwaltung → Benutzer & Gruppen → Benutzer** oder suchen Sie nach **Benutzer**. Die Schnittstelle kann normale und Systemkonten auflisten, einen Benutzer erstellen, Kontometadaten bearbeiten, zusätzliche und primäre Gruppen verwalten, ein Passwort ändern oder sperren, es entsperren, eine Passwortänderung beim nächsten Anmelden erzwingen, ein Avatar festlegen oder entfernen und das automatische Anmelden aktivieren.

Die Benutzererstellung verlangt nach einem Benutzernamen, vollständigen Namen, Shell, zusätzlichen Gruppen und einer optionalen Passwortbestätigung. Ein leeres Passwort erstellt das Konto mit gesperrtem Passwort; es wird nicht stillschweigend ein nutzbares Passwort erstellt. Eine Benutzerseite gruppiert die Kontofelder (vollständiger Name, Shell, primäre und zusätzliche Gruppen, gefolgt von **Änderungen speichern**), den **Automatischen Login**-Schalter, die **Passwort**-Aktionen (ändern, sperren, entsperren, Passwortänderung beim nächsten Login erforderlich) und die **Avatar**-Aktionen. Benutzername, UID / GID und das Home-Verzeichnis werden nur zur Information angezeigt. Die **Gefahrenzone** am Ende enthält die separaten zerstörerischen Aktionen, um das Konto zu löschen, während das Home-Verzeichnis erhalten bleibt, oder um das Home-Verzeichnis ebenfalls zu löschen. Lesen Sie den Bestätigungstext sorgfältig, bevor Sie letzteres wählen. Kontovorgänge erfordern Berechtigungen und können die Systemautorisierungsaufforderung öffnen. Avatare können vom Begrüßungsprogramm über die Standard-Gesichtsbildintegration des Kontos verwendet werden.

**Automatische Anmeldung** startet die Sitzung des ausgewählten Kontos beim nächsten Start, ohne den Anmeldebildschirm anzuzeigen, nach demselben Prinzip, das GNOME, LightDM und SDDM für diese Funktion verwenden: Es wird kein Passwort gespeichert. Wenn sie für ein Konto aktiviert wird, wird sie für zuvor konfigurierte Konten deaktiviert (nur ein Konto kann gleichzeitig automatisch angemeldet werden). Sie gilt einmalig beim nächsten Start; nach einer manuellen Abmeldung erscheint der normale Anmeldebildschirm für den Rest dieser Sitzung. Die Umschaltung wird sofort übernommen, mit einer Bestätigung. Eingabeaufforderung und ist nur für Administratoren verfügbar.

Gruppen haben ihre eigenen Listen- und Bearbeitungsseiten, einschließlich Mitgliederverwaltung und Gruppendeletion. Das Control Center ersetzt nicht die Kontorichtlinie des Systems: Geschützte Systemkonten und Vorgänge, die eine Autorisierung erfordern, unterliegen weiterhin den Einschränkungen des Betriebssystems.

### Einstellungen des Control Centers

**Einstellungen** ist der letzte Eintrag des Homes. Die **Sprache**-Seite legt die Benutzersprache über locale.conf fest, und die **Stil**-Seite steuert die dekorativen Symbole und die Transparenz des Kontrollzentrums. Stil enthält keinen Blur-Effekt: Blur ist eine globale Hyprland-Einstellung unter Hyprland > Blur. Die Stil-Seite ändert die Darstellung der Einstellungsanwendung; sie ist kein Schalter, der ARGVUS-Dienste deaktiviert oder Einstellungsanbieter entfernt. Das Speichern der Symbole kann eine Autorisierung erfordern, während die visuelle Änderung auf die aktuelle Anwendung angewendet. Esc auf Stil kehrt zur Einstellungsseite zurück, und Esc dort kehrt zur Startseite zurück.

### Über ARGVUS

**Über ARGVUS** befindet sich in **Tools & Diagnostics → Über ARGVUS** (suche `about`, `hardware` oder `firmware`, oder führe `argvus control-center about` aus). Es hat fünf Registerkarten, die mit `Tab`/`Shift+Tab`, `←`/`→` oder `h`/`l` gewechselt werden: **System & Hardware**, **Über**, **Spenden**, **Credits** und **Copyright**. Die Kopfzeile zeigt den vollständigen Pfad, zum Beispiel `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**System & Hardware** ist schreibgeschützt und gruppiert die Fakten dieser Maschine in drei Abschnitte, wobei die Werte in einer Spalte ausgerichtet sind:

| Abschnitt | Felder |
| --- | --- |
| Boot | Firmware (Hersteller, Version und Datum), Typ (UEFI oder Legacy (BIOS)), Secure Boot (nur UEFI), Virtualisierung, Bootloader |
| System | ARGVUS, Hostname, Betriebssystem, Distributor, Kernel, Architektur, Fenstermanager (Hyprland), Kompositor (Wayland), GPU-Treiber, GTK, Thema (mit Dunkel- oder Hellmodus), Anzeigen (eine Zeile pro Monitor), Audio (PipeWire), Shell (Quickshell), Anmeldung (greetd), Init (systemd), Pakete, Betriebszeit, Gebietsschema |
| Hardware | Akku (eine Zeile pro Akku), CPU (mit ihrer maximalen Frequenz), Threads, Kerne, GPU, Speicher (genutzt / insgesamt), Swap, Speicherplatz, Modell (nur wenn es sich vom Motherboard unterscheidet), Motherboard |

Jeder Wert wird vom laufenden System ausgelesen, ohne Privilegien, und die Seite funktioniert auf jedem Rechner gleich. Ein Wert, der nicht ausgelesen werden kann, wird als `N/A` angezeigt, und eine Zeile, die nicht zutrifft, wird weggelassen:

- **Firmware**, **Modell** und **Motherboard** stammen von DMI (`/sys/class/dmi/id`); Platzhalterwerte wie `To be filled by O.E.M.` werden ignoriert. **Typ** und **Secure Boot** stammen von `/sys/firmware/efi`, und **Virtualisierung** von `systemd-detect-virt`.
- **Bootloader** wird aus der EFI-Variable `LoaderInfo` gelesen, die von systemd-boot, Limine und anderen Bootloadern, die dem Boot Loader Interface folgen, veröffentlicht wird. GRUB veröffentlicht sie nicht, daher wird sie von `/boot/grub` (oder `/efi/grub`) erkannt und zusammen mit der Version des `grub`-Pakets angezeigt.
- Die Versionen von ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd und GRUB stammen aus einer `pacman -Q`-Abfrage und werden ohne Epoch und Paketversion angezeigt (`1:25.2.3-1` wird zu `25.2.3`). Die Hyprland-Version stammt aus `hyprctl version` oder aus dem Paket, wenn keine Sitzung antwortet.
- **GPU-Treiber** nennt den Treiber, der an jede DRM-Karte in `/sys/class/drm` gebunden ist. Ein proprietärer Treiber zeigt seine Modulversion an (zum Beispiel `nvidia 580.82.07`); ein Kernel-integrierter Treiber zeigt die Mesa-Version an (zum Beispiel `amdgpu (Mesa 25.2.3)`).
- **Anzeige** listet die aktiven Monitore von `hyprctl monitors -j` auf: Anschluss, Auflösung, Bildwiederholrate und Skalierung (die Skalierung wird bei 1 weggelassen).
- **Pakete** zählt alle installierten Pakete, die explizit installierten und die fremden (AUR oder lokal gebaut), von `pacman -Qq`, `-Qqe` und `-Qqm`.
- **Betriebszeit** kommt von `/proc/uptime` und ist der Wert, als die Seite geöffnet wurde. **Gebietsschema** kommt von `LANG` in `/etc/locale.conf`.
- **Batterie** listet die Batterien des Computers von `/sys/class/power_supply` auf, ausgewählt nach Typ, sodass Peripheriegeräte wie kabellose Mäuse nicht gezählt werden. Jede Zeile zeigt die Ladung, den Zustand, die Gesundheit (volle Kapazität im Vergleich zur Designkapazität) und die Ladezyklen, wenn diese vom Firmware gemeldet werden. Ein Gerät ohne Batterie zeigt **Nicht erkannt** an.
- **CPU** fügt die höchste maximale Frequenz aller Kerne (cpufreq) hinzu, was die angegebene Maximalfrequenz bei Hybridprozessoren ist; sie wird weggelassen, wenn cpufreq nicht verfügbar ist, wie in den meisten virtuellen Maschinen. **Threads** und **Kerne** stammen von `/proc/cpuinfo`.
- **Speicher** und **Swap** stammen von `/proc/meminfo` (zram eingeschlossen); ein System ohne Swap zeigt **Deaktiviert**.
- **Speicher** beschreibt die Festplatte unter `/`: ihr Modell, ihre Art (NVMe, SSD, HDD oder eMMC/SD; virtuelle Festplatten zeigen keine Art), das Dateisystem sowie den belegten und den Gesamtspeicher. Die Festplatte wird über Partitionen, LUKS und LVM bis zum physischen Gerät gefunden.

Die Werte werden im Hintergrund beim ersten Zeichnen der Seite gelesen, sodass weder der Start des Kontrollzentrums noch das Öffnen der Seite auf sie warten; der Tab zeigt **Systeminformationen werden geladen...** in der Zwischenzeit an. Drücken Sie `c` auf diesem Tab, um den gesamten Bericht als Klartext in die Zwischenablage zu kopieren (mit `wl-copy`), zum Beispiel um ihn in einem Fehlerbericht einzufügen.

## Suche und optionale Funktionen

Der Startbildschirm verfügt über ein durchsuchbares Verzeichnis von Routen. Suchbegriffe umfassen benutzerorientierte Konzepte wie Erscheinungsbild, Themen, Hintergrundbilder, Schriftarten, Tastenkombinationen, Maus, Touchpad, Anzeigen, Netzwerk und Sitzung. Optionale Anbieter werden nur einbezogen, wenn ihre Funktion kompiliert ist und ihre Laufzeitfähigkeit erkannt wird, sodass das sichtbare Menü bewusst fähigkeitsbewusst ist.

Während die Homesuche aktiv ist, werden druckbare Tasten in die Abfrage eingegeben, einschließlich `j` und `k`; verwenden Sie die Pfeiltasten `↑` und `↓`, um durch passende Routen zu navigieren. Drücken Sie `Enter`, um die ausgewählte Route zu öffnen, `Backspace`, um die Abfrage zu bearbeiten, und `Esc`, um die Abfrage zu löschen oder den Suchmodus zu verlassen.

## Erscheinungsbild-Workflow

Öffnen Sie **Erscheinungsbild**, um die integrierten visuellen Steuerungen zu sehen. Dessen Gruppen sind:

- **Desktop** — **Thema** (wählen Sie eine gebündelte Familie und ihren Sticky- oder Spaced-Modus, oder exportieren, importieren, anwenden und löschen Sie eigene Themen, siehe [Importieren und Exportieren von Themen](/docs/components/argvus-themes/import-and-export/)), **Aussehensmodus** (Sticky oder Spaced, unabhängig vom Thema), **Akzentfarbe** (ein gebündelter Akzent oder eine gültige sechsstellige RGB-Farbe), **Hintergrundbild** (ein gebündeltes Hintergrundbild oder eine benutzerdefinierte Datei) **Symbole** (ein installiertes Symbolthema: durchsuchen Sie die Liste mit `/`, wählen Sie eines mit Space oder Enter drücken, dann **Übernehmen** auswählen; es ändert sich nichts vor dem Übernehmen) und **Schriften** (die Schriftziele und Schriftwiedergabeeinstellungen, geöffnet auf der Seite Schriften).
- **Anwendungsstil** — die **Terminal**- und **Launcher**-Transparenz.

Die Karte 'Aussehen' enthält die Oberflächen, die diese Einstellungen verwenden: **Widget-Telemetrie** (aktivieren Sie die Telemetrie-Oberfläche, wählen Sie ihre verfügbaren Blöcke aus und ordnen Sie sie mit `Shift+Up`/`Shift+Down` auf einem fokussierten Block neu) und **Systemsteuerung** (aktivieren, deaktivieren und ordnen Sie Panel-Karten mit `Shift+Up`/`Shift+Down` auf einer fokussierten Karte neu, angewendet sofort über dasselbe Primitive, das das Drag-and-Drop des echten Panels verwendet).

Diese Aktionen aktualisieren den logischen ARGVUS-Zustand und wenden die betroffene Laufzeitkonfiguration an. Siehe [Appearance](/docs/user-guide/appearance/), [Themes](/docs/user-guide/appearance/) und [Windows and layout](/docs/argvus-hyprland/windows-and-layout/).

## Tastaturnutzung

Die Einstellungsanwendung unterstützt die Tastaturnavigation. Jede Seite ist eine einzelne Liste: Verwenden Sie die Pfeiltasten oder `j`/`k` zum Bewegen, `Enter` zum Öffnen oder Ausführen der ausgewählten Zeile, `Space` zum Umschalten, `Esc` zum Zurückgehen, `/` zum Durchsuchen von Listen und `?` für Hilfe; die Fußzeile zeigt die Tasten an, die für die ausgewählte Zeile gelten. Aktionen sind Zeilen der Liste, keine Tasten, und `Tab` bewegt sich nicht zu ihnen, außer auf den Seiten mit einer **Aktionsleiste** unter der Liste (Standard-Apps, Zeitzone, Tastaturlayout, System-Locale und SSH). Tasten und Verknüpfung erstellen/bearbeiten). Zeilen, die nur Informationen anzeigen, werden übersprungen. Solange ein Textfeld, eine Suche oder ein Listenfilter aktiv ist, werden `q` und `?` als Text eingegeben, anstatt die Anwendung zu beenden oder die Hilfe zu öffnen. Auf **Standard-Apps** ist **Standardeinstellungen wiederherstellen** die Schaltfläche in der **Aktionen**-Leiste unter der Liste: `Tab` oder `←/→` werden fokussiert und `Enter` fragt nach dem Zurücksetzen. **Standardwerte zurücksetzen** auf den anderen Seiten und **Alle Verknüpfungen wiederherstellen** sind Zeilen in einer **Gefahrenzone** am Ende ihrer Seiten; `r` fragt nach der derselbe Reset, und in der Liste der Tastenkombinationen stellt `r` die ausgewählte Verknüpfung nach einer Bestätigung wieder her. Unter **Sprache & Region → Tastaturlayout** fokussiert `/` das **Suchfeld** oben auf der Seite, und `Esc` löscht es; `●` markiert das verwendete Layout. Es folgt der Taskleiste und der Tastenkombination, die Layouts durchläuft, und die Seite wird innerhalb von etwa einer Sekunde aktualisiert, ohne etwas in der Konfiguration zu schreiben. `Enter` setzt das Standardlayout, das erste, mit dem Hyprland startet, und dasjenige, das gekennzeichnet ist `Default` im Detail; `Space` fügt ein Layout hinzu oder entfernt es (`[x]`). Diese Tasten ändern nur einen Entwurf, und die **Anwenden**-Schaltfläche in der **Aktionen**-Leiste unter der Liste schreibt die Layouts und die Standardkonfiguration in einem Schritt nach Hyprland. Aus der Liste verschiebt `Tab` oder `←/→` den Fokus auf diese Leiste, während **Anwenden** aktiviert ist (die Fußzeile zeigt dann `←/→ Actions` an); `←/→` navigiert durch ihre Schaltflächen und `↑/↓` kehrt zur Liste zurück. Beim Verlassen der Seite mit nicht angewandten Änderungen wird zuerst um Bestätigung gebeten. Unter **Sprache & Region → Zeitzone** Das Layout ist dasselbe: `/` fokussiert das **Suchfeld** oben, `Enter` wählt eine Zone in einem Entwurf aus (markiert mit `●`), und die **Übernehmen**-Schaltfläche in der **Aktionsleiste** unter der Liste speichert sie in der `environment.d/20-timezone.conf` des Benutzers; `Tab`, `←/→` und `↑/↓` funktionieren wie beim Tastaturlayout. Das Verlassen mit einer nicht angewendeten Zone fordert zuerst um Bestätigung. In **System-Lokale** ist das Layout dasselbe wie bei der Zeitzone: `/` fokussiert das **Suchfeld** oben, `Enter` oder `Space` markiert oder entmarkiert einen Ort (`[x]`) in einem Entwurf, und die **Übernehmen**-Schaltfläche in der **Aktionen**-Leiste unter der Liste schreibt die Auswahl in `/etc/locale.gen` und führt `locale-gen` aus. Da dadurch die Systemsprachen neu generiert werden, fragt **Übernehmen** zuerst nach einer Bestätigung; `Tab`, `←/→` und `↑/↓` funktionieren wie bei den anderen beiden. Unter **Desktop & Eingabe → Tastenkombinationen → Alle Kurzbefehle** befindet sich das Suchfeld ebenfalls oben, wobei `/` es fokussiert, aber die Seite keine **Aktionen**-Leiste hat: ihre Liste behält die Tastenkombination. Abschnitte und die **Gefahrenzone**, und der Footer listet `r Restore default` nur auf der Liste selbst auf, nicht unter **Verknüpfung bearbeiten**.

Seiten, die Änderungen sammeln, bevor sie gespeichert werden (ein Benutzer, eine Gruppe, die Kontoerstellung, Passwortänderung und Firewall-Konfiguration), beenden diesen Block mit einer **Speichern**- oder **Übernehmen**-Zeile, die gedimmt bleibt, bis sich etwas geändert hat. Wenn das Zurückgehen nicht gespeicherte Änderungen verwerfen würde, fragt das Control Center zuerst nach. Jede Bestätigung zeigt **Bestätigen** und **Abbrechen** als zwei Zeilen, wobei der Fokus auf **Abbrechen** liegt: `Enter` führt die fokussierte Zeile aus, `y` bestätigt und `n` oder `Esc` brechen ab.

In **Darstellung** ist jede Seite eine einzelne Liste: Die Fußzeile zeigt nur die Tasten, die für die ausgewählte Zeile gelten, und `Space` öffnet oder wählt ebenfalls die Zeile aus. Seiten, die Änderungen vor ihrer Anwendung sammeln (Taskleiste, Widget-Telemetrie, Systemsteuerung und die Effekt-Schieberegler), enden mit einer **Übernehmen**-Zeile, die gedimmt bleibt, bis sich etwas ändert. Die Auswahl von **Übernehmen** zeigt den Ladebildschirm nur, wenn die Änderung eine Oberfläche neu erstellt: eine Taskleistenhöhe, eine Dienstprogrammgruppe, ein Symbol oder ein Datums- und Zeitformat. startet die Taskleiste neu; das Ein- oder Ausschalten der Widget-Telemetrie oder das Ändern ihrer Blöcke oder Reihenfolge startet das Widget neu; das Ein- oder Ausschalten der Systemsteuerung oder das Ändern ihrer Karten, während sie eingeschaltet ist, startet das Panel neu. Der Bildschirm bleibt dann stehen, bis die letzte Änderung der Seite angewendet ist, sodass keine Änderung davor erscheint, und die Oberfläche wird einmal neu gestartet. Ränder, Transparenz, Unschärfe, Animationen, Abstände, Schriftarten, Hintergrundbild und die anderen Einstellungen werden direkt angewendet, ohne den Ladebildschirm. Auf Prozentreihen, `←`/`→` (oder `+`/`-`) ändern Sie den Wert in Schritten von 5 und `Enter` ermöglicht es Ihnen, ihn einzugeben; verwenden Sie `Esc`, um zurückzugehen. Wenn das Zurückgehen Änderungen verwerfen würde, die nicht angewendet wurden, fragt das Control Center zuerst. Das Löschen eines benutzerdefinierten Themas (`d`) oder das Ersetzen eines importierten Themas erfordert eine Bestätigung; der Fokus beginnt auf **Abbrechen**, `y` bestätigt und `n` oder `Esc` brechen ab.

## Persistenz und Zurücksetzen

Einstellungsseiten wenden Änderungen über ihren zugehörigen Anbieter an und speichern den unterstützten Benutzerstatus. Erscheinungsbildänderungen werden in der kanonischen Konfiguration gespeichert, und `argvus-config` ist die einzige Komponente, die die abgeleiteten Verbraucherdateien unter `~/.config/argvus/data/generated/` schreibt; diese generierten Dateien sind nicht der Ort, um eine dauerhafte Änderung vorzunehmen. Eingabe- und Tastenkombinationsseiten haben ihren eigenen gespeicherten Zustand und Rücksetzaktionen. Andere Systemänderungen können Berechtigungen oder einen Dienstneustart erfordern.

Es gibt keinen globalen Reset für jede ARGVUS-Einstellung. Stellen Sie eine Änderung von der Seite wieder her, die sie besitzt, oder verwenden Sie das dokumentierte Wiederherstellungsverfahren der entsprechenden Funktion.

Der Reset-Support ist bewusst pro Bereich vorgesehen:

- **Aussehen** bietet einen Reset für die Hervorhebungsfarbe auf den Standard des aktiven Themas. Es gibt keinen einzelnen globalen Aussehens-Reset; stellen Sie andere Aussehenswerte über ihre eigenen Steuerelemente wieder her.
- **Schriftarten** können alle Schriftarteinstellungen, ein Ziel oder eine einzelne Einstellung wiederherstellen.
- **Standardanwendungen** können alle Standardeinstellungen, eine Kategorie oder einen einzelnen Selektor wiederherstellen.
- **Tastenkombinationen** können eine Bindung oder alle Bindungen wiederherstellen und dann die generierten Sitzungsbindungen neu laden.
- **Fensterregeln** befinden sich in **Desktop & Eingabe → Hyprland → Fenster → Regeln** (suche `window rules` oder führe `argvus control-center window-rules` aus). Jede Regel hat eine **Arbeitsbereich**-Zeile (`Enter` oder `←/→` durchläuft 1 bis 10) und eine **Fensterklassen**-Zeile, die ein Textfeld mit durch Kommas getrennten regulären Ausdrücken öffnet. **Regel hinzufügen** erstellt `rule-N` mit Arbeitsbereich 1 und ohne Klassen. Das **Regel entfernen** jeder Regel befindet sich in der **Gefahrenzone** und fragt nach Bestätigung. Jede Änderung wird geschrieben in `hyprland.window_rules` in `argvus config` und lädt die Konfiguration sofort neu; ungültige Klassenmuster werden abgelehnt und als Fehler angezeigt.
- **Projekte** befindet sich unter **Tools & Diagnostics → Projekte** (suche `projects` oder führe `argvus control-center projects` aus). **Projektordner hinzufügen** und **Stammordner hinzufügen** befinden sich oben auf der Seite und öffnen ein Textfeld mit dem Pfad; der Launcher prüft, ob es sich um ein Verzeichnis handelt, und zeigt sonst einen Fehler an, einschließlich wenn der Ordner die effektive Projektanzahl über 9 hinaus erhöhen würde (`SUPER + CTRL + 1..9` kann nicht mehr adressieren). Jeder konfigurierte Ordner, der als Stamm markiert ist (seine Unterordner sind Projekte) oder ein Einzelnes Projekt, erhält seinen eigenen Block unter dem Formular mit einer **Entfernen**-Zeile, die um Bestätigung bittet. Die Seite ruft `argvus-projects` auf, sodass Änderungen vom Launcher geschrieben werden und beim nächsten Ausführen von `SUPER + O` oder `SUPER + CTRL + 1..9` wirksam werden. Wenn `argvus-projects` nicht installiert ist, zeigt die Seite dies an.
- **Andere Systemseiten** zeigen Zurücksetzen-, Anwenden-, Löschen- oder Wiederherstellen-Aktionen nur, wenn der zugrunde liegende Anbieter sie unterstützt.

Das Entfernen einer Benutzereinstellung kann dazu führen, dass ein Anbieter in seinen verpackten/Standardzustand zurückfällt, aber das manuelle Löschen von Dateien ist kein allgemeines Wiederherstellungsverfahren. Bevorzugen Sie die Rücksetzaktion der Seite oder den dokumentierten, funktionsspezifischen Befehl.

## CLI-Einstiegspunkte

Der Helfer ist `argvus-control-center`, ausgeführt über `argvus control-center`. Er kann direkt fokussierte Bereiche öffnen, zum Beispiel:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```
Führen Sie `argvus control-center --help` für die verfügbaren Routen in der installierten Version aus. Die Befehlszeilenroute ist eine Verknüpfung zur gleichen Einstellungsanwendung, kein separates Konfigurationssystem.

Nützliche direkte Routen sind `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (mit den Seiten `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` und `proxy`), `audio`, `bluetooth`, `boot`, `packages` (mit den Seiten `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` und `mirrors`), `storage`, `system-doctor` (`diagnostics` und `diagnostic` bleiben Aliase), `about` (mit den Tabs `system`, `about`, `donate`, `credits` und `copyright`), `power`, `session`, `displays` und `appearance`. Auch Appearance akzeptiert fokussierte Routen wie `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` und `widget-telemetry`; display akzeptiert `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` und `hdr`. Verwenden Sie die installierte `--help`-Ausgabe beim Skripten, da die Verfügbarkeit immer noch von der installierten Version abhängt.

## Verwandt

- [Erste Konfiguration](/docs/getting-started/)
- Systemsteuerung
- [Erscheinungsbild](/docs/user-guide/appearance/)
- [Tastenkombinationen](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Maus und Touchpad](/docs/argvus-hyprland/input/)

### Systemdoktor

Öffnen Sie **Tools & Diagnostics → System Doctor**, suchen Sie danach oder führen Sie `argvus control-center system-doctor` aus. Die vorherigen Routen `diagnostics` und `diagnostic` sowie die Kategorieselectionen bleiben verfügbar. Drücken Sie `r` zum Aktualisieren, Enter, um eine Prüfung zu inspizieren, `o` in einem Detail, um die zugehörige Seite zu öffnen, und Esc, um zurückzugehen. Die Sammlung läuft im Hintergrund und ist schreibgeschützt.

Prüfungen verwenden systemd fehlgeschlagene Einheiten-Zählungen, Grafiktools und den Storage-Snapshot für die Nutzung des Dateisystems und den SMART-Zustand. Ein Problem erfordert Beweise; nicht verfügbare Tools oder nicht zugängliche SMART-Daten werden als nicht verfügbar angezeigt, und eine fehlgeschlagene oder zeitüberschrittene Überprüfung ist ein Fehler. Kategorien ohne gesammelte Beweise bedeuten nicht, dass die Prüfung erfolgreich war. Paketaktualisierungen verwenden die lokale Pacman-Datenbank und synchronisieren keine Repositories; eine Sperrdatei kann veraltet sein und beweist nicht, dass eine Operation läuft. SMART kann erfordert Berechtigungen und es wird kein erhöhter Zugriff angefordert. Die Zusammenfassung zeichnet die zuletzt abgeschlossene Erfassung als Unix-Zeit auf.

Storage und System Doctor teilen einen In-Memory-Schnappschuss für bis zu 10 Sekunden. Das Drücken von `r` sammelt ausdrücklich neue Daten; gleichzeitige Anfragen verwenden eine Sammlung, die gerade abgeschlossen wurde. Die Ausgabe der Diagnostic- und Storage-Subprozesse ist auf 1 MiB pro Stream begrenzt, und zu große Ausgaben werden als fehlgeschlagene Sammlung behandelt.

## Nachtlicht

Öffnen Sie **Desktop & Eingabe > Anzeigen > Nachtlicht** oder führen Sie `argvus control-center displays night-light` aus. Konfigurieren Sie Temperatur, manuelle Aktivierung und feste lokale Zeitpläne über `argvus config display night-light`. Siehe [Nachtlicht](/docs/user-guide/night-light/) für Befehle, Status und Laufzeitanforderungen.

## Schnappschuss

Öffnen Sie **Systemsteuerung > Systemverwaltung > Snapshot**, suchen Sie nach `Snapper` oder führen Sie `argvus control-center snapshot` aus. Die `snapshot` Cargo-Funktion ist in `full` (dem Standard) enthalten und unabhängig von `storage`. Die Seite ist als **Experimentell** gekennzeichnet, da die Funktion sich noch in der Entwicklung befindet und möglicherweise unvollständig ist.

Diese Seite verwaltet nur eine bestehende Snapper-Konfiguration, deren `SUBVOLUME` `/` ist und deren `FSTYPE` `btrfs` ist. Sie überprüft das Laufwerk, das `/` mit `findmnt` enthält; eine Konfiguration namens `root`, die auf einen anderen Ort zeigt, wird nicht akzeptiert. Mehrere übereinstimmende Konfigurationen erfordern eine Auswahl. Das `snapper` Arch-Paket ist optional und muss manuell installiert werden. Snapper muss `--jsonout` sowie explizite aktive/Standard-Spalten unterstützen. Es wird keine Konfiguration automatisch erstellt.

Bei einer Nicht-Btrfs-Root zeigt die Seite **Dieses Dateisystem ist nicht Btrfs** an und bietet keine Snapshot-Operationen an. Erkennungsfehler, fehlendes Snapper, fehlende Root-Konfiguration, Berechtigungsverweigerung und Backend-Fehler sind separate Zustände. Wenn Abfragen von normalen Benutzern verweigert werden, drücken Sie **a — Autorisieren**, um Polkit-Zugriff anzufordern. Administrative Aktionen werden im eingeschränkten `system-settings`-Helfer ausgeführt, nicht in einer erhöhten TUI; eine Authentifizierung kann bei späteren Abfragen erneut verlangt werden.
Für die Authentifizierung ist ein vorhandener grafischer Polkit-Agent erforderlich.

Tasten: **↑/↓** auswählen, **Enter** Details/Konfiguration, **c** erstellen, **d** löschen, **p** Aufbewahrungsrichtlinien, **r** aktualisieren, **Esc** zurück, **?** Hilfe. In der Ansicht für Aufbewahrung **↑/↓** eine Richtlinie auswählen, **e** bearbeiten, **c** Bereinigungsalgorithmen durchschalten und **Enter** zur Bestätigung öffnen. Die Erstellung öffnet eine optionale Beschreibung (bis zu 256 druckbare Zeichen); eine leere Beschreibung verwendet „ARGVUS manuell“
„Snapshot“. Sowohl das Erstellen als auch das Löschen erfordert eine Bestätigung, zunächst mit Fokus auf Abbrechen. Das Löschen gilt für einen ausgewählten Snapshot und schließt Snapshot 0 aus, den aktuell aktiven Snapshot und den Standardsnapshot. Der Helfer überprüft diese Bedingungen erneut, bevor er Snapper aufruft. Listen werden nach den Vorgängen aktualisiert; ein unterbrochener Vorgang wird niemals automatisch wiederholt. Überprüfen Sie zuerst die aktualisierte Liste.

Um Snapshots zu vergleichen, wählen Sie den Basis-Snapshot und drücken Sie **b**, dann wählen Sie einen anderen aus
Ziel und drücken Sie **v**. Die Seite zeigt die sortierten IDs, die Anzahl der erstellten, entfernten und geänderten Pfade sowie die sechs Statusanzeigen von Snapper. Wählen Sie einen Pfad aus und drücken Sie **Enter**, um die Inhaltsdifferenz anzufordern; **Esc** kehrt zu den geänderten Pfaden und dann zur Snapshot-Liste zurück. Die Anzeigen codieren Erstellung/Entfernung oder Inhalt/Typ in Position 1, Berechtigungen in 2, Eigentümer in 3, Gruppe in 4, erweiterte Attribute in 5 und ACL in 6. Ein Punkt bedeutet keine Änderung in dieser Position. Status listet Pfade auf;
diff zeigt den Textinhalt an. Binäre/nicht-UTF-8-diff-Ausgaben werden nicht angezeigt. Diff-Ausgaben über 256 KiB werden als gekürzt gemeldet und nicht angezeigt. Mehrdeutige oder fehlerhafte Statuszeilen werden abgelehnt. Das Zeilenformat von Snapper kann Pfade mit Zeilenumbrüchen nicht eindeutig darstellen; solche Namen werden nicht unterstützt. Berechtigungsfehler können über **a — Autorisieren** erneut versucht werden; ein entfernte Snapshot oder geänderte Root-Konfiguration erfordert das Aktualisieren der Liste. Backend-Tests verwenden eine Fake
`ProcessRunner` und erfordern weder Snapper, Btrfs noch Root.

Wählen Sie aus dem Vergleich einen Pfad aus und drücken Sie **r**, um die Wiederherstellung der Datei aus dem Ziel-Snapshot zu überprüfen. Der Helfer überprüft, dass beide Snapshots noch zur Root-Konfiguration gehören, der Pfad noch im Vergleich vorhanden ist und die Quelle eine reguläre Datei in Snapper's Standard-`/.snapshots/<id>/snapshot`-Ansicht ist. Der eingeschränkte Snapper-Diff wird angezeigt, wenn er textuell ist; binäre oder zu große Diffs werden
identifiziert, ohne ihre vollständigen Inhalte in die Benutzeroberfläche zu laden. **Enter** öffnet eine separate Bestätigung, die die Quelle und das Ziel benennt; **Esc** bricht ab, ohne die Datei zu ändern. Die Bestätigung erstellt zuerst einen `single`-Sicherheitsschnappschuss und verwendet dann Snapper `undochange` nur für diesen Pfad und überprüft, ob er dem ausgewählten Schnappschuss entspricht. Die ID des Sicherheitsschnappschusses wird gemeldet und der Schnappschuss wird beibehalten. Die Wiederherstellung teilt sich das von ARGVUS-Paket-Upgrades verwendete Lock, sodass sie sich nicht überschneiden kann.
Eine offizielle Control Center pacman Transaktion.

Die erste Version akzeptiert nur reguläre Dateien, die root gehören, unter root-eigenen übergeordneten Verzeichnissen, die nicht von Gruppe oder anderen beschreibbar sind. Sie lehnt Verzeichnisse, Symlinks, Hardlinks, spezielle Dateien, Mountpoints/Subvolumes, Snappers internes Verzeichnis und von Benutzern beschreibbare Orte ab. Dateien, die im ausgewählten Snapshot fehlen, werden niemals gelöscht. Snappers Root `undochange` garantiert keine Anwendung oder
Datenbankkonsistenz und nicht atomar; der Sicherheitssnapshot ist ein Wiederherstellungspunkt,
Keine Garantie. Snapper bewahrt nur Metadaten, die von seinem Build unterstützt werden. Separat
Untervolumes und Dateien außerhalb des konfigurierten Root bleiben außerhalb der Operation.

Manuelle Schnappschüsse sind standardmäßig `single`, nur für Snapper lesegeschützt, ohne Aufräumen
Algorithmus von dieser Seite zugewiesen. Bestehende Snapper-Timer und externe Operationen
weiterhin angewendet. Schnappschüsse sind keine Full-System-Images: separate Untervolumen,
einschließlich eines separaten `/home`, sind nicht enthalten. Diese Version bietet **kein vollständiges System-Rollback, keine Installation oder Erstellung einer Snapper-Konfiguration**. Drücken Sie **p** auf einer unterstützten Root-Konfiguration, um die bekannten Aufbewahrungseinstellungen zu überprüfen. Fehlende Werte werden als ARGVUS-Fallbacks gekennzeichnet, nicht als Werte, die aus der installierten Snapper-Erstellungsvorlage gelesen wurden; unbekannte Schlüssel dienen nur der Diagnose. Drücken Sie **e**, um einen unterstützten Wert zu bearbeiten. Die Bestätigung zeigt die alten und
neue Werte; nach dem Speichern liest die Seite die Konfiguration erneut und überprüft sie. Drücken Sie **c**, um die Bereinigungsalgorithmen zu wechseln, und **Enter**, um einen zu prüfen und zu bestätigen. Die Bereinigung verwendet Snapper's offiziellen Algorithmus; es gibt keine exakte Vorschau, und die gemeldete Anzahl der entfernten Elemente wird durch den Vergleich von Listen nach der Ausführung gemessen. Arch Snapper 0.13.2 bietet `snapper-timeline.timer` und `snapper-cleanup.timer`; die Erstellung der Zeitleiste erfordert den Zeitleisten-Timer, und konfigurierte Bereinigungsrichtlinien werden ohne Bereinigung nicht ausgeführt
Planung. Die Seite zeigt nur den Status der Einheit an und aktiviert keine Timer. Platzbeschränkungen erfordern konfigurierte Btrfs-qgroups und werden hier nicht angezeigt.

### Paketoperationen und Snapshots

Die Paketinstallation, Updates und Entfernung erfordern weder Snapper noch Btrfs oder eine Snapshot-Konfiguration. **Alle upgraden** führt `pacman -Syyu` über den privilegierten Helfer mit `pkexec` aus; einzelne offizielle Updates führen `pacman -S <package>` aus. ARGVUS erstellt für diese Vorgänge keine automatischen Snapshots. Die Snapshots-Seite bleibt unabhängig und behält ihre eigenen Btrfs-/Snapper-Anforderungen bei. Separat in pacman konfigurierte Hooks bleiben vom System kontrolliert.

### Manuelle Validierung

- Auf einem ext4/xfs-Root, öffnen Sie die Seite auf Englisch und Portugiesisch. Überprüfen Sie auf Portugiesisch die genaue Meldung **Este sistema de arquivos não é Btrfs** und prüfen Sie, dass c/d keine Operation öffnen.
- In einer wegwerfbaren Btrfs-VM testen Sie Snapper nicht vorhanden, Snapper ohne Root-Konfiguration und eine Konfiguration namens `root`, die auf ein anderes Untervolume verweist.
- Mit einer vorhandenen Root-Konfiguration Listen von Snapshots aufrufen, Details öffnen, erstellen einer
  Snapshot mit Leerzeichen/Sonderzeichen, das Löschen abbrechen und dann nur den neu erstellten Snapshot löschen, nachdem seine Nummer und Beschreibung bestätigt wurden.
- Überprüfen Sie verweigerte/abgebrochene Authentifizierung, geschützte aktive/standardmäßige Snapshots, Aktualisierung, Tastaturnavigation, Größenänderung und beide Sprachen. Testen Sie das Löschen nicht an Snapshots, die für die Wiederherstellung benötigt werden.

Nur das `argvus`-Paket ändert sich. Erstellen Sie es aus `de/argvus` mit `make build`, um das `.pkg.tar.zst`-Paket zu erzeugen; die Installation ist eine separate Bedieneraktion.

Sprache unter Sprache & Region speichert die Gebietsschemaeinstellungen im Benutzerprofil (`~/.config/locale.conf`), ohne `/etc/locale.conf` zu ändern. Die Sitzung verwendet diese Präferenz nach erneutem Login. Während des Willkommens zeichnen F6 Weiter oder die Fußzeile Weiter-Schaltfläche die Überprüfung auf und gehen weiter; Shift+Tab fokussiert die Schaltfläche und Enter aktiviert sie, auch auf Informationsseiten.

Bluetooth erscheint im Willkommensbildschirm nur, wenn das Kontrollzentrum einen Adapter erkennt. Bestehender Fortschritt für einen ausgeblendeten Bluetooth-Schritt wird beibehalten.

Sprache und die Tastaturlayout-Einstellungen für die Sitzung sind persönliche Präferenzen und erfordern keine Administratorrechte. Neue Profile verwenden standardmäßig Englisch und das US-Tastaturlayout; bestehende persönliche Präferenzen bleiben erhalten. Datum/Uhrzeit, NTP, Systemgebietsschein-Erstellung und das Konsolen-(TTY-)Tastaturlayout behalten ihr systemweites administratives Verhalten bei.

Für eine brasilianische Sitzungs-Tastatur wählen Sie `br`, wenden Sie das Layout an, und wählen Sie dann `abnt2` unter Variante. Dies schreibt die persönliche Hyprland-Konfiguration ohne administrative Berechtigung; das Konsolen-Keyboard ist separat. ABNT2 verwendet Tote-Tasten: `~` gefolgt von `a` erzeugt `ã`, und `~` gefolgt von Leerzeichen erzeugt `~`. Die Anwendung der Variante aktiviert das konfigurierte Standard-Layout; Aktivierungsfehler werden gemeldet.

Die Zeitzone speichert TZ unter `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` ohne Autorisierung. Melden Sie sich ab und wieder an, damit Anwendungen sie übernehmen. Datum & Uhrzeit zeigt weiterhin die Systemuhr in der Systemzeitzone an und ermöglicht deren Bearbeitung mit administrativer Autorisierung; NTP bleibt unverändert.

Anzeigeprofile und Arbeitsbereichszuweisungen werden über die generierte Lua-Konfiguration und einen Hyprland-Neuladevorgang angewendet. Die Monitorkraft wird über den Lua-DPMS-Dispatcher verwendet, der von Hyprland 0.56.2 unterstützt wird.

Die Schatten-Renderleistung ist auf 1–4 (Standard 4) für die Kompatibilität mit Hyprland 0.57 begrenzt. Ältere gespeicherte Werte über 4 werden beim Laden der Konfiguration begrenzt.

## Fensterwackeln

Öffne **Desktop & Eingabe → Hyprland → Animationen → Fenster** und benutze die
**Wackel**-Steuerungen. Suche nach `wobble` oder öffne die Seite direkt:

```sh
argvus control-center hyprland animations windows
```
Wobble ist standardmäßig deaktiviert. Seine ARGVUS-Standardeinstellungen sind Steifigkeit `100`, Dämpfung `10`, Masse `1`, Intensität `1` und Mesh-Auflösung `8`. Steifigkeit und Masse akzeptieren `0.0001–1000`, Dämpfung `0–1000`, Intensität `0–2`, und das Mesh eine Ganzzahl `2–32`. Eine höhere Mesh-Auflösung erhöht die Rendering-Kosten. Verwenden Sie die Pfeiltasten, um Werte anzupassen, oder Enter, um sie einzugeben. **Wobble-Standardeinstellungen wiederherstellen** ändert den Entwurf; **Übernehmen** speichert alle ausstehenden Animationseinstellungen und lädt Hyprland einmal neu, ohne
Die Sitzung wird neu gestartet. Beim Zurückgehen innerhalb von Animationen bleibt der gemeinsame Entwurf erhalten.

Der globale Animationsschalter unterdrückt Wackeln, behält jedoch seine individuellen Einstellungen bei. Die kanonischen Werte befinden sich unter `effects.wobble_*` im ARGVUS-Zustand; `hypr/state.lua` wird generiert und darf nicht manuell bearbeitet werden.

Dies erfordert Hyprland **0.57.0 oder neuer**, erzwungen durch die Paketabhängigkeit von `argvus-hyprland`. Version 0.56 wird für diese Konfiguration nicht unterstützt. Das maximale Intensitätsniveau folgt der [veröffentlichten 0.57.0 Quelle ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), die es auf `2` begrenzt, auch wenn in neuerer Dokumentation `3` aufgeführt sein mag.

Willkommen öffnet sich mit dem ARGVUS-Logo und WELCOME. Drücken Sie F6 Weiter, um die Checkliste anzuzeigen, ohne einen Schritt abzuschließen.

Frische Installationen verwenden standardmäßig die persönliche Zeitzone `America/New_York` und nur das Tastaturlayout `us` ist aktiviert. Bestehende persönliche Einstellungen werden beibehalten; die Systemzeitzone bleibt unverändert.

Optionen des Taskleistenmoduls für Batterie und Bluetooth erscheinen nur, wenn die entsprechende Hardware erkannt wird. Das Ausblenden von "Weiter wach halten" blendet dessen Indikator aus, ohne den Suspendierungsblocker zu ändern. Dienstprogramme unterstützen ein eingeklappbares Hover-Panel. Die Darstellung der Arbeitsbereichssymbole zeigt Symbole für geöffnete Fenster; die Systemablage steuert Symbole von Hintergrundanwendungen.


Die Seite mit den Tastenkombinationen enthält **KI-Agenten-Auswahl (optional argvus-ai Paket)** mit der Standardkombination SUPER + A. Sie verwendet die app.ai Katalog-ID, daher folgen das Umbelegen und Deaktivieren der gleichen standardmäßigen keyboard_shortcuts-Konfiguration wie andere Einträge. Ihre ausführbare Lua-Bindung wird von argvus-ai bereitgestellt; der Katalogeintrag allein aktiviert sie ohne dieses optionale Paket nicht.


Systemsteuerung > Darstellung > Taskleiste > Symbole > Optionale Module umfasst jetzt **KI-Agenten**. Standardmäßig ist es aktiviert und wird in /taskbar/icons/ai_agents_enabled gespeichert. Schalten Sie es im Taskleistenentwurf um und auf Übernehmen, um die Dienstgruppierung beizubehalten und neu zu generieren; das Deaktivieren ändert weder die SUPER + A-Tastenkombination noch Einträge im Anwendungsstarter.

Autostart-Schalter bearbeiten den `.desktop`-Eintrag des Benutzers oder erstellen eine Benutzerkopie eines Systemeintrags. Namen, Befehle und Aktivierungskennzeichen stammen von `[Desktop Entry]`; zusätzliche Desktop-Aktionsgruppen bleiben unverändert.

Dienstbestätigungen behalten den ausgewählten Benutzer-/Systembereich zusammen mit der Einheit und Aktion bei. Eine Einheit mit demselben Namen in einem anderen Bereich kann eine bestätigte Aktion nicht umleiten; Detailseiten behalten ebenfalls den ursprünglichen Bereich bei.

Der Speicher umfasst verschachtelte Mountpoints wie `/home`, `/boot` und `/boot/efi` aus dem Dateisystem-Inventar. Die Diagnostik verwendet dasselbe Inventar für die bestehenden Root- und Home-Dateisystemnutzungsprüfungen.

Neue Wiederherstellungs-Sicherheits-Snapshots verwenden den `number`-Bereinigungsalgorithmus von Snapper. Ihre Entfernung folgt der konfigurierten Aufbewahrungsrichtlinie, wenn die Bereinigung ausgeführt wird. Bestehende Snapshots ohne Bereinigungsalgorithmus werden nicht verändert, und bei manueller Erstellung bleibt die bestehende Richtlinie erhalten.

Snapshot-Vergleiche melden nicht unterstützte Pfadaufzeichnungen separat. Gültige Dateien bleiben zur Überprüfung verfügbar; Pfade mit ungültiger Codierung, Steuerzeichen oder nicht unterstützten Komponenten bieten keine Diff- oder Wiederherstellungsaktionen an.

Die Energieeinstellungen schreiben das ARGVUS-logind-Drop-in unter `[Login]` und erstellen bei Bedarf dessen Verzeichnis. Ladefehler werden gemeldet. Das Verhalten des Batterie-Deckels wird auf `HandleLidSwitch` abgebildet; der legacy `default`-Kontext ist ein Alias, während AC- und Dock-Verhalten ihre eigenen logind-Schlüssel verwenden.Die Mirrorlist-Anwendung akzeptiert nur Kommentare, Leerzeilen und HTTP/HTTPS `Server`-Einträge. Nutzlasten sind auf 64 KiB begrenzt und werden vor der Erhöhung sowie erneut durch den privilegierten Helfer überprüft; pacman-Direktiven wie `SigLevel` und `XferCommand` werden abgelehnt.

Die Cache-Bereinigung für deinstallierte Pakete behält keine zwischengespeicherten Versionen. Repository-Operationen lehnen es ab, gemeinsam genutzte Bereiche mehrerer Repository-Abschnitte einzubeziehen. Hostnamen mit umgebenden Leerzeichen und eine leere generierte Gebietsschema-Auswahl werden abgelehnt. Einstellungen-Datei-Schreibvorgänge lehnen symlinked Ziele und bestehende temporäre Dateien ab.

Paket-Suchen und Seitenladevorgänge verwerfen Ergebnisse einer früheren Seite oder Abfrage. Die aktuelle Seite wird dann erneut geladen, sodass ein altes Ergebnis nicht eine andere Ansicht als kürzlich aktualisiert markieren kann.

Transaktionsausgaben werden bereinigt und begrenzt. Die Anzeige behält bis zu 10.000 Zeilen oder 4 MiB bei und meldet ausgelassene Ausgaben. Erfasste Prozessströme respektieren ihr konfiguriertes Limit, standardmäßig 8 MiB für die Live-Ausführung; überschüssige Ausgabe wird entleert und nach Beendigung des Prozesses gemeldet.

Das Boot-Inventory liest den Arch-Kernel `pkgbase`, um Kernel-Images, Initramfs-Dateien und mkinitcpio-Presets zu finden, während die Kernel-Version separat angezeigt wird. Literal-mkinitcpio-Arrays werden gelesen, ohne ihre Klammern einzuschließen oder die Konfiguration auszuführen.

Änderungen der Energieeinstellungen für Deckel, Netzschalter und Aktiv-Halten werden nur dann in der kanonischen Konfiguration gespeichert, nachdem die Laufzeitaktion erfolgreich war. Das Abbrechen der Autorisierung lässt den zuvor gespeicherten Wert unverändert. Wenn kein ARGVUS logind-Drop-in einen Wert bereitstellt, wird die Hauptlogind-Konfiguration als Fallback gelesen.

Nachdem eine Änderung der Aufbewahrung fehlschlägt, liest die Seite die aktuellen Einstellungen erneut, bevor ihre Werte wieder angezeigt werden. Die Konfigurationsauswahl beginnt beim ersten Eintrag.

ARGVUS-Paketoperationen teilen sich ein exklusives Sperre mit selektiver Wiederherstellung. Installation, Entfernung, Neuinstallation, Downgrade, Aktualisierung der Metadaten, Bereinigung des Caches und Änderungen am Repository oder Spiegel werden verweigert, solange diese Sperre gehalten wird. Diese Koordination gilt für Operationen, die über den ARGVUS-Helfer gestartet werden; externe Pacman- und AUR-Tools verwenden diese Sperre nicht.

Während eine Hintergrundoperation Änderungen anwendet, halten `q` und `Ctrl+C` das Kontrollzentrum offen und zeigen eine Meldung zum Warten an. Dies gilt auch nach dem Navigieren weg von der Seite, die die Aktion gestartet hat. Warten Sie auf deren Ergebnis und fordern Sie dann erneut den Exit an. Gewöhnliche schreibgeschützte Aktualisierungen blockieren den Exit nicht.

Wi-Fi-Passwort, manuelle DNS- und Netzwerksuchfelder akzeptieren eingefügten Text in Klammern. Eingefügte Steuerzeichen und Zeilenumbrüche werden weggelassen. Nicht unterstützte nur mit Strg oder nur mit Alt ausgeführte Tastenkürzel fügen keine Buchstaben ein. Editoren akzeptieren druckbare AltGr-Ereignisse, die als Strg+Alt kodiert sind; Strg+S speichert nur ohne Alt. Der gemeinsame Texteditor behält die vertikale Navigation in visuellen Spalten bei und richtet das horizontale Scrollen für breite Zeichen aus.

Änderungen der Audio-Lautstärke und Stummschaltungen für Mikrofone und nicht standardmäßige Ausgänge überschreiben die gespeicherten Standard-Ausgabepräferenzen nicht mehr. Das Backend speichert diese Präferenzen nur, wenn es den ausgewählten Knoten als aktuellen Standardausgang identifizieren kann. Die Seite zeigt von PipeWire gemeldete verstärkte Lautstärken über 100 % an; explizite Lautstärkeregelungen bleiben auf 0–100 % beschränkt.

Die Hardwareübersicht identifiziert GPU-Modelle getrennt von ihrer PCI-Adresse und Geräteklasse und bewahrt Handelsnamen in Klammern. Ihre Zusammenfassung zeigt den für die Sitzung abgefragten OpenGL-Renderer und alle von der Vulkan-Suche zurückgegebenen Gerätenamen, einschließlich Software-Geräten. Diese globalen Ergebnisse werden nicht als Fähigkeiten jeder physischen GPU dargestellt.

Wenn ein Administrator das für die automatische Anmeldung konfigurierte Konto sperrt oder entfernt, deaktiviert der Kontenanbieter zunächst den greetd-Eintrag für die automatische Anmeldung. Wenn diese Bereinigung fehlschlägt, wird die Kontoverwaltung verweigert. Sollte das Sperren oder Entfernen später fehlschlagen, bleibt die automatische Anmeldung deaktiviert. Die automatische Anmeldung für ein anderes Konto bleibt erhalten; das Entsperren eines Kontos aktiviert die automatische Anmeldung nicht. Bestehende Sitzungen werden nicht geschlossen und greetd wird nicht neu gestartet.

## Tastenkombinationen erstellen und verwalten

**Desktop & Eingabe → Tastenkombinationen** öffnet ein Menü mit **Verknüpfungen erstellen** und **Alle Verknüpfungen**. Bestehende CLI-Aliase (`keybindings`, `shortcuts`, `keyboard`) öffnen dieses Menü; fügen Sie `create` oder `list` hinzu, um ein untergeordnetes Element direkt zu öffnen. Die globale Suche findet ebenfalls beide untergeordneten Elemente.

**Verknüpfungen erstellen** fragt nach einer Beschreibung, einer Tastenkombination und einem Aktionstyp. Wählen Sie **Aktuelle Verknüpfung** und drücken Sie `Enter`, dann drücken Sie die gewünschte Tastenkombination. **Verknüpfung ändern** startet die gleiche Erfassung; keine der beiden öffnet einen Texteditor. Das Erfassen ändert nur den Entwurf. Wenn eine Kombination bereits verwendet wird, wird der Konflikt sofort angezeigt, noch bevor die Beschreibung und der Befehl ausgefüllt werden. Das Bestätigen von **Ersetzen** wählt die Kombination aus und zeichnet Ihre Wahl auf; es erstellt nicht die Verknüpfung oder eine andere deaktivieren, bis **Erstellen**/**Speichern**. Das Abbrechen des Konflikts bewahrt die vorherige Kombination. Ein **Befehl** wird ausgeführt, wenn die Verknüpfung gedrückt wird; beim Erstellen wird er nie ausgeführt. Ein **Hyprland-Dispatcher** verwendet einen unterstützten Lua-Dispatcher-Namen, wie `window.float`, mit optionalen Argumenten als JSON-Objekt, zum Beispiel `{"action":"toggle"}`. Submaps, Mausbindungen und erweiterte Flags bleiben außerhalb dieses Formats.

Verwenden Sie `Tab`, um zwischen dem Formular und **Aktionen** zu wechseln, `←/→`, um zwischen **Erstellen** und **Abbrechen** zu wechseln, und `Enter`, um die ausgewählte Aktion auszuführen. Abbrechen oder `Esc` verwirft den Entwurf. Nach dem Erstellen öffnet **Alle Shortcuts** mit dem neuen Element ausgewählt und sichtbar. Benutzerdefinierte Einträge erscheinen in **Benutzerdefinierte Shortcuts**; die Suche gleicht Beschreibung, Tasten und Aktion ab. `Enter` bearbeitet Beschreibung, Aktion und Tasten, `Space` aktiviert/deaktiviert, und das Auswählen der **Löschen**-Zeile und Drücken von `Enter` öffnet eine Bestätigung. Fokussiert auf Abbrechen. Bestätigen löscht den Eintrag und kehrt zur aktualisierten Liste zurück; Abbrechen behält ihn.

Aktivierte Tastenkombinationen im gleichen Kontext können eine Kombination nicht stillschweigend wiederverwenden: konfigurierbare Konflikte erfordern eine Bestätigung zum Ersetzen, und geschützte Tastenkombinationen können nicht ersetzt werden. Das Ersetzen speichert die neue Tastenkombination und deaktiviert Konflikte in einer Konfigurationsaktion. **Standard wiederherstellen** gilt nur für offizielle Einträge; **Alle Tastenkombinationen wiederherstellen** bewahrt benutzerdefinierte Einträge. Wenn das Wiederherstellen mit einer aktiven benutzerdefinierten Tastenkombination kollidieren würde, ändern oder deaktivieren Sie zuerst diese benutzerdefinierte Tastenkombination.

Speichern und anwenden laufen im Hintergrund. Wenn das Speichern fehlschlägt, bleibt der Entwurf verfügbar. Wenn die Anwendung auf die Sitzung nach dem Speichern fehlschlägt, zeigt die Seite an, dass sie gespeichert wurde, und behält die gleiche ID für einen erneuten Versuch. Benutzerdefinierte Datensätze befinden sich in `/hyprland/custom_shortcuts` in der kanonischen Konfiguration, projiziert auf `data/generated/hypr/custom-shortcuts.lua`; generierte Dateien nicht bearbeiten.

Tastenkombinationsfehler bleiben innerhalb der TUI: Das Popup passt seine Größe der umbrochenen Nachricht an und behält **Enter OK** sichtbar. Technische Tastenkombinationsdiagnosen gehen in das lokale Journal (`journalctl -t argvus-control-center`) statt auf stdout/stderr. Eine Löschung, die gespeichert, aber nicht angewendet werden kann, meldet diese Unterscheidung; laden Sie die Sitzungs-Konfiguration neu, um die zuvor aktive Bindung zu entfernen.

**Update-Status** in **Taskleiste → Symbole → Optionale Module** zeigt die Anzahl der verfügbaren Updates an und ist standardmäßig aktiviert. **AUR prüfen** ist standardmäßig deaktiviert und erfordert `paru` oder `yay`; wenden Sie den Taskleistenentwurf an, um eine der Einstellungen zu speichern. Die Prüfungen werden beim Start und stündlich ausgeführt; ein Klick auf die Anzeige öffnet die offizielle Seite für Paketaktualisierungen.

## Globaler Fenster-Modus

Öffne **Systemsteuerung → Desktop & Eingabe → Hyprland → Fenster → Modus** und wähle **Fenster kacheln (Standard)** oder **Schwebende Fenster**. Derselbe Auswahlknopf ist über das Fenster-Modus-Symbol in der Taskleisten-Dienstprogrammgruppe verfügbar.

Die Einstellung gilt für jeden Monitor und Arbeitsbereich und wird nach dem Abmelden gespeichert. Floating wandelt bestehende Anwendungsfenster und neu geöffnete Fenster um. Tile stellt nur Fenster wieder her, die durch diese Richtlinie konvertiert wurden; Dialoge und andere Fenster, die bereits im Floating-Modus waren, bleiben im Floating-Modus. Ein Fenster, das während aktiver Floating-Einstellung explizit angeheftet wurde, bleibt ebenfalls im Floating-Modus, um das Anheften beizubehalten. Das gekachelte Layout wird neu berechnet; sein genau vorheriger Split-Baum wird nicht wiederhergestellt.

`SUPER + SHIFT + Space` behält sein normales Verhalten pro Fenster im Kachelmodus bei. Im Floating-Modus wechselt es ein Fenster nicht zurück in den Kachelmodus. Panels, die Taskleiste und Benachrichtigungen sind Layer-Shell-Oberflächen und werden nicht beeinflusst. Vollbild und Maximierung verwenden weiterhin das normale Verhalten von Hyprland.

Die kanonische Präferenz ist `/hyprland/window_mode` in `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Ihre generierte Projektion ist `data/generated/hypr/window-mode.lua`; nicht bearbeiten. Das Wiederherstellungsregister pro Fenster befindet sich unter `$XDG_RUNTIME_DIR/argvus`, ist auf die Kompositor-Instanz beschränkt und überlebt Konfigurations-Neuladungen. Es wird in einer neuen Sitzung nicht wiederverwendet.

Wenn die Anwendung aussteht oder fehlschlägt, unterscheidet die Schnittstelle dies von der gespeicherten Einstellung. Wählen Sie die Option erneut, um es zu versuchen. Ohne einen Compositor wird die Einstellung für die nächste Anmeldung gespeichert.

## Fensterbuttons und Anwendungsleiste

Der Floating-Modus fügt normalen Anwendungsfenstern eine ARGVUS-Titelleiste mit **Minimieren**, **Maximieren / Wiederherstellen** und **Schließen** hinzu. Ein Doppelklick auf die Titelleiste wechselt zwischen Maximierung und Normalgröße. Die zusätzliche Dekoration wird im Kachel- und Vollbildmodus ausgeblendet. Anwendungen, die ihre eigenen Steuerungen zeichnen, können beide Leisten anzeigen; einheitliche ARGVUS-Steuerungen sind beabsichtigt.

Eine zweite Waybar erscheint auf jedem Monitor, am Rand gegenüber der Haupttaskleiste. Sie listet nur die Fenster des aktuellen Arbeitsbereichs dieses Monitors auf, einschließlich der minimierten Fenster. Versteckte Gruppentabs erzeugen keine zusätzlichen Einträge. Minimierte Einträge behalten ihre ursprüngliche Monitor-/Arbeitsbereichszuordnung. Klicken Sie auf ein inaktives Fenster, um es zu aktivieren, auf ein aktives Fenster, um es zu minimieren, oder auf ein minimiertes Fenster, um es wiederherzustellen; sein Kontextmenü bietet Wiederherstellen, Minimieren, Maximieren/Wiederherstellen und Schließen. Unbekannte Anwendung Symbole fallen auf ein Fenstersymbol zurück.

`SUPER + Alt + Up` platziert die Hauptleiste oben und die Fensterleiste unten. `SUPER + Alt + Down` macht das Gegenteil. Eine Wiederholung einer Richtung lässt die Positionen unverändert. Kachel hat nur die Hauptleiste. Die Rückkehr zu Kachel stellt alle minimierten Fenster wieder her, bevor die Fensterleiste entfernt wird, und bewahrt dann die ursprüngliche Kachel-/Schwebepolitik.

Die Minimierung verwendet den reservierten `special:argvus-minimized`-Arbeitsbereich. Verwenden Sie diesen Arbeitsbereich nicht manuell. Der `argvus windows service`-Prozess zeichnet Identität, ursprünglichen Arbeitsbereich/Monitor, Geometrie, Vollbild-/Maximierungs- und Anheftungsstatus in einem laufzeitregister, das spezifisch für die Kompositorinstanz unter `$XDG_RUNTIME_DIR/argvus` ist, auf. Die Aufzeichnungen überstehen einen Dienstneustart oder eine Konfigurationsneuladung, überschreiten jedoch keine Kompositorsitzungen. Das Schließen eines Fensters entfernt dessen Aufzeichnung. Wenn ein Monitor verschwindet, verwendet die Wiederherstellung ein verfügbarer Output. Eine fehlende oder gescheiterte Wiederherstellungsleiste verhindert neue Minimierungen; eine gescheiterte Wiederherstellung behält den Datensatz und die Wiederherstellungsleiste bei.

Die Sitzung besitzt `argvus-windows.service` und `argvus-window-bar.service`. Letzterer hält sein Waybar-Kind im Tile-Modus inaktiv. Das hyprbars-Plugin wird von `argvus-hyprland` mitgeliefert, gebaut für das Hyprland 0.57.0 ABI auf x86_64; es ist nicht im Home-Verzeichnis des Benutzers kompiliert. Baue das Plugin mit einem Compositor-ABI-Update neu.

Befehle: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` und `minimize|restore|activate|maximize|close ADDRESS`. Die Fensteradressen stammen aus der Liste. `position top --persist` speichert/projiziert die Präferenz, ohne die Hauptleiste neu zu starten. Konfigurations- und Laufzeitfehlerdetails werden im Journal geschrieben; sichtbare Aktionsfehler zeigen ebenfalls eine übersetzte Benachrichtigung an.

## Windows-Untermenü und Schließen der Anwendung

**Desktop & Eingabe → Hyprland → Windows** gruppiert **Regeln**, **Abstände** und **Modus**.
Regeln bearbeiten die Platzierung pro Anwendung; Abstände bearbeiten innere und äußere Lücken; Modus enthält den sofortigen Kachel-/Schwebe-Selektor, getrennt vom Regel-Formular.
Suche öffnet die gleichen Seiten, und Zurück kehrt zu Windows vor Hyprland zurück.

Direkte Routen sind `argvus control-center hyprland windows`, optional gefolgt von `rules`, `spaces` oder `mode`. Die vorhandenen Routen `window-rules` und `appearance window-spaces` bleiben Aliase und zeigen die neuen Breadcrumbs an.

`SUPER + Q` schließt das dedizierte Control-Center-Fenster sofort, ohne die Bestätigung des OS-Fensters von Kitty. Nicht angewendete Entwürfe werden verworfen, wenn das Fenster auf diese Weise geschlossen wird. Die Überschreibung gilt nur für die Fensterklasse `argvus-control-center`; reguläre Terminals und andere TUIs behalten ihre Richtlinie zur Schließen-Bestätigung bei.

`SUPER + Alt + Space` schaltet den globalen Kachel-/Schwebemodus um und verwendet dieselbe Einstellung wie die Taskleiste und das Kontrollzentrum. Der entsprechende Befehl ist `argvus config window-mode toggle`. Gleichzeitige Aufrufe werden serialisiert; das Umschalten verwendet die gespeicherte Einstellung, auch wenn die Anwendung aussteht. Die Verknüpfung kann in den Tastaturkürzeln geändert oder deaktiviert werden; `SUPER + Shift + Space` behält seine Aktion für einzelne Fenster bei.
