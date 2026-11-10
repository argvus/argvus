---
title: D-Bus-Schnittstellen
description: D-Bus-Verträge, die von ARGVUS-Integrationen verwendet werden.
---
ARGVUS nutzt die System-D-Bus-APIs, anstatt einen desktopweiten Bus anzubieten. Wichtige Nutzer sind unter anderem BlueZ und logind im Control Center und Greeter, UDisks2 bei Wechseldatenträgern und ratbagd in den Hardware-Einstellungen.

Die genauen Schnittstellen gehören dem Anbieter oder dem Upstream-Dienst. Überprüfen Sie das zugehörige Repository und die installierten Introspektionsdaten, bevor Sie sich auf eine Methode oder Eigenschaft verlassen.
