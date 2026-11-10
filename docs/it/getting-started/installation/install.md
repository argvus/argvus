---
title: Installa ARGVUS
description: Installa, aggiorna, rimuovi e configura ARGVUS.
---
ARGVUS è distribuito come pacchetto Arch Linux tramite l'archivio ufficiale dei pacchetti ARGVUS.

## Prima dell'installazione

Abilitare l'archivio ufficiale Arch Linux `multilib` prima di installare ARGVUS. Modificare `/etc/pacman.conf` e decommentare sia la sezione `[multilib]` che la sua riga `Include`:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
Quindi sincronizza i database dei pacchetti ed esegui un aggiornamento completo del sistema:

```sh
sudo pacman -Syu
```
Il programma di installazione rapido configura il repository ARGVUS, ma non abilita `multilib` per te.

## Installazione rapida

Su un'installazione fresca di Arch Linux, puoi installare ARGVUS direttamente da un TTY senza installare prima un altro ambiente desktop.

Esegui:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
L'installatore:

1. Importa e firma localmente la chiave di firma del repository ARGVUS.
2. Configura il repository dei pacchetti ARGVUS.
3. Aggiorna il database dei pacchetti.
4. Installa il pacchetto `argvus` e le sue dipendenze.

Dopo l'installazione, esci e seleziona **ARGVUS** dal tuo display manager.

> **Nota:** L'installatore rapido richiede `curl`, `sudo` e gli strumenti standard di gestione dei pacchetti di Arch Linux.

## Configurare solo il repository

Per configurare il repository dei pacchetti ARGVUS senza installare il desktop, esegui:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
Installa ARGVUS manualmente successivamente:

```sh
sudo pacman -Syu argvus
```
## Installazione manuale

Se preferisci configurare il repository passo dopo passo, usa i seguenti comandi.

### 1. Scarica la chiave di firma del repository

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. Importa la chiave di firma

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. Firma la chiave localmente

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. Configurare il repository ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
Aggiungi la configurazione del repository a `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
La configurazione del repository abilita i repository ARGVUS, inclusi `argvus` e `argvus-extras`, con `SigLevel = Required TrustedOnly`. Pacman accetta i pacchetti solo quando le loro firme sono valide e riconosciute come affidabili dal portachiavi locale.

### 5. Installa ARGVUS

```sh
sudo pacman -Syu argvus
```
Dopo l'installazione, esci e seleziona **ARGVUS** dal tuo gestore di visualizzazione.

## Aggiornamento

Aggiorna ARGVUS insieme al resto di Arch Linux:

```sh
sudo pacman -Syu
```
Per aggiornare solo il pacchetto ARGVUS:

```sh
sudo pacman -S argvus
```

> **Raccomandazione:** Arch Linux generalmente raccomanda un aggiornamento completo del sistema con `sudo pacman -Syu` invece di aggiornare singoli pacchetti.

## Rimuovere ARGVUS

Rimuovere ARGVUS mantenendo le dipendenze che sono ancora necessarie ad altri pacchetti installati:

```sh
sudo pacman -R argvus
```
Rimuovere ARGVUS e le dipendenze che non sono più richieste da nessun pacchetto installato:

```sh
sudo pacman -Rns argvus
```
### Rimuovere il repository ARGVUS

Se vuoi anche rimuovere il repository dei pacchetti, elimina la sua configurazione:

```sh
sudo rm /etc/pacman.d/argvus.conf
```
Quindi rimuovi la riga corrispondente da `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```
Puoi modificare il file con:

```sh
sudo nano /etc/pacman.conf
```
### Rimuovere la chiave di firma

Per rimuovere la chiave di firma del repository ARGVUS dal portachiavi di pacman, prima elenca le chiavi:

```sh
sudo pacman-key --list
```
Quindi rimuovi la chiave ARGVUS con il suo ID chiave:

```sh
sudo pacman-key --delete KEY_ID
```
Sostituire `KEY_ID` con l'ID della chiave di firma ARGVUS.

> **Avviso:** Rimuovere questa chiave solo quando si sta rimuovendo anche il repository ARGVUS. Non rimuovere chiavi ancora necessarie ad altri repository.

## Configurazione utente

L'installazione del pacchetto non scrive la configurazione in `$HOME`. ARGVUS legge le impostazioni predefinite del pacchetto da `/usr/share/argvus` e funziona senza copiarle nella tua directory home.

La configurazione a runtime segue questa priorità generale:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` è l'unico componente che scrive `~/.config/argvus/data/generated/`, e quella proiezione è il livello attivo: una copia gestita o nativa entra in vigore solo mentre il file generato corrispondente è assente.

La configurazione gestita da ARGVUS si trova sotto `~/.config/argvus/`. Un'opzione di override `$XDG_CONFIG_HOME/<app>` rimane disponibile per le applicazioni che supportano una configurazione utente nativa.

Il dispatcher `argvus --setup` è opzionale ed è per override manuali espliciti, non per il recupero. Se `~/.config/argvus` viene eliminato, `SUPER + SHIFT + R` ricrea il profilo canonico e lo stato runtime generato attraverso la normale pipeline di ricarica.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` è una lista bianca esplicita dei file di profilo TUI supportati; non clona `/usr/share/argvus` e non copia script, servizi, risorse, file generati o `config.json`. Esegui `argvus --setup --help` per vedere le opzioni correnti. Una volta copiati, i file diventano override di proprietà dell'utente e gli aggiornamenti del pacchetto non li sostituiscono.

Per sostituire una configurazione copiata esistente, usa `--force`. ARGVUS crea un backup con data e ora prima di sostituirlo:

```sh
argvus --setup --copy <app> --force
```
## Sovrascritture utente di Hyprland

Per la guida completa alle sovrascritture di Hyprland, inclusa la differenza tra una configurazione nativa completa e frammenti incrementali ARGVUS, vedi [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### Backup della macchina virtuale

Alcune macchine virtuali possono riscontrare problemi di rendering con Hyprland, Kitty o Quickshell. ARGVUS rileva le macchine virtuali utilizzando il driver grafico `vmwgfx` e abilita il rendering software quando necessario.

Se il rilevamento non funziona sulla tua macchina virtuale, abilita manualmente il backup in `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Dopo aver creato o modificato `user.lua`, esci da Hyprland e accedi di nuovo in modo che la variabile venga applicata alle applicazioni avviate dalla sessione.

Verificalo con:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
L'output previsto è:

```text
1
```

> **Nota:** `LIBGL_ALWAYS_SOFTWARE=1` obbliga le applicazioni Mesa/OpenGL a usare il rendering software, solitamente tramite `llvmpipe`. Può ridurre le prestazioni grafiche e dovrebbe essere usato come soluzione di riserva per le macchine virtuali interessate.
