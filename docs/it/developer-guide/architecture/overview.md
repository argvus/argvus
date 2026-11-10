---
title: Panoramica dell'architettura
description: Come ARGVUS forma un ambiente desktop integrato.
---
ARGVUS ha cinque livelli di runtime:

1. punti di ingresso e dispatch in `argvus`;
2. ciclo di vita della sessione in `argvus-session`;
3. compositore e superfici della shell in `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` e `argvus-control-panel`;
4. applicazioni utente come Centro di Controllo, calendario e monitor di sistema;
5. provider e risorse per display, rete, energia, aspetto, account e notifiche.

Lo stato condiviso collega questi livelli. `argvus-config` contiene il documento canonico ed è l'unico a scrivere in `data/generated/`, quindi una modifica del tema o degli effetti rigenera i file consumati da diversi pacchetti in una singola transazione; i servizi di sessione forniscono la gestione del ciclo di vita senza assumere la proprietà dell'implementazione di ogni componente.

## Confini di responsabilità

| Livello | Responsabilità principale | Esempi |
| --- | --- | --- |
| Punti di ingresso e dispatch | Avviare applicazioni desktop e risolvere i profili ARGVUS. | `argvus`, `argvus-app-profiles` |
| Ciclo di vita della sessione | Preparare l'ambiente, avviare Hyprland, sincronizzare l'ambiente del servizio utente e gestire la chiusura della sessione. | `argvus-session`, `argvus sessionctl` |
| Compositore e shell | Definire il comportamento delle finestre e le superfici desktop visibili. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Configurazione | Gestire `config.json`, il modello di percorso e tutti i file generati per il consumatore. | `argvus-config` |
| Applicazioni | Fornire impostazioni e flussi di lavoro desktop concentrati. | Pannello di controllo, calendario, launcher, monitor di sistema e dispositivi rimovibili |
| Provider e risorse | Implementare operazioni di dominio e risorse visive condivise. | display, rete, alimentazione, notifiche, aspetto, font e sfondi |

Il pacchetto `argvus` è il target di installazione e coordinatore, ma non è un runtime monolitico. I pacchetti focalizzati mantengono la proprietà dei loro binari, configurazioni e servizi. Il gestore delle sessioni li avvia al momento appropriato nella sessione grafica senza assorbirne l'implementazione.

Questa distinzione è importante durante il debug: un errore della sessione appartiene prima a `argvus-session`; un errore della barra delle applicazioni appartiene a `argvus-taskbar`/`argvus-waybar`; e un errore in un tema o sfondo generato appartiene al percorso di configurazione dell'aspetto e della sessione.
