---
title: Personalizzare il colore di evidenziazione
description: Cambia il colore di evidenziazione di ARGVUS indipendentemente dal tema selezionato e ritorna al valore predefinito del tema.
---
Il **colore di evidenziazione** è l'accento utilizzato per i bordi di messa a fuoco, la selezione e gli stati attivi sul desktop. Ogni tema ha un colore di evidenziazione predefinito, ma puoi sostituirlo con qualsiasi colore tu voglia senza cambiare il tema.

## Cambiare il colore di evidenziazione

1. Apri **Centro di Controllo → Aspetto → Colore di evidenziazione**.
2. Scegli **Modifica colore di evidenziazione**. Il valore corrente è mostrato accanto all'opzione.
3. Digita un colore RGB a sei cifre, con o senza il prefisso `#`. Ad esempio `3590BD` e `#3590BD` sono entrambi accettati.
4. Premi `Enter` per applicare.

Puoi anche incollare un colore nel campo. ARGVUS accetta il testo incollato solo se è un valore RGB valido a sei cifre. Le forme brevi come `#123` e i valori con caratteri non esadecimali vengono rifiutati con un messaggio di errore e il colore non viene modificato.

Il nuovo colore viene applicato immediatamente al desktop e salvato nella configurazione di ARGVUS.

## Tornare al colore predefinito del tema

Per tornare al colore definito dal tema selezionato, apri **Centro di Controllo → Aspetto → Colore evidenziazione** e scegli **Reimposta sul tema predefinito**.

## Come il colore evidenziazione interagisce con i temi

- Un colore evidenziazione scelto è una sovrascrittura indipendente. Rimane anche quando si passa a un altro tema, quindi il colore selezionato resta lo stesso.
- Un tema applicato dopo **Reimposta sul tema predefinito** prende nuovamente il controllo del colore evidenziazione e utilizza il proprio predefinito.
- Esportare il tema attivo memorizza il colore evidenziazione attivo al momento dell'esportazione come accento della nuova palette del tema. Vedi [Importazione ed esportazione dei temi](/docs/components/argvus-themes/import-and-export/).

## Dove viene memorizzato il valore

Il colore evidenziato fa parte della tua configurazione ARGVUS, in `$XDG_CONFIG_HOME/argvus/config.json` (di solito `~/.config/argvus/config.json`). Non modificare i file generati sotto `$XDG_CONFIG_HOME/argvus/data/generated`: ARGVUS li riscrive dalla configurazione.
