---
title: Luce notturna
description: Regola la temperatura del colore e i programmi locali fissi su Hyprland.
---
Apri **Centro di Controllo > Desktop e Input > Schermi > Luce Notturna**, cerca **Luce Notturna** oppure esegui `argvus control-center displays night-light`.

La Luce Notturna e il suo programma sono inizialmente disattivati. Scegli una temperatura da **1000 a 20000 Kelvin** (predefinito **4500 K**). Valori più bassi producono colori più caldi. La pagina mostra l’attivazione manuale salvata, lo stato effettivo e la prossima transizione. Inserire un valore lo modifica; Esc annulla o torna a Schermi. I valori non validi vengono rifiutati prima del salvataggio. Le modifiche vengono salvate e applicate immediatamente. Se l’applicazione fallisce dopo il salvataggio, la preferenza salvata rimane visibile insieme allo stato effettivo e all’errore.

## Programma e azioni rapide

Il programma fisso utilizza il fuso orario locale corrente. L’inizio è incluso e la fine esclusa; **20:00–07:00** attraversa la mezzanotte. Inizio e fine devono essere diversi e utilizzare `HH:MM`. Non viene utilizzata alcuna posizione o calcolo solare.

Usa la scheda **Display** del Pannello di Controllo o **SUPER + Shift + B** per rapide attivazioni. Il selettore della luminosità mantiene tutte le sue azioni di luminosità e aggiunge la Luce Notturna quando il backend è disponibile, anche su un display senza backend della luminosità. Non esiste un modulo permanente sulla barra delle applicazioni.

Con la programmazione disattivata, l'attivazione manuale è persistente. Con la programmazione attivata, un rapido toggle crea una sostituzione temporanea fino alla prossima transizione programmata. Cambiare la programmazione o la temperatura cancella quella sostituzione. Disconnettersi, un nuovo istanza del compositore o una modifica della prossima transizione (incluso un cambiamento del fuso orario) la invalida anche.

## Comandi e stato

Tutte le interfacce usano lo stesso comando pubblico:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` restituisce lo schema JSON versione 1: `available`, `unavailable_reason`, `settings` (abilitato, temperatura, pianificazione_ababilitata, inizio, fine), `active`, `effective_temperature`, `mode` (`manual` o `scheduled`), `manual_override`, `next_transition` (ISO 8601 con offset), `minimum_temperature` e `maximum_temperature`. I campi effettivi sono null quando non disponibili; la temperatura effettiva è null quando l'identità è attiva. I guasti di disponibilità sono riportati nello stato; i guasti dell'azione restituiscono un valore diverso da zero con una diagnostica.

La preferenza canonica è `/displays/night_light`, memorizzata in modo atomico da `argvus config` in `$XDG_CONFIG_HOME/argvus/config/displays.json`. Le sovrascritture volatili e il blocco dell'azione si trovano sotto `$XDG_RUNTIME_DIR/argvus/night-light/`. I file di monitor generati sono indipendenti. Non modificare i file generati per configurare questa funzione.

## Durata

Richiede Hyprland e **hyprsunset >= 0.4.0**. La sessione avvia una volta l'upstream `hyprsunset.service`, applica lo stato salvato tramite `argvus-night-light-init.service` e valuta le pianificazioni ogni minuto tramite `argvus-night-light.timer` e `argvus-night-light.service`. Anche le modifiche di orologio e fuso orario fanno scattare la valutazione. Con la pianificazione disattivata, l'operazione periodica lascia invariato lo stato manuale. Le applicazioni ripetute interrogano l'identità/temperatura effettiva e saltano le scritture non modificate. Non è necessario ricaricare il compositore.

ARGVUS fornisce un drop-in di systemd per il servizio upstream: inizia con `--identity` e isola la scoperta della configurazione a `/usr/share/argvus/session/config/night-light`. Questo evita conflitti con i profili nativi e lascia intatti i file personali `hyprsunset.conf`. Il drop-in si applica a quel servizio upstream finché il pacchetto della sessione ARGVUS è installato, inclusi gli avvii al di fuori di ARGVUS. Le attuali sovrascritture di systemd dell'utente possono avere la precedenza; ispezionare `systemctl --user cat hyprsunset.service` quando si diagnostica quella situazione. Hyprsunset 0.4.0 rigetta il percorso separato dell'argomento di `--config`, quindi la scoperta della configurazione è isolata utilizzando variabili XDG locali al servizio.

Il backend utilizza l'IPC ufficiale Hyprsunset (`temperature`, `identity`, `identity get`), con I/O limitato e lettura dopo le modifiche. Non viene utilizzato alcun shader, backend alternativo del compositore, scrittura della configurazione di sistema o escalation dei privilegi. I controlli non disponibili sono disabilitati nel Centro di Controllo e nascosti nella scheda Display.
