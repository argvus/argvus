---
title: Internacionalización ARGVUS
description: Catálogos de idiomas, selección y soporte Unicode.
---

# Internacionalización de ARGVUS

ARGVUS incluye en inglés (`en-US`), portugués brasileño (`pt-BR`), español (`es-ES`), francés (`fr-FR`), italiano (`it-IT`), alemán (`de-DE`), ruso (`ru-RU`), chino simplificado (`zh-CN`), japonés (`ja-JP`) y coreano (`ko-KR`). El inglés es el catálogo canónico y de respaldo. Las ocho traducciones adicionales fueron producidas con asistencia de traducción automática y correcciones técnicas; no se ha certificado revisión independiente por hablantes nativos.

## Seleccionar un idioma

Abra el Centro de Control > Preferencias > Idioma y Región > Idioma, o Configuración > Idioma. La página detecta los catálogos instalados y muestra sus nombres nativos. Guarda `LANG` en `$XDG_CONFIG_HOME/locale.conf` (predeterminado `~/.config/locale.conf`). Primero habilite la localización del sistema UTF-8 correspondiente en Localizaciones del Sistema. Los ejemplos incluyen `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` y `ko_KR.UTF-8`.

La preferencia no modifica `/etc/locale.conf`. Se conservan los comentarios y la configuración de la categoría regional. El Panel de Control se reinicia después de guardar; cierre sesión y vuelva a iniciarla para aplicar el idioma en toda la sesión. Los usuarios no configurados usan inglés. Las variables de entorno heredadas y los archivos de idioma ARGVUS antiguos no reemplazan esta preferencia. El saludo de inicio se ejecuta antes de que la preferencia del usuario esté disponible y mantiene su propio idioma de cuenta de servicio, inglés por defecto.

Las variantes regionales usan el catálogo instalado para su idioma cuando falta un catálogo exacto; por ejemplo, `es_MX.UTF-8` usa `es-ES`. `zh-Hans` selecciona chino simplificado. El chino tradicional (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) no se asigna silenciosamente al chino simplificado y vuelve al inglés.

## Catálogos y contribuciones

La fuente es `resources/locales/<locale>/<domain>.json` en el repositorio `argvus`. Los catálogos instalados se encuentran en `/usr/share/argvus/i18n/`. Se envían en el paquete maestro `argvus`. `locale.json` proporciona el identificador y los nombres nativos/de visualización. No hay un catálogo PO separado ni una plataforma de traducción en línea requerida.

Use claves semánticas y conserve los nombres de marcadores de posición, comandos, opciones, rutas y atajos de teclado. Traduce mensajes completos en lugar de ensamblar oraciones a partir de fragmentos traducidos. Las nuevas claves canónicas necesitan traducciones en los diez idiomas. Mantenga la terminología consistente dentro de cada dominio y revise las traducciones en contexto, especialmente la autenticación, los permisos y las acciones destructivas.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```
`ARGVUS_I18N_DIR` selecciona una raíz de catálogo alternativa para el desarrollo y la comprobación de paquetes en fase de prueba. La validación verifica dominios, claves, valores de cadenas, marcadores de posición, traducciones vacías y formas plurales de enteros específicas de la localidad. La validación demuestra la integridad estructural, no la precisión lingüística.

## Unicode y fuentes

`argvus-fonts` requiere las fuentes Noto CJK además de las familias Latin, símbolo y emoji. Fontconfig proporciona una fuente de reserva para chino, japonés y hangul sin reemplazar la familia primaria elegida por el usuario. Las aplicaciones y terminales aún necesitan habilitar la fuente de reserva.

Los helpers de texto compartido de Ratatui miden las celdas del terminal, preservan los clústeres de grafemas extendidos durante la truncación y ajustan el texto ancho. Los editores de texto mueven y eliminan grafemas completos. Las áreas extremadamente estrechas no pueden mostrar un glifo de dos celdas en una sola; el helper de ajuste forzado utiliza un carácter de reemplazo en ese caso. Los emojis y los caracteres de ancho ambiguo aún pueden diferir entre implementaciones de terminal/fuente. La política de ancho del terminal debería coincidir con el ancho Unicode normal de Ratatui.

La selección de idioma no instala un método de entrada ni cambia la distribución del teclado. Use un método de entrada compatible con Wayland al introducir chino, japonés o coreano.
