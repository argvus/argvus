---
title: Instalar ARGVUS
description: Instalar, actualizar, eliminar y configurar ARGVUS.
---
ARGVUS se distribuye como un paquete de Arch Linux a través del repositorio oficial de paquetes de ARGVUS.

## Antes de instalar

Habilite el repositorio oficial de Arch Linux `multilib` antes de instalar ARGVUS. Edite `/etc/pacman.conf` y descomente tanto la sección `[multilib]` como su línea `Include`:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
Luego sincroniza las bases de datos de paquetes y realiza una actualización completa del sistema:

```sh
sudo pacman -Syu
```
El instalador rápido configura el repositorio ARGVUS, pero no habilita `multilib` por ti.

## Instalación rápida

En una instalación nueva de Arch Linux, puedes instalar ARGVUS directamente desde un TTY sin instalar primero otro entorno de escritorio.

Ejecuta:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
El instalador:

1. Importa y firma localmente la clave de firma del repositorio ARGVUS.
2. Configura el repositorio de paquetes ARGVUS.
3. Actualiza la base de datos de paquetes.
4. Instala el paquete `argvus` y sus dependencias.

Después de la instalación, cierra sesión y selecciona **ARGVUS** desde tu gestor de inicio de sesión.

> **Nota:** El instalador rápido requiere `curl`, `sudo` y las herramientas estándar de gestión de paquetes de Arch Linux.

## Configurar solo el repositorio

Para configurar el repositorio de paquetes ARGVUS sin instalar el escritorio, ejecuta:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
Instale ARGVUS manualmente después:

```sh
sudo pacman -Syu argvus
```
## Instalación manual

Si prefieres configurar el repositorio paso a paso, usa los siguientes comandos.

### 1. Descargar la clave de firma del repositorio

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. Importar la clave de firma

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. Firmar la clave localmente

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. Configurar el repositorio ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
Agregue la configuración del repositorio a `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
La configuración del repositorio habilita los repositorios ARGVUS, incluidos `argvus` y `argvus-extras`, con `SigLevel = Required TrustedOnly`. Pacman acepta paquetes solo cuando sus firmas son válidas y están confiadas por el llavero local.

### 5. Instalar ARGVUS

```sh
sudo pacman -Syu argvus
```
Después de la instalación, cierra sesión y selecciona **ARGVUS** desde tu gestor de pantalla.

## Actualización

Actualiza ARGVUS junto con el resto de Arch Linux:

```sh
sudo pacman -Syu
```
Para actualizar solo el paquete ARGVUS:

```sh
sudo pacman -S argvus
```

> **Recomendación:** Arch Linux generalmente recomienda una actualización completa del sistema con `sudo pacman -Syu` en lugar de actualizar paquetes individuales.

## Eliminar ARGVUS

Elimine ARGVUS mientras mantiene las dependencias que todavía son necesarias por otros paquetes instalados:

```sh
sudo pacman -R argvus
```
Elimine ARGVUS y las dependencias que ya no son requeridas por ningún paquete instalado:

```sh
sudo pacman -Rns argvus
```
### Eliminar el repositorio ARGVUS

Si también desea eliminar el repositorio del paquete, elimine su configuración:

```sh
sudo rm /etc/pacman.d/argvus.conf
```
Luego elimine la línea correspondiente de `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```
Puedes editar el archivo con:

```sh
sudo nano /etc/pacman.conf
```
### Eliminar la clave de firma

Para eliminar la clave de firma del repositorio ARGVUS del llavero de pacman, primero liste las claves:

```sh
sudo pacman-key --list
```
Luego elimina la clave ARGVUS con su ID de clave:

```sh
sudo pacman-key --delete KEY_ID
```
Reemplace `KEY_ID` con el ID de clave de firma ARGVUS.

> **Advertencia:** Elimine esta clave solo cuando también esté eliminando el repositorio ARGVUS. No elimine claves que todavía sean necesarias para otros repositorios.

## Configuración de usuario

La instalación del paquete no escribe la configuración en `$HOME`. ARGVUS lee los valores predeterminados del paquete desde `/usr/share/argvus` y funciona sin copiarlos en su directorio personal.

La configuración en tiempo de ejecución sigue esta prioridad general:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` es el único componente que escribe `~/.config/argvus/data/generated/`, y esa proyección es la capa activa: una copia administrada o nativa solo tiene efecto mientras el archivo generado correspondiente esté ausente.

La configuración administrada por ARGVUS se encuentra bajo `~/.config/argvus/`. Una anulación opcional `$XDG_CONFIG_HOME/<app>` permanece disponible para aplicaciones que admiten una configuración de usuario nativa.

El despachador `argvus --setup` es opcional y es para anulaciones manuales explícitas, no para recuperación. Si se elimina `~/.config/argvus`, `SUPER + SHIFT + R` recrea el perfil canónico y el estado de ejecución generado a través del proceso normal de recarga.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` es una lista de permitidos explícita de los archivos de perfil TUI compatibles; no clona `/usr/share/argvus` y no copia scripts, servicios, recursos, archivos generados o `config.json`. Ejecute `argvus --setup --help` para ver las opciones actuales. Una vez copiados, los archivos se convierten en sobrescrituras propiedad del usuario y las actualizaciones del paquete no los reemplazan.

Para reemplazar una configuración copiada existente, use `--force`. ARGVUS crea una copia de seguridad con marca de tiempo antes de reemplazarla:

```sh
argvus --setup --copy <app> --force
```
## Anulaciones de usuario de Hyprland

Para la guía completa de anulaciones de Hyprland, incluyendo la diferencia entre una configuración nativa completa y fragmentos incrementales ARGVUS, consulte [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### Respaldo para máquina virtual

Algunas máquinas virtuales pueden experimentar problemas de renderizado con Hyprland, Kitty o Quickshell. ARGVUS detecta máquinas virtuales usando el controlador gráfico `vmwgfx` y habilita el renderizado por software cuando es necesario.

Si la detección no funciona en tu máquina virtual, habilita el respaldo manualmente en `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Después de crear o modificar `user.lua`, cierra sesión en Hyprland y vuelve a iniciarla para que la variable se aplique a las aplicaciones lanzadas por la sesión.

Verifícalo con:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
La salida esperada es:

```text
1
```
> **Nota:** `LIBGL_ALWAYS_SOFTWARE=1` obliga a las aplicaciones Mesa/OpenGL a usar renderizado por software, usualmente a través de `llvmpipe`. Puede reducir el rendimiento gráfico y debería usarse como una solución alternativa para las máquinas virtuales afectadas.
