---
title: Begrüßer
description: Der Login-Begrüßer, der greetd startet, wird im argvus-Paket mitgeliefert.
---
`argvus-greeter` ist der Login-Begrüßer für greetd. Es ist ein keyboard-orientiertes Terminal-Frontend, das die installierten Wayland-Sitzungen erkennt, die Konten mit ihren Avataren anzeigt und das aktive ARGVUS-Thema für das ausgewählte Konto anwendet.

Es ist Teil des `argvus`-Pakets, des einzigen Pakets des Desktops. Sein Quellcode befindet sich in `crates/argvus-greeter` des `argvus`-Repositoriums, als Mitglied des Root-Cargo-Arbeitsbereichs, und es wird zusammen mit dem Rest von `argvus` gebaut.

Der Begrüßer authentifiziert die Benutzer nicht selbst. greetd übernimmt die Authentifizierung über PAM. Der Launcher öffnet den Begrüßer mit `argvus tui-terminal`, sodass der Begrüßer kein separates Terminalprogramm benötigt.

Die Konfigurationsdateien des Begrüßers werden unter `/etc/argvus` installiert. Verwenden Sie `argvus greeter-setup --help`, um die Einrichtungsoptionen zu sehen, die das Paket bereitstellt. Für den Login-Ablauf siehe [Sessions](/docs/user-guide/sessions/).

Die Anmeldungsübergabe-Socket autorisiert jeden Client mit Kernel-Peer-Zugangsdaten. Nur root, der Besitzer des Hilfsprogramms und das von greetd authentifizierte Konto dürfen Steuerbefehle senden. Andere lokale Konten werden abgelehnt, obwohl die Socket Verbindungen zwischen Benutzern erlaubt. Ein vorbereitetes DRM-Frame bleibt an seinen ursprünglichen Descriptor gebunden; der Helfer wartet auf diesen Descriptor, bevor er die Anzeige übernimmt.

Wenn das Authentifizierungsfeld fokussiert ist, werden druckbare Zeichen (einschließlich `j` und `k`) in die Antwort eingegeben. Tab und Shift-Tab bewegen den Fokus; Rücktaste entfernt das letzte Zeichen. Dies gilt für geheime und nicht geheime Eingabeaufforderungen.

Die Kontoliste scrollt, um das ausgewählte Konto sichtbar zu halten, während Sie navigieren oder das Terminalgröße ändern.
