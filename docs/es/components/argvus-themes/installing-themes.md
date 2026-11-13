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
| `argvus-theme-one-dark` | Uno oscuro |
| `argvus-theme-dracula` | Drácula |
| `argvus-theme-silver-dark` | Plata Oscuro |
| `argvus-theme-slate-dark` | Pizarra Oscura |
| `argvus-theme-universe` | Universo |
| `argvus-theme-gruvbox-high-dark` | Gruvbox Alto Oscuro |
| `argvus-theme-gruvbox-dark` | Gruvbox Oscuro |
| `argvus-theme-rose-pine` | Pino rosado |
| `argvus-theme-tokyo-night` | Noche de Tokio |
| `argvus-theme-solitude` | Soledad |
| `argvus-theme-sunset` | Puesta de sol |
| `argvus-theme-matrix` | Matriz |
| `argvus-theme-monokai-dark` | Monokai Oscuro |

**Temas claros**

| Paquete | Tema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Luz |
| `argvus-theme-solarized-light` | Luz Solarizada |
| `argvus-theme-nord-light` | Luz del Norte |
| `argvus-theme-one-light` | Una luz |
| `argvus-theme-everforest-light` | Luz del bosque eterno |
| `argvus-theme-catppuccin-latte` | Catuchino Latte |
| `argvus-theme-gruvbox-light` | Luz Gruvbox |
| `argvus-theme-watercolor` | Acuarela |

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

## Instalar temas desde Git

Instala un repositorio del skeleton mediante HTTPS o SSH con tus credenciales de Git. La instalación es por usuario y no activa el tema. `--ref` selecciona una rama o etiqueta; `--replace` sustituye un tema personalizado existente. Los IDs oficiales instalados están protegidos.

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

El tema aparece en **Custom themes**; pulsa `r` para actualizar. Aplica otro tema antes de eliminar el activo en el CLI o Control Center. Vuelve a aplicarlo tras sustituirlo.

Debe existir un único tema schema 2 en `src/usr/share/argvus/themes/<id>/`, con ID coincidente. Incluye las imágenes en el tema o en `src/usr/share/`. No se ejecutan scripts. No se admiten submódulos ni Git LFS. Instala `git` y `openssh` para SSH. Destino: `$XDG_DATA_HOME/argvus/themes/<id>`. Los paquetes oficiales usan pacman.
