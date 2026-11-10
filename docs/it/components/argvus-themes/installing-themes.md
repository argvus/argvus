---
title: Installazione dei temi
description: Installa i pacchetti del tema ufficiale ARGVUS con pacman.
---
I temi ufficiali sono pubblicati come pacchetti Arch Linux. Ogni tema è un pacchetto a sé stante chiamato `argvus-theme-<THEME_NAME>`.

## Dove reperire i temi

La sorgente di ogni tema ufficiale si trova nell'organizzazione del progetto ARGVUS:

- Ogni tema è un repository chiamato `argvus-theme-<THEME_NAME>` (per esempio `argvus-theme-dracula`), costruito come pacchetto con lo stesso nome.

I pacchetti dei temi dipendono da `argvus-appearance`. Installa prima ARGVUS, come descritto in [Installation](/docs/getting-started/installation/), affinché il componente dell'aspetto sia presente.

## Installare un singolo tema

Per installare tutti i temi ufficiali contemporaneamente, installa il metapacchetto `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```
Dipende da tutti i pacchetti pubblicati `argvus-theme-*`. ARGVUS Dark e ARGVUS Light sono già spediti da `argvus-appearance`.

Installa un tema con il suo nome di pacchetto:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
Per esempio:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
Il pacchetto installa il manifesto del tema, `/usr/share/argvus/themes/<theme-id>/theme.toml` (il suo nome, categoria, sfondo e palette, con opzionale `overrides/`), e lo sfondo del tema sotto `/usr/share/backgrounds/argvus/`. I file di ogni componente vengono generati dalla palette quando il tema viene applicato. Non scrive nella tua directory personale e non cambia il tema attuale. Selezionalo dal Centro di Controllo successivamente, vedi [Temi ufficiali](/docs/components/argvus-themes/official-themes/).

## Installare più temi

Non esiste un meta-pacchetto che installi tutti i temi. Installa i temi che desideri con un solo comando:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Pacchetti disponibili

I pacchetti sottostanti sono i pacchetti di temi ufficiali pubblicati oggi.

**Temi scuri**

| Pacchetto | Tema |
| --- | --- |
| `argvus-theme-one-dark` | One Dark |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silver Dark |
| `argvus-theme-slate-dark` | Slate Dark |
| `argvus-theme-universe` | Universe |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé Pine |
| `argvus-theme-tokyo-night` | Tokyo Night |
| `argvus-theme-solitude` | Solitude |
| `argvus-theme-sunset` | Sunset |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**Temi chiari**

| Pacchetto | Tema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Chiaro |
| `argvus-theme-solarized-light` | Solarized Chiaro |
| `argvus-theme-nord-light` | Nord Chiaro |
| `argvus-theme-one-light` | One Chiaro |
| `argvus-theme-everforest-light` | Everforest Chiaro |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Chiaro |
| `argvus-theme-watercolor` | Acquerello |

I temi integrati **ARGVUS Dark** e **ARGVUS Light** non richiedono un pacchetto separato; sono inclusi in `argvus-appearance`.

## Controlla cosa è installato

Elenca i pacchetti di temi installati con:

```bash
pacman -Qs argvus-theme
```
Apri il Centro di Controllo e vai su **Aspetto → Temi → Temi Ufficiali** per vedere i temi attualmente disponibili nel selettore. Un tema che è installato ma manca da quell'elenco è un problema di impacchettamento; consulta la sezione di risoluzione dei problemi della guida [Themes](/docs/components/argvus-themes/).

## Aggiungere un tema proprio

Un tema è un manifesto, quindi un tema proprio non ha bisogno di un pacchetto. Crea `~/.local/share/argvus/themes/<id>/theme.toml`, dove il nome della directory è l'ID del tema (lettere minuscole, cifre e trattini singoli):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
Solo `bg`, `fg` e `accent` sono richiesti: ogni altro colore ha un valore predefinito derivato da essi, e il manifesto ne definisce uno solo per modificarlo. Controllalo, quindi applicalo come qualsiasi altro tema:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` mostra i tuoi temi accanto a quelli ufficiali (vince un tema ufficiale con lo stesso ID). I tuoi temi appaiono nel menu dei temi (`SUPER + Shift + T`); il greeter pre-login non può leggere la tua directory home, quindi mantiene l'ultimo tema ufficiale che hai applicato mentre uno dei tuoi temi è attivo.

Le copie per tema che le versioni precedenti di ARGVUS scrivevano sotto `~/.config/argvus/data/<app>/themes/` non vengono più lette. Il primo cambio di tema dopo l'aggiornamento le sposta in `~/.config/argvus/data/backups/theme-copies-<time>/`; per mantenere un file che hai modificato, mettilo in un tema tuo sotto `overrides/` e nominalo nella tabella `[overrides]` del manifesto.

## Rimuovere un tema

Rimuovi un singolo pacchetto tema con:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
Rimuovere un pacchetto di temi non elimina i profili importati. Se il tema rimosso è quello attualmente applicato, seleziona un altro tema dal Centro di Controllo prima di rimuoverlo.
