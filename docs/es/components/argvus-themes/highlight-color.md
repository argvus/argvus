---
title: Personalizando el color de resaltado
description: Cambie el color de resaltado de ARGVUS independientemente del tema seleccionado y vuelva al valor predeterminado del tema.
---
El **color de resaltado** es el acento utilizado para los bordes de enfoque, la selección y los estados activos en todo el escritorio. Cada tema tiene un color de resaltado predeterminado, pero puedes reemplazarlo con cualquier color que desees sin cambiar el tema.

## Cambiar el color de resaltado

1. Abre **Centro de Control → Apariencia → Color de resaltado**.
2. Elige **Editar color de resaltado**. El valor actual se muestra junto a la opción.
3. Escribe un color RGB de seis dígitos, con o sin el `#` inicial. Por ejemplo, `3590BD` y `#3590BD` son aceptados.
4. Presiona `Enter` para aplicar.

También puedes pegar un color en el campo. ARGVUS acepta el texto pegado solo cuando es un valor RGB de seis dígitos válido. Formas cortas como `#123` y valores con caracteres no hexadecimales son rechazados con un mensaje de error, y el color no se cambia.

El nuevo color se aplica al escritorio inmediatamente y se guarda en tu configuración de ARGVUS.

## Volver al valor predeterminado del tema

Para volver al color definido por el tema seleccionado, abre **Centro de Control → Apariencia → Color de resaltado** y elige **Restablecer al valor predeterminado del tema**.

## Cómo interactúa el color de resaltado con los temas

- Un color de resaltado que elijas es una anulación independiente. Permanece cuando cambias a otro tema, por lo que el color que elegiste se mantiene igual.
- Un tema que apliques después de **Restablecer al valor predeterminado del tema** toma el color de resaltado nuevamente y usa su valor predeterminado.
- Exportar el tema activo almacena el color de resaltado que está activo al exportar como el acento de la paleta del nuevo tema. Ver [Importar y exportar temas](/docs/components/argvus-themes/import-and-export/)].

## Dónde se almacena el valor

El color destacado es parte de su configuración ARGVUS, en `$XDG_CONFIG_HOME/argvus/config.json` (generalmente `~/.config/argvus/config.json`). No edite los archivos generados bajo `$XDG_CONFIG_HOME/argvus/data/generated`: ARGVUS los reescribe a partir de la configuración.
