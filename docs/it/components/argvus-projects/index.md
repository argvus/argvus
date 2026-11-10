---
title: Progetti
description: Il selettore di progetti e la configurazione dei progetti, tramite i progetti argvus.
---
`argvus projects` elenca e apre i tuoi progetti. È un comando in corso di elaborazione del pacchetto `argvus`, e `SUPER + O` apre il selettore.

## Comandi

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## Cos'è un progetto

Un progetto è una directory elencata nella configurazione (`/projects/paths`), o una sottodirectory di una root elencata in `/projects/roots`. Al massimo nove progetti sono elencati: i numeri dei collegamenti `SUPER + CTRL + N`. Aprire un progetto concentra lo spazio di lavoro di Hyprland. La prima volta che si apre un progetto, ARGVUS avvia un terminale e l'editor impostato in `/projects/editor` al suo interno.

La configurazione viene letta e scritta tramite `argvus config`, quindi il Centro di Controllo e la shell vedono la stessa lista.

## Spazi di lavoro del progetto e la barra delle applicazioni

I progetti utilizzano normali spazi di lavoro Hyprland, non spazi di lavoro speciali. `SUPER + O` mostra il numero del progetto, la directory e il collegamento diretto `SUPER + CTRL + N`. Entrambi i punti di accesso si concentrano sullo stesso spazio di lavoro. Solo gli spazi di lavoro aperti appaiono nella barra delle applicazioni, con un'etichetta `Project: name`, stato attivo e un tooltip contenente il percorso completo e il collegamento. Clicca sul pulsante dello spazio di lavoro per tornarvi. La barra delle applicazioni segue il monitor dello spazio di lavoro.

Il percorso della directory canonica determina un'identità stabile dello spazio di lavoro; cambiare l'ordine della lista cambia il collegamento, non lo spazio di lavoro. I progetti con lo stesso nome di directory restano separati e i collegamenti simbolici alla stessa directory vengono deduplicati. Il lancio di terminali e editor punta esplicitamente allo spazio di lavoro del progetto. Gli editor che riutilizzano un processo/finestra esistente possono mantenere il comportamento dello spazio di lavoro proprio di quell'applicazione.

Gli spazi di lavoro aperti prima dell'aggiornamento mantengono le finestre esistenti e i nomi nativi. Aprire un progetto dopo l'aggiornamento utilizza la sua nuova identità derivata dal percorso; chiudere le vecchie finestre del progetto quando conveniente. Non viene tentata alcuna associazione o spostamento automatico perché un vecchio nome non identifica la directory in modo affidabile. In caso di nomi di directory duplicati, utilizzare il selettore, un numero o il percorso canonico completo anziché un nome ambiguo.

Aprire un progetto rigenera `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` in modo atomico dalla lista di progetti configurata e dalle traduzioni. Questi sono metadati di sessione per etichette e suggerimenti degli spazi di lavoro, non configurazione; non modificarli. Riaprire un progetto dopo aver modificato la lista o la lingua per aggiornare le sue etichette. Aggiornare sia `argvus` che `argvus-waybar`, oltre alla configurazione `argvus-taskbar`, per questa integrazione.
