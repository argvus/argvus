---
title: Analizador de Uso de Disco ARGVUS
description: Explora el uso del disco de manera interactiva con ncdu en una ventana de Kitty con tema.
---
Abre **ARGVUS Disk Usage Analyzer** desde el lanzador de aplicaciones o ejecuta:

```sh
argvus du
```
El analizador abre una ventana dedicada de Kitty centrada e incrusta la interfaz real de ncdu dentro de su marco ARGVUS. Comienza en `$HOME`. Edita la ruta del directorio o elige **Examinar** para abrir el administrador de archivos configurado. Con Superfile, navega a un directorio y presiona **Espacio** para seleccionarlo; `q` o `Esc` cancela. Otros administradores de archivos configurados se abren normalmente, pero pueden no devolver un directorio seleccionado, así que ingresa o pega la ruta en el analizador.

Usa Tab o las teclas de flecha para moverte entre el directorio, las opciones y las acciones. El gráfico, el porcentaje, el esquema de color oscuro y el orden de directorios primero están habilitados por defecto. Las opciones restantes de escaneo y visualización se pueden alternar antes de seleccionar **Analizar**. En Resultados, usa los controles de teclado de ncdu; `q` regresa al formulario y `Esc` detiene el escaneo. Presiona `d` para eliminar el archivo o directorio seleccionado. ncdu pide confirmación; la eliminación es permanente y utiliza los permisos del usuario actual. No se solicitan privilegios elevados.

La interfaz externa sigue el tema activo ARGVUS y la configuración regional. Se requieren Kitty y ncdu; el paquete `argvus-devtools` proporciona ncdu. Si falta Kitty o ncdu, la aplicación informa la dependencia faltante.
