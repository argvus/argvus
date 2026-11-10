---
title: Fornitori e servizi
description: Pacchetti del provider e le loro interfacce di sistema.
---
I progetti provider espongono interfacce mirate ai consumatori dell'interfaccia utente:

- `argvus network` e `argvus bluetooth` integrano NetworkManager e i helper per Bluetooth;
- `argvus-display` integra lo stato del monitor con Hyprland;
- `argvus-power` integra azioni di inattività, blocco e alimentazione;
- `argvus-firewall` gestisce `argvus-firewall.service`;
- l'helper `argvus removable-devices` in `argvus` utilizza UDisks2;
- l'helper `argvus control-center` in `argvus` utilizza le API del provider e le interfacce D-Bus di sistema.

Il Centro di Controllo è un client di questi provider, non il loro sostituto.
