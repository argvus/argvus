---
title: Calculatrice ARGVUS
description: Basique · Scientifique · Programmation · Financier · Conversion d’unités
---

# Calculatrice ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Choisissez un profil dans Réglages → Profil avec ↑/↓ et Entrée, ou cliquez sur une ligne. Le profil et ses options sont enregistrés immédiatement et restaurés au prochain lancement. Chaque profil conserve son expression pendant la session. Entrée ou = remplace l’expression par le résultat sans le sélectionner, avec le curseur à la fin.

Les taux sont des pourcentages par période ; utilisez la même période pour taux, nombre et versement. Les versements ont lieu en fin de période. Le taux zéro est accepté. Les intérêts simples et composés donnent le total accumulé. La conversion fonctionne hors ligne, sans devises. L’historique conserve 100 entrées avec profil, options et champs financiers ; choisir un résultat restaure ce contexte. Effacer supprime aussi le fichier enregistré.

## Paramètres

- `s Paramètres` → **Profil** → Basique / Scientifique / Programmation / Financier / Conversion d’unités.
- ↑ / ↓ · Enter · Escape / ←.
- **Copier le résultat** · **Historique** · **Effacer l’historique**.
- **Basique**: 370 × 420; **Scientifique**: 440 × 640.
- **Programmation**: 520 × 640; **Financier**: 440 × 640; **Conversion d’unités**: 440 × 520.

## Basique / Scientifique

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Unité angulaire**: Degrés / Radians. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 chiffres significatifs · 4096 octets · 64 niveaux d’analyse.

## Programmation

Par défaut : décimal, 64 bits signés. L’arithmétique boucle selon la largeur choisie ; le décalage droit signé conserve le signe. La division entière tronque vers zéro. Division par zéro et décalages hors de [0, bits) sont des erreurs. Sans préfixe, la base choisie s’applique. Changer de base convertit l’expression valide ; largeur ou signe réinterprètent les mêmes bits.

- **Base**: 2 / 8 / 10 / 16; **Taille du mot (bits)**: 8 / 16 / 32 / 64.
- **Signé** / **Non signé** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Signé); `-7/2 = -3`.

## Financier

Intérêts simples · Intérêts composés · Versement fixe · Valeur actuelle · Valeur future · Variation en pourcentage.

- `P`: Capital initial; `FV`: Montant futur; `A`: Versement par période.
- `r`: Taux par période (%) / 100; `n`: Nombre de périodes.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Intérêts simples: `P*(1+r*n)`; Intérêts composés: `P*G`.
- Versement fixe: `P*G/H`; Valeur actuelle: `(FV+A*H)/G`; Valeur future: `P*G+A*H`.
- Variation en pourcentage: `(Montant final - Montant initial) / Montant initial * 100`.

## Conversion d’unités

Longueur · Masse · Température · Surface · Volume · Temps · Vitesse · Stockage de données.

**De** / **Vers**: Paramètres · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Historique

100 · Profil + `context` · `Effacer l’historique`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Le texte suit Applications Font dans Control Center → Appearance → Desktop Style → Fonts. Les applications GTK héritent de gtk-font-name ; les tailles des nombres et titres sont conservées. Le lanceur lit apps_family généré et suit les modifications.

L’en-tête associe un badge ARGVUS dans la couleur d’accent au nom de l’application. Le pied cliquable s Settings occupe une bande pleine largeur au fond distinct du thème, suivant la hiérarchie du Control Center.
