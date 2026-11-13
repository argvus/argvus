---
title: Internacionalización ARGVUS
description: Catálogos de idiomas, selección y soporte Unicode.
---

# Internacionalización de ARGVUS

ARGVUS barcos 18 idiomas de interfaz: árabe`ar`Bengali`bn-BD`), alemán (`de-DE`),
Inglés`en-US`), español (`es-ES`), francés (`fr-FR`Hola.`hi-IN`), indonesio
(Asuntos)`id-ID`Italiano`it-IT`), japonés (`ja-JP`Coreano`ko-KR`), brasileño portugués
(Asuntos)`pt-BR`), europeo portugués (`pt-PT`), ruso (`ru-RU`), turco (`tr-TR`), Urdu
(Asuntos)`ur-PK`), vietnamita (`vi-VN`) y chino simplificado (`zh-CN`). El inglés es
catálogo canónico y retroceso. Las traducciones se produjeron con AI asistencia y asistencia
correcciones técnicas; no se ha certificado la revisión independiente del altavoz nativo.

## Seleccionar un idioma

Open Control Center √≥ Preferencias √ Idioma > Región √ Idioma. La página descubre
instalado catálogos y muestra sus nombres nativos. Se salva `ARGVUS_LANGUAGE` dentro
`$XDG_CONFIG_HOME/locale.conf` (default) `~/.config/locale.conf`), independientemente de
`LANG`; no es necesario generar un sistema local. Portugués brasileño`pt-BR`) y
Portugués Europeo`pt-PT`) son opciones separadas.
La preferencia no modifica `/etc/locale.conf` o regionales `LANG` estableciendo.
Se conservan los comentarios y la configuración regional de la categoría. El Panel de Control se reinicia
después del ahorro; iniciar una nueva sesión para aplicar el idioma a lo largo de los componentes del período de sesiones.
Los usuarios inconfigurados usan inglés. Los controles separados de preferencia regional local
`LANG`, formato regional y aplicaciones de terceros. El saludador de inicio de sesión corre
antes de la preferencia del usuario está disponible y conserva su propia cuenta de servicio
idioma, inglés por defecto.
Las variantes regionales utilizan el catálogo instalado para su idioma cuando es exacto
catálogo está ausente; por ejemplo `es_MX.UTF-8` usos `es-ES`. `zh-Hans` selecciones
Chino simplificado. Chino tradicional`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) es
no mapeado silenciosamente a chino simplificado y cae de nuevo al inglés.

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
