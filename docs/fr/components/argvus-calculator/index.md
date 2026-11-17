---
title: Calculatrice ARGVUS
description: Calculatrice ARGVUS — GTK4 / ARGVUS
---

# Calculatrice ARGVUS

`argvus calculator` · launcher: **Calculatrice ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Scientifique**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radians**: Décoché : degrés.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Effacer**; Backspace: **Effacer un caractère**; Tab / Shift+Tab.
- **Copier le résultat**; **Historique**: 100; **Effacer l’historique**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Historique

L’historique est conservé entre les ouvertures (100 résultats maximum). Effacer supprime aussi les données enregistrées. Réglages : ↑/↓ pour naviguer, Entrée ou clic pour activer, Échap pour revenir.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

S’ouvre toujours dans une petite fenêtre flottante centrée (370 × 420) ; le mode scientifique peut l’agrandir.

L’écran principal contient uniquement expression, résultat et clavier. Le menu donne accès au mode scientifique, aux unités angulaires, à la copie et à l’historique séparé. Retour ou Échap revient au calcul ; sélectionner un résultat le réutilise. Le mode basique rétablit la fenêtre compacte et conserve l’expression.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Redimensionner la fenêtre agrandit les touches. Chaque ouverture utilise la taille par défaut. Les écrans secondaires ont une flèche de retour en haut.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Le mode basique ou scientifique choisi est enregistré immédiatement et restauré au prochain lancement.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
