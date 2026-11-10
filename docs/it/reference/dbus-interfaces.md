---
title: Interfacce D-Bus
description: Contratti D-Bus utilizzati dalle integrazioni ARGVUS.
---
ARGVUS utilizza le API di sistema D-Bus invece di esporre un bus unico per l'intero desktop. Consumatori importanti includono BlueZ e logind nel Centro di Controllo e nel greeter, UDisks2 nei dispositivi rimovibili e ratbagd nelle impostazioni hardware.

Le interfacce esatte appartengono al fornitore o al servizio upstream. Controllare il repository proprietario e i dati di introspezione installati prima di fare affidamento su un metodo o una proprietà.
