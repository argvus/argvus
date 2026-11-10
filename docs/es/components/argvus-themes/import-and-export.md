---
title: Importar y exportar temas
description: Exporta el tema activo como un tema propio, importa un archivo de tema, aplícalo y elimínalo.
---
La exportación y la importación funcionan con **temas**, en el mismo formato que los temas oficiales. Exportar guarda el tema activo, con tu color de resaltado y fondo de pantalla, como un nuevo tema propio. Importar instala un archivo de tema entre tus temas. Ambos están en **Centro de Control → Apariencia → Temas**, y tus temas se enumeran en **Temas Personalizados**.

## Qué contiene un tema exportado

El archivo, `argvus-theme-<id>.tar.gz`, contiene un directorio de tema, organizado como un tema oficial:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```
El manifiesto se basa en el tema activo, con tu nombre, un nuevo id, tu color destacado como el acento de la paleta y el fondo de pantalla al lado. El perfil de apariencia conserva la configuración visual actualmente aplicada en ARGVUS, incluyendo el espaciado de las ventanas, los bordes, los márgenes de la barra de tareas, la presentación del panel, los colores, los efectos y las fuentes. El fondo de pantalla personalizado y el color de acento se almacenan con el tema. Aplicar cualquier tema vuelve a aplicar su perfil completo (o los valores predeterminados de ARGVUS cuando no existe un perfil), reemplazando los cambios visuales independientes realizados desde la última vez que se aplicó. Cambiar entre Sticky o Spaced siempre vuelve a aplicar la geometría de ventanas y barra de tareas de ese modo; guarda la geometría personal en un tema personalizado exportado para mantenerla disponible cuando se aplique ese tema. Los ajustes no visuales, como el audio, el teclado y las preferencias de red, no forman parte de un tema.

La apariencia aplicada persiste a través de reinicios de sesión y recargas. La preparación de la sesión reconstruye los archivos necesarios a partir de la configuración actual sin restaurar la geometría del tema original.

Los ajustes cambiados después de aplicar un tema son temporales en el siguiente cambio de tema o modo. Exporta el tema desde **Centro de Control → Apariencia → Temas → Exportar tema** para preservar la configuración visual actual y restaurarla más tarde aplicando ese tema exportado.

Tus temas están instalados en `~/.local/share/argvus/themes/<id>/`. Extraer un archivo allí también instala el tema.

## Exportar el tema activo

1. Abre **Centro de Control → Apariencia → Temas**.
2. Elige **Exportar tema**, o presiona `e` en una página de Temas.
3. Escribe un nombre para el tema. El campo muestra el archivo que se escribirá.
4. Presiona `Enter`.

El id se crea a partir del nombre: letras minúsculas, dígitos y guiones. Cuando el id ya está en uso por un tema instalado o por un archivo en tu directorio personal, se añaden `-2`, `-3`, y así sucesivamente. El archivo se escribe en tu directorio personal, por ejemplo `~/argvus-theme-my-theme.tar.gz`. ARGVUS verifica el tema antes de escribirlo: el manifiesto debe cargarse y cada componente debe renderizar en ambos modos.

## Importar un tema

1. Copia el archivo `.tar.gz` en tu directorio personal (`~`). ARGVUS lista los archivos allí que contienen un tema.
2. Abre **Centro de Control → Apariencia → Temas → Importar tema**, o presiona `i` en una página de Temas.
3. Selecciona el archivo y presiona `Enter`.

ARGVUS verifica el archivo antes de instalarlo. Debe contener un único directorio con un `theme.toml`, solo con rutas relativas (sin `..`, sin enlaces), con un máximo de 128 entradas y un límite de tamaño por archivo; el id debe coincidir con el directorio, el fondo de pantalla debe estar en el archivo y cada componente debe mostrarse en ambos modos. Un perfil de apariencia incluido debe usar secciones compatibles y JSON válido. Si alguna verificación falla, no se instala nada.

El tema se agrega a **Temas Personalizados**. Importarlo no lo aplica: eliges cuándo.

### Colisiones de nombres

- Si uno de tus temas tiene el mismo id, ARGVUS pregunta **¿Reemplazar tu tema con el mismo ID?**. Elige **Reemplazar** para sustituirlo, o presiona `Esc` para conservar el tuyo. El tema reemplazado se conserva como respaldo hasta que se instale el nuevo.
- Se rechaza un archivo con el id de un tema oficial. Expórtalo nuevamente con otro nombre.

## Aplicar un tema personalizado

1. Abre **Centro de Control → Apariencia → Temas → Temas Personalizados**.
2. Selecciona el tema y presiona `Enter`.

Se aplica la configuración completa exportada del tema, incluyendo su fondo de pantalla, espaciado, bordes, color de acento, efectos, fuentes y presentación del panel. Sus valores visuales reemplazan la configuración actual. ARGVUS luego reinicia la barra de tareas, el servicio de recarga de Hyprland, el fondo de pantalla y el Panel de Control para que cada superficie use el tema personalizado.

## Eliminar un tema personalizado

1. Abre **Centro de Control → Apariencia → Temas → Temas Personalizados**.
2. Selecciona el tema y presiona `d`.
3. Confirma con `y`, o presiona `n` o `Esc` para cancelar.

El directorio del tema se elimina de `~/.local/share/argvus/themes`. Si era el tema activo, ARGVUS cambia primero a **ARGVUS Dark**.

## Perfiles de versiones anteriores

Las versiones anteriores exportaban *perfiles de tema* (`profile.tar.gz`), una copia de la configuración de apariencia. La primera vez que se abre una página de Temas, cada perfil en `~/.local/share/argvus/themes/<name>/` se convierte en un tema hecho a partir de su tema base, con su color destacado y fondo de pantalla. El perfil original y la antigua lista `custom-themes.json` se trasladan a `~/.config/argvus/data/backups/theme-profiles-<seconds>/`, y la línea de estado informa sobre la conversión. Un perfil cuyo tema base no está instalado solo se mueve a la copia de seguridad.

## Compartir un tema

Un tema exportado es un archivo regular. Cópialo a otra máquina que ejecute ARGVUS e impórtalo allí.

## Solución de problemas

- **El archivo no está listado.** Debe estar directamente dentro de tu directorio personal, terminar con `.tar.gz` y contener `<id>/theme.toml`.
- **La importación es rechazada.** La línea de estado indica la razón: un tema oficial con el mismo id, un fondo de pantalla faltante, un manifiesto que no se carga o un componente que no se renderiza.
- **La exportación es rechazada.** El nombre necesita al menos una letra o un dígito, y el tema activo debe estar instalado.
