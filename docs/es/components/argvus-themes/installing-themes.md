---
title: Instalando temas
description: Instala los paquetes oficiales del tema ARGVUS con pacman.
---
Los temas oficiales se publican como paquetes de Arch Linux. Cada tema es su propio paquete llamado `argvus-theme-<THEME_NAME>`.

## Dónde obtener temas

La fuente de cada tema oficial se encuentra en la organización del proyecto ARGVUS:

- Cada tema es un repositorio llamado `argvus-theme-<THEME_NAME>` (por ejemplo, `argvus-theme-dracula`), construido como un paquete con el mismo nombre.

Los paquetes de temas dependen de `argvus-appearance`. Instala ARGVUS primero, como se describe en [Installation](/docs/getting-started/installation/), para que el componente de apariencia esté presente.

## Instalar un solo tema

Para instalar todos los temas oficiales a la vez, instala el metapaquete `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```
Depende de todos los paquetes publicados `argvus-theme-*`. ARGVUS Dark y ARGVUS Light ya son enviados por `argvus-appearance`.

Instala un tema con su nombre de paquete:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
Por ejemplo:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
El paquete instala el manifiesto del tema, `/usr/share/argvus/themes/<theme-id>/theme.toml` (su nombre, categoría, fondo de pantalla y paleta, con opcional `overrides/`), y el fondo de pantalla del tema bajo `/usr/share/backgrounds/argvus/`. Los archivos de cada componente se generan a partir de la paleta cuando se aplica el tema. No escribe en su directorio personal, y no cambia su tema actual. Selecciónelo desde el Centro de Control después, vea [temas oficiales](/docs/components/argvus-themes/official-themes/).

## Instalar varios temas

No hay un meta-paquete que instale todos los temas. Instale los temas que desee en un solo comando:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Paquetes disponibles

Los paquetes a continuación son los paquetes de tema oficiales publicados hoy.

**Temas oscuros**

| Paquete | Tema |
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

**Temas claros**

| Paquete | Tema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Light |
| `argvus-theme-solarized-light` | Solarized Light |
| `argvus-theme-nord-light` | Nord Light |
| `argvus-theme-one-light` | One Light |
| `argvus-theme-everforest-light` | Everforest Light |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Light |
| `argvus-theme-watercolor` | Watercolor |

Los temas integrados **ARGVUS Dark** y **ARGVUS Light** no necesitan un paquete separado; vienen con `argvus-appearance`.

## Verificar qué está instalado

Enumera los paquetes de temas instalados con:

```bash
pacman -Qs argvus-theme
```
Abre el Centro de Control y ve a **Apariencia → Temas → Temas Oficiales** para ver los temas que están actualmente disponibles en el selector. Un tema que está instalado pero no aparece en esa lista es un problema de empaquetado; consulta la sección de resolución de problemas de la [guía de Temas](/docs/components/argvus-themes/).

## Añadir un tema propio

Un tema es un manifiesto, por lo que un tema propio no necesita paquete. Crea `~/.local/share/argvus/themes/<id>/theme.toml`, donde el nombre del directorio es el ID del tema (letras minúsculas, dígitos y guiones simples):

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
Solo `bg`, `fg` y `accent` son necesarios: cada otro color tiene un valor predeterminado derivado de ellos, y el manifiesto define uno solo para cambiarlo. Verifícalo, luego aplícalo como cualquier otro tema:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` muestra tus temas junto a los oficiales (un tema oficial con el mismo ID tiene prioridad). Tus temas aparecen en el menú de temas (`SUPER + Shift + T`); el saludo previo al inicio de sesión no puede leer tu directorio personal, por lo que mantiene el último tema oficial que aplicaste mientras uno de tus temas está activo.

Las copias por tema que las versiones antiguas de ARGVUS escribían en `~/.config/argvus/data/<app>/themes/` ya no se leen. El primer cambio de tema después de la actualización las mueve a `~/.config/argvus/data/backups/theme-copies-<time>/`; para conservar un archivo que editaste, colócalo en un tema propio bajo `overrides/` y asígnale un nombre en la tabla `[overrides]` del manifiesto.

## Eliminar un tema

Elimina un solo paquete de tema con:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
Eliminar un paquete de tema no borra tus perfiles importados. Si el tema eliminado es el que está aplicado actualmente, selecciona otro tema desde el Centro de Control antes de eliminarlo.
