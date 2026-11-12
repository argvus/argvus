---
title: Centro de control
description: Configure las partes persistentes de su escritorio ARGVUS.
---
El Centro de Control es la aplicación de configuración de ARGVUS diseñada para usar principalmente con el teclado. Úsalo cuando quieras configurar el escritorio en lugar de realizar una acción única.

## Centro de Control y Panel de Control

| Usar… | Para… |
| --- | --- |
| **Centro de Control** | Temas, fondos de pantalla, efectos, disposición, comportamiento de la barra de tareas, fuentes, aplicaciones predeterminadas, configuración de teclado e idioma, configuración de idioma, región y sistema. |
| **Panel de Control** | Estado actual y acciones frecuentes como volumen, brillo, red, notificaciones, controles de energía y sesión. |

El Panel de Control puede mostrar controles rápidos para una configuración que se haya configurado permanentemente en el Centro de Control. Por ejemplo, usa el panel para una acción rápida de apariencia o pantalla, y el Centro de Control cuando quieras cambiar la configuración del escritorio que está detrás. Consulta Panel de Control y [Dónde configurar cosas](/docs/user-guide/where-to-configure/).

## Áreas principales

El hogar actual del Centro de Control proporciona estas áreas:

El hogar es un catálogo de destinos agrupados en seis tarjetas. Cada destino abre su página directamente, y la búsqueda lo encuentra bajo cualquier tarjeta.
- **Apariencia** — personaliza el escritorio: Estilo de Escritorio (tema, modo de apariencia, color de acento, fondos de pantalla, íconos y fuentes), Estilo de Aplicación (transparencia del terminal y del lanzador), Barra de Tareas, Panel de Control y Telemetría de Widgets.
- **Escritorio y Entrada** — configura el espacio de trabajo: Hyprland (Windows: Reglas, Espacios y Modo, animaciones, desenfoque, bordes, sombra y la versión instalada), Pantallas, Teclado y Atajos y Ratón y Panel Táctil.
- **Preferencias** — configuraciones personales y regionales: Idioma y Región (el idioma, zona horaria, formatos y centro de distribución del diseño del teclado), Fecha y Hora, Aplicaciones Predeterminadas, Energía y Sesión.
- **Hardware y Conectividad** — dispositivos y conexiones: Resumen de Hardware, Red y Wi-Fi, Bluetooth, Audio y Almacenamiento.
- **Administración del Sistema** — gestionar el sistema operativo: Paquetes, Servicios, Arranque, Nombre del Host, Usuarios y Grupos, Cortafuegos y Notificaciones (No Molestar).
- **Configuración** — el propio Centro de Control, listado después de las tarjetas: Idioma (el idioma del usuario) y Estilo (iconos decorativos y transparencia). Presione `s` para abrirlo.
- **Herramientas y Diagnósticos** — herramientas de desarrollo y mantenimiento: Bienvenida a ARGVUS, Proyectos, Fragmentos (texto guardado para `argvus snippets`, escrito con `SUPER + ALT + N`), claves SSH (pares de claves en `~/.ssh` administrados por `argvus keys`), claves GPG (el llavero administrado por `argvus gpg`; gpg-agent usa pinentry-gtk), Servicios de Desarrollo, Doctor del Sistema y Acerca de ARGVUS.

Cuando No Molestar está habilitado, las ventanas emergentes de notificación se cierran y las nuevas notificaciones se guardan para más tarde. El Panel de Control oculta su historial de notificaciones hasta que No Molestar se desactive.

## Bienvenida a ARGVUS

Elija **Open ARGVUS Welcome** en **Herramientas y Diagnósticos**, busque `welcome` o ejecute `argvus welcome`. Welcome está integrado en el Centro de Control y utiliza su ventana única. Enter abre la página de configuración seleccionada; cuando termine, regrese a la guía con Esc. F6 Siguiente marca el paso actual y abre el siguiente en la misma ventana. Esc solo vuelve atrás, y Enter en la lista de verificación siempre abre la página seleccionada. O revisa el paso nuevamente, S omite sin marcarlo, y F finaliza la guía incluso si hay pasos opcionales pendientes. Cancelar o regresar no completa un paso. El progreso existente se conserva.

La guía cubre idioma y región, pantallas, hardware y controladores, red, audio, Bluetooth, apariencia, energía, aplicaciones predeterminadas e inicio de sesión de sesión. Nunca instala controladores ni paquetes y no aplica cambios en el sistema por sí misma. El progreso se guarda en `$XDG_STATE_HOME/argvus/welcome/progress.json` (o `~/.local/state/argvus/welcome/progress.json`).

Después de que el escritorio está listo, la sesión ejecuta `argvus welcome --if-needed` para TTY, ARGVUS Greeter y otros gestores de inicio de sesión. Abre Bienvenida a menos que los registros de progreso por usuario `opened`, completación o cierre. La primera renderización exitosa guarda `opened` en `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, evitando su reapertura en inicios de sesión posteriores. No depende de `/etc/skel`. Las cuentas existentes sin ese registro también reciben una apertura automática. Los fallos antes de la renderización permiten otro intento en el siguiente inicio de sesión. `argvus welcome` manual permanece disponible.

Las páginas exactas del proveedor pueden depender de los paquetes instalados y de los permisos. No asuma que un dominio opcional que falta significa un Centro de Control roto.

Cuando la cuadrícula tiene más entradas de las que caben en la pantalla, aparece una barra de desplazamiento a lo largo del borde derecho para indicar que hay más contenido debajo.

## Encontrando una configuración

La pantalla de inicio registra las configuraciones por título, categoría, identificador y palabras clave. La búsqueda no distingue entre mayúsculas y minúsculas y acepta términos prácticos como `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` y `DPI`. Las coincidencias exactas de título se clasifican primero, seguidas de un prefijo de título, cualquier palabra del título o de la categoría que contenga el texto escrito, y finalmente las coincidencias de palabras clave; por lo que escribir cualquier parte de una palabra (no solo su inicio) aún puede encontrar el destino.

El menú visible es consciente de las capacidades. Una ruta puede estar ausente cuando su proveedor no está instalado, cuando el hardware no está disponible o cuando la sesión actual no puede proporcionar la capacidad requerida. Por lo tanto, la búsqueda también es una forma útil de descubrir lo que esta instalación puede configurar.

El árbol principal de configuraciones actualmente incluye:

| Área | Destinos actuales |
| --- | --- |
| Apariencia | Estilo de Escritorio: tema, modo de apariencia, color de acento, fondos de pantalla, íconos y fuentes; Estilo de Aplicación: transparencia del terminal y del lanzador; Barra de tareas; Panel de control; Telemetría de widgets |
| Escritorio y Entrada | Hyprland (Windows: Reglas, Espacios y Modo, animaciones, desenfoque, bordes y sombra), Pantallas, Teclado y Atajos, Ratón y Panel táctil |
| Preferencias | Idioma y Región (idioma, zona horaria, formatos, locales del sistema y distribución del teclado), Fecha y Hora, Aplicaciones predeterminadas, Energía, Sesión |
| Hardware y Conectividad | Resumen de Hardware (resumen, CPU, GPU, memoria, dispositivos), Red y Wi-Fi, Bluetooth, Audio, Almacenamiento |
| Administración del Sistema | Paquetes, Servicios, Inicio, Nombre del Host, Usuarios y Grupos, Cortafuegos, Notificaciones |
| Configuración | Idioma (idioma del usuario), Estilo (iconos y transparencia) |
| Herramientas y Diagnósticos | Proyectos, Fragmentos, Claves SSH, Claves GPG, Servicios de Desarrollo, Doctor del Sistema, Acerca de ARGVUS |

Algunas de estas son páginas completas y otras son subpáginas a las que se accede mediante búsqueda o un resumen de dominio. La lista exacta está determinada por los proveedores de ARGVUS instalados.

Cada página muestra su ruta en el encabezado, por ejemplo `Control Center > Hardware & Connectivity > Audio`. La ruta comienza con la tarjeta de Inicio que lista la página, por lo que la misma página se lee igual desde Inicio, desde la búsqueda y desde la CLI. La raíz y los nombres de las tarjetas están traducidos: la interfaz en portugués muestra `Central de Controle > Hardware e conectividade > Áudio`.

## Para qué sirve cada área

### Apariencia

La apariencia es el área principal de personalización. Su grupo de Escritorio contiene el tema (con su modo Fijo o Espaciado), el modo de apariencia, el color de acento, los fondos de pantalla, el tema de íconos y la página de Fuentes, y su grupo de Estilo de Aplicación contiene la transparencia del terminal y del lanzador. La barra de tareas, el Panel de Control y la Telemetría de Widgets se enumeran directamente bajo Apariencia en la página de inicio y reutilizan la página de transparencia compartida de Apariencia para sus propias superficies. El desenfoque es una configuración global, bajo Hyprland > Desenfoque, y se aplica a todas las superficies ARGVUS que tienen el desenfoque habilitado. Esa página edita su interruptor de Activación y los valores de desenfoque de Hyprland (Tamaño, Pasadas, Brillo, Ruido, Contraste, Vivacidad y Oscuridad de la vivacidad) como un borrador, y nada cambia hasta que se selecciona **Aplicar**. Las animaciones funcionan de la misma manera bajo Hyprland > Animaciones: su interruptor de Activación, las curvas Personalizadas (Bézier, Resorte, Lineal y Suavizado) y las cinco categorías de animación forman un solo borrador que está aplicado junto con **Aplicar**. Bézier y Spring tienen parámetros editables (los puntos de Bézier y la masa, rigidez y amortiguamiento del resorte); Linear y Ease-out son curvas fijas, por lo que su página solo lista sus puntos de control y no se pueden editar. Las opciones de estilo siguen lo que dibuja Hyprland 0.56.2: Apertura y Cierre ofrecen slide, popin y gnomed, la entrada y salida de capa ofrecen slide, popin y fade, y las salidas del espacio de trabajo ofrecen slide, fade, slidefade, slidevert y slidefadevert. Estos controles se coordinan con los componentes de apariencia y sesión en lugar de cambiar solo la ventana del Centro de Control.

### Barra de tareas

La barra de tareas configura la posición del panel, el espacio alrededor de él y sus ventanas, su altura y borde, el grupo de iconos de utilidad y los bloques de fecha/hora, además de su propia transparencia. Se lista bajo **Apariencia** en la página de inicio. En modo Sticky, **Spaces > Bottom** predetermina a `2`; Spaced mantiene su línea base de `18` píxeles.

**Altura** elige 24, 28 (el predeterminado), 32 o 36 píxeles. **Borde** tiene un interruptor de **Activar** y un **Grosor** de 1 a 3 píxeles, siendo 1 el predeterminado. Con el borde desactivado, la ventana de la barra de tareas, el grupo derecho y el reproductor MPRIS no tienen borde. Estos cambios son un borrador hasta que se seleccione **Aplicar**. La altura y el borde aplicados se mantienen cuando cambia el tema; hasta que se apliquen, el tema los decide.

La página de **Iconos** abre **Módulos opcionales**, **Presentación del espacio de trabajo**, **Lanzador** y **Utilidades**. Los módulos opcionales pueden mostrarse u ocultarse individualmente: reproductor de audio, Focus, lanzador, red, perfil de energía, distribución del teclado, memoria, CPU y lecturas de temperatura, fecha, reloj, batería, bandeja del sistema, notificaciones, dispositivos extraíbles, grabación, Bluetooth, entrada/salida de audio y mantener despierto. Los espacios de trabajo permanecen disponibles y pueden mostrar sus nombres (por defecto) o íconos de aplicaciones. El título de la ventana enfocada, Configuración y las acciones de Energía siempre están disponibles. La visibilidad del lanzador se controla en Módulos opcionales; su submenú cambia el ícono personalizado. Los cambios se aplican después de **Aplicar**, lo que reinicia la barra de tareas.

### Panel de control

El Panel de control configura la visibilidad y el orden de sus tarjetas de acción rápida, además de sus propias sesiones y transparencia.

La página se abre y acepta entradas tan pronto como se leen los ajustes de la tarjeta. Las tarjetas que dependen del hardware (Bluetooth y Brillo) se verifican en segundo plano: hasta que esa verificación termine, mantienen la disponibilidad que se vio por última vez en esta ventana, y la lista se actualiza automáticamente cuando llega el resultado.

### Telemetría de Widgets

La telemetría de widgets configura las sesiones y la transparencia del widget de información del sistema.

### Fuentes

Se accede a Fuentes desde **Apariencia > Estilo del escritorio > Fuentes**. Permite seleccionar una familia y tamaño para la barra de tareas, telemetría/información del sistema, Panel de Control, interfaz del sistema ARGVUS, aplicaciones, terminal y navegador. También expone configuraciones de renderizado de fuentes como suavizado de bordes, hinting, modo subpíxel y DPI. La familia primaria predeterminada es IBM Plex Mono; los objetivos individuales se pueden cambiar de manera independiente y restaurarse desde la página de Fuentes.

### Aplicaciones predeterminadas

Las aplicaciones predeterminadas seleccionan programas instalados para funciones como terminal, gestor de archivos, editor de texto, editor de terminal, navegador, visor de imágenes, visor de PDF, reproductor de video, reproductor de audio, herramienta de archivos comprimidos y lanzador. La selección es guardada por ARGVUS y puede actualizar las asociaciones estándar de XDG. Los gestores de archivos TUI utilizan el editor de terminal seleccionado al abrir archivos de texto; los editores de texto gráficos siguen siendo los predeterminados para las asociaciones de archivos de escritorio.

### Fragmentos

Fragmentos es la página para el texto que `argvus snippets` escribe con `SUPER + ALT + N`. Se encuentra en **Herramientas y Diagnósticos → Fragmentos** (busque `snippets` o ejecute `argvus control-center snippets`).
- **Nuevo fragmento** — escribe un **Nombre** y un **Contenido**, luego **Guardar fragmento**. Un nombre que ya exista reemplaza el contenido de ese fragmento.
- **Fragmentos guardados** — listados con su posición (`1  email`) y el contenido en una sola línea. Presiona Enter en el contenido para editarlo. **Tipo** indica el tipo del fragmento, y **Eliminar** pide confirmación.
- **Abrir selector** — abre el mismo selector que `SUPER + ALT + N`.

La escritura comienza 3 segundos después de presionar Enter, porque el Centro de Control tiene el foco en ese momento. Haz clic en el campo que debería recibir el texto dentro de ese tiempo. La página solo edita contenido de una línea; un fragmento de varias líneas se puede crear en una terminal con `argvus snippets add`.

La página lee las entradas de `argvus-config` y las escribe a través de `argvus snippets`, para que el lanzador mantenga la validación. Cuando `argvus snippets` no está instalado, la página lo indica.

### Claves SSH

Las claves SSH son la página para los pares de claves que `argvus keys` administra. Está en **Herramientas y Diagnósticos → Claves SSH** (busca `ssh keys` o ejecuta `argvus control-center ssh-keys`).

La página tiene cuatro partes, de arriba hacia abajo:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```
#### Cómo usarlo

1. Abre **Centro de Control → Herramientas y Diagnósticos → Claves SSH**. Desde la página de inicio, también puedes buscar `ssh keys`.
2. **Encontrar una clave:** presiona `/`, luego escribe para mostrar solo los pares cuyo nombre o comentario coincida. `Enter` termina la búsqueda y ejecuta la fila resaltada, como en otras listas, así que verifica la fila antes de presionarla; `Esc` borra la búsqueda.
3. **Crear un par:** en **Generar**, selecciona **Nombre**, presiona `Enter`, escribe el nombre y presiona `Enter` de nuevo. Selecciona **Tipo** y presiona `Enter` para alternar entre `ed25519` y `rsa`. Selecciona **Generar par de claves** y presiona `Enter`. El nuevo par aparece bajo **Claves** de inmediato, sin frase de contraseña.
4. **Copiar la clave pública:** en el bloque del par, seleccione **Copiar clave pública** y presione `Enter`. El contenido del archivo `.pub` se envía al portapapeles de Wayland, listo para pegar en el `authorized_keys` de un servidor o en un host de Git. La clave privada nunca se copia. Esto requiere `wl-clipboard` (`wl-copy`); sin él, la página indica que la clave no fue copiada.
5. **Cargar un par en el agente:** en el bloque del par, seleccione **Cargar en el agente** y presione `Enter`. Una clave sin contraseña se carga de inmediato, sin ventana. Una clave con contraseña abre una terminal ARGVUS, donde OpenSSH la solicita; cierre la terminal con `Enter` cuando indique que la clave está cargada. La página verifica el agente cada pocos segundos mientras está abierta, por lo que la fila cambia a **Descargar del agente** por sí misma. Para actualizar de inmediato, use **Actualizar** en **Acciones**.
6. **Eliminar una clave del agente:** en el bloque del par, seleccione **Descargar del agente**. Para eliminar todas las claves, use **Descargar todas las claves del agente** en **Acciones**. Los archivos clave permanecen en el disco.
7. **Eliminar un par:** en el bloque del par, seleccione **Eliminar** y presione `Enter`. La confirmación se centra en **Cancelar**, por lo que un `Enter` perdido no hace nada. Presione `y` para confirmar o `n`/`Esc` para cancelar. Una clave que está cargada en el agente no se puede eliminar; descargarlo primero.

Moviéndose: `↑`/`↓` o `j`/`k` se mueven por la lista, `Enter` activa una fila, `Esc` retrocede y `?` muestra ayuda. `Tab` mueve el foco a **Acciones**, donde `←`/`→` elige un botón y `Enter` lo ejecuta. `↑`/`↓` devuelve el foco a la lista.

Las frases de contraseña nunca se escriben en el Centro de control ni se pasan como argumentos de comando. La página lee la lista con `argvus keys list` y escribe solo a través de `argvus keys`. Cuando `argvus-keys` no está instalado, la página lo dice.

#### Solución de problemas

| Síntoma | Causa y solución |
| --- | --- |
| `Could not open a connection to your authentication agent` después de **Cargar en el agente** | El terminal no recibió el socket del agente. Hyprland mantiene el entorno con el que se inició, y el agente puede haberse iniciado después. Actualiza `argvus-launcher`, que vuelve a `$XDG_RUNTIME_DIR/ssh-agent.socket`, y verifica `systemctl --user status argvus-ssh-agent.service`. |
| **Agente** muestra `Agent unavailable` | El lanzador no puede alcanzar al agente. Verifica el servicio como se indica arriba. |
| La página dice `argvus-keys is not installed` | Actualiza el paquete `argvus`: `argvus keys` es un subcomando del mismo. |
| **Eliminar** muestra un error | La clave está cargada en el agente y se mantiene. Descárgala primero, luego elimínala de nuevo. |

### Claves GPG

Las claves GPG son la página para el anillo de claves que `argvus gpg` gestiona. Se encuentra en **Herramientas y Diagnósticos → Claves GPG** (busca `gpg keys` o ejecuta `argvus control-center gpg-keys`).

La página tiene estas secciones, de arriba hacia abajo:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```
`▾` marca una sección expandida y `▸` una colapsada. `Enter` o `Space` la expande o colapsa.

#### Cómo usarlo

1. Abra **Centro de control → Herramientas y Diagnósticos → Claves GPG**. Desde la página de inicio, también puede buscar `gpg keys`.
2. **Pinentry** está fijado a `pinentry-gtk`. La página solo lo muestra. La primera vez que se genera una clave, ARGVUS escribe ese pinentry en `gpg-agent.conf`, entre sus propias líneas de marcador, y mantiene sus otras líneas. Si ya escribió una línea `pinentry-program` fuera de esos marcadores, la generación se detiene con un error para que pueda eliminar esa línea.
3. **Crear una clave:** en **Generar clave ▾**, escriba **Nombre completo** y **Correo electrónico**. Ambos son obligatorios, y **Generar clave** permanece deshabilitado hasta que ambos tengan texto. **Comentario** es opcional. El ID de usuario se convierte en `Name (comment) <e-mail>`.
- **Algoritmo** alterna entre **RSA** y **DSA ElGamal**. Una clave DSA obtiene una subclave de cifrado ElGamal; una clave RSA obtiene una subclave RSA.
- **Fortaleza de la clave** recorre los tamaños del algoritmo: RSA 1024, 2048 y 4096; DSA 1024, 2048 y 3072. gpg no crea claves DSA superiores a 3072 bits, por lo que 4096 no se ofrece para DSA.
- **Fecha de vencimiento** abre su propia página. Escriba la fecha en el formato de su idioma: `DD/MM/YYYY` en portugués, español, francés, italiano, alemán y ruso; `YYYY/MM/DD` en inglés, chino, japonés y coreano. La fecha debe ser posterior a hoy. Active **Nunca expira** para que la clave sea permanente; el campo de fecha se desactiva y no se utiliza.
- **Frase de contraseña** abre su propia página. **Proteger con una frase de contraseña** está activado por defecto; escriba la **Frase de contraseña** y luego **Confirmar frase de contraseña**. Ambas deben coincidir. Desactivar la protección borra ambos campos, y la página solicita confirmación antes de generar una clave sin protección.
4. Seleccione **Generar clave** y presione `Enter`. La clave obtiene una clave de firma y una subclave de cifrado. La frase de contraseña se envía al lanzador en su entrada estándar, nunca como un argumento de comando. Si la generación falla, los demás campos permanecen, pero las frases de contraseña se borran y deben ser escritas nuevamente.
5. **Cancelar** borra todos los campos del formulario y devuelve las fechas y tamaños a sus valores predeterminados.
6. **Importar:** en **Importar ▾**, elija **Importar clave pública…** o **Importar clave privada…**. Se abre el gestor de archivos para que pueda elegir el archivo. Una clave privada solicita confirmación primero, y su frase de contraseña la pedirá gpg cuando se use la clave. Si el gestor de archivos es gráfico, no puede informar el archivo a la página; entonces la página abre un campo donde debe escribir la ruta.
7. **Claves ›** abre el llavero. Cada clave muestra su identidad, huella digital, tipo, caducidad y si su clave privada está en el llavero. Cada clave tiene estas acciones:
   - **Editar clave ›** abre la página de la propia clave (descrita más abajo).
- **Exportar clave pública…** pide una ruta de archivo, propuesta como `~/<fingerprint>.asc`. El archivo se crea con el modo `600`. Un archivo existente nunca se reemplaza.
- **Exportar clave privada…** aparece solo para claves con parte privada. Pide confirmación, luego una ruta de archivo, propuesta como `~/<fingerprint>-private.asc`. El archivo se crea con el modo `600`.
- **Copiar clave pública** y **Copiar huella digital** copian al portapapeles de Wayland con `wl-copy`.
- **Eliminar clave…** pide confirmación, con el enfoque en Cancelar. Elimina la clave y su parte privada.
8. **Buscar una clave:** en la página de **Claves**, la barra de búsqueda en la parte superior filtra las claves. Escribe para reducir la lista; la búsqueda coincide con la identidad (que incluye el comentario) y la huella digital. `Esc` borra la búsqueda.
9. **Editar clave ›** muestra los detalles de la clave, luego estas acciones:
   - **Fecha de caducidad ›** cambia la expiración de la clave y de cada una de sus subclaves. Use **Nunca caduca** para eliminar la expiración. **Aplicar** guarda el cambio.
- **Cambiar frase de paso** solicita la frase de paso antigua y la nueva en la ventana de pinentry.
- **Firmar clave con ›** certifica esta clave con una de sus otras claves que tenga una parte privada. Se solicita la frase de paso del firmante. La página pide confirmación primero.
- **Revocar clave ›** enumera los motivos: sin motivo, clave comprometida, clave reemplazada, clave ya no usada. Cada uno solicita confirmación. Una clave revocada ya no se puede usar.
- **Confianza** establece hasta qué punto confía en la clave: no establecido, marginal, completo o máximo.
- **Publicar en el servidor de claves…** envía la clave pública al servidor de claves configurado en `gpg.conf`, después de la confirmación. **Obtener del servidor de claves** descarga la clave de ese servidor de claves.

Moverse: `↑`/`↓` o `j`/`k` se desplazan por la lista, `Enter` activa una fila, `Esc` regresa y `?` muestra ayuda.

Las frases de contraseña se escriben solo en los campos enmascarados de la página de la frase de contraseña o en la ventana de ingreso de PIN. La página envía la frase de contraseña de generación a `argvus gpg` en la entrada estándar, y el lanzador la entrega a gpg a través de un descriptor de archivo, por lo que nunca aparece en la lista de procesos, en un argumento de comando o en los registros. Los cambios de claves se ejecutan en segundo plano; mientras uno se ejecuta, los otros esperan. La página lee el llavero con `argvus gpg list` y escribe solo a través de `argvus gpg`. Cuando `argvus gpg` no está instalado, la página lo indica.

Aún no en la página: buscar en el servidor de claves una clave por nombre o correo electrónico. Utilice **Recuperar del servidor de claves** con una huella digital conocida.

#### Solución de problemas

| Síntoma | Causa y solución |
| --- | --- |
| La página dice `argvus gpg is not available` | El `argvus-launcher` instalado es anterior al comando GPG. Ejecute `argvus gpg list` en un terminal: `unknown command` significa que el paquete debe reconstruirse e instalarse. Las claves también necesitan `gnupg`. |
| **Generar clave** permanece deshabilitado | Escriba tanto **Nombre completo** como **Correo electrónico**. |
| Una línea de estado dice `argvus-gpg: pinentry-gtk is not installed` | Instale el paquete `pinentry`, que proporciona `pinentry-gtk`. |
| La generación se detiene con un error `pinentry-program` | `gpg-agent.conf` establece un pinentry fuera del bloque ARGVUS. Elimine esa línea; ARGVUS luego escribe el suyo propio. |
| **Pinentry** muestra `Not set` | gpg-agent utiliza su valor predeterminado integrado hasta que se genera la primera clave desde la página. |
| La página dice que el `argvus-launcher` instalado es más antiguo que el Centro de Control | Reinstale `argvus-launcher`: ejecute `make build` y luego `make install` en su repositorio, y vuelva a abrir la página. Las acciones de la clave permanecen desactivadas hasta que las versiones coincidan. |
| El gestor de archivos se abre, pero la página pide una ruta | El gestor de archivos es gráfico y no puede informar el archivo elegido. Escriba la ruta en el campo. |
| **Copiar clave pública** dice que no se pudo copiar | `wl-copy` no está instalado o la sesión de Wayland no está disponible. Instale `wl-clipboard`. |
| **Editar clave ›** dice que la clave ya no está en el llavero | El llavero cambió desde que se abrió la página. Vuelva a **Claves ›** y actualice. |
| Una línea de estado muestra `argvus-gpg: ...` | El lanzador rechazó la acción, y el texto da la razón, como un vencimiento inválido, un archivo que ya existe o un `--yes` perdido. |
| Una acción espera y no sucede nada | La ventana de pinentry está oculta o no se abrió. Compruebe si la sesión expone `WAYLAND_DISPLAY` o `DISPLAY`. |
| La importación es rechazada | El archivo no contiene ninguna clave del tipo elegido. Las claves públicas pasan por **Importar clave pública…**; las claves privadas por **Importar clave privada…**. |

### Idioma y Región

Esta tarjeta lista el idioma (la configuración regional), la zona horaria, la fecha y hora, las localidades del sistema y la distribución del teclado, y cada uno se abre directamente desde la página principal. Las páginas del teclado también contienen la variante y el mapa de teclas de la consola. La búsqueda de la página principal también muestra atajos de teclado y configuraciones de ratón/panel táctil como rutas de configuración directas.

La lista de idiomas detecta los catálogos instalados y muestra los nombres nativos de los diez idiomas. Habilite la localidad del sistema UTF-8 correspondiente antes de seleccionarla.

Tanto **Preferencias > Idioma y región > Idioma** como **Configuración > Idioma** guardan la misma preferencia de usuario: `LANG` en `$XDG_CONFIG_HOME/locale.conf` (por defecto `~/.config/locale.conf`), por ejemplo `LANG=en_US.UTF-8`. Se preservan los comentarios y las variables regionales. No se escribe ningún archivo global ni configuración de idioma ARGVUS paralela. Un usuario no configurado recibe inglés; los archivos heredados `session.language` y `data/language` se ignoran. Cierra sesión y vuelve a iniciarla para aplicar el idioma a toda la sesión y las aplicaciones.

**Locales del sistema** cambia `/etc/locale.gen` y ejecuta `locale-gen`, por lo que aplicarlo requiere autorización de administrador. Un aviso persistente **[WARN]** arriba del pie de página identifica las páginas con configuraciones administrativas. El mismo aviso cubre zona horaria, fecha/hora, nombre de host, mapa de teclas de la consola, edición de cuentas y grupos, firewall, servicios del sistema, gobernador de CPU, perfiles y políticas de energía, configuración de arranque, operaciones de paquetes, instantáneas y cambios de red protegidos. Las preferencias personales, servicios de usuario y páginas de solo lectura no reciben este aviso. El aviso es informativo; la autorización solo se solicita cuando una acción lo necesita.

### Hardware y pantallas

Cuando se instalan las capacidades correspondientes, el Centro de Control puede mostrar información de hardware como CPU, GPU, memoria, energía y dispositivos. La página de entrada configura el comportamiento del ratón y del touchpad; el área de pantalla maneja la resolución del monitor, la tasa de refresco, la escala, la posición, la orientación, la pantalla principal, VRR y HDR cuando son compatibles con la sesión en ejecución. Los cambios de pantalla se aplican de inmediato. Después de un cambio que puede dejar la pantalla inutilizable (resolución o tasa de refresco, posición, duplicación, deshabilitar un monitor, profundidad de color) o después de **Aplicar**, el Centro de Control pregunta si se desea mantener, con el enfoque en **Revertir**: presione `y` para mantenerlo; `n`, `Esc` o `Enter` para revertirlo, y sin una respuesta dentro de los 15 segundos, la configuración anterior vuelve automáticamente. Al eliminar la configuración guardada de un monitor desconectado y al borrar un perfil se solicita confirmación; los perfiles abren su propia página con **Aplicar perfil**, **Renombrar** y **Eliminar**. perfil**. El diseño se guarda a través de `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` y `monitor-remove`, por lo que la sección canónica `/displays` de `config.json` es la única fuente del diseño del monitor. Solo el catálogo de perfiles permanece en `display/config.toml`.

### Conectividad y audio

El área de red expone el estado y las páginas de conexión del proveedor de red instalado, incluyendo Wi-Fi, Ethernet, VPN, DNS y rutas de proxy cuando están disponibles. El firewall se encuentra listado bajo **Administración del Sistema**. Bluetooth depende del soporte de Bluetooth. El área de audio se proporciona cuando su capacidad opcional está habilitada e integra con el servicio de audio instalado.

La página de inicio de **Red y Wi-Fi** y su página de **Estado** se rellena a medida que responde cada consulta de NetworkManager, en lugar de después de la más lenta: primero la lista de dispositivos, luego la radio, las direcciones y el DNS de las interfaces conectadas, las conexiones guardadas y las redes Wi-Fi que NetworkManager ya conoce (un nuevo escaneo solo se ejecuta desde la página de Wi-Fi con `r`). Hasta que llegue la lista de dispositivos, las filas muestran *Cargando...*, y el interruptor de Wi-Fi solo se ofrece cuando la computadora tiene una tarjeta Wi-Fi. Iniciar una acción (el interruptor de Wi-Fi, la conexión, DNS) mientras una página se está actualizando reemplaza la actualización, y la página se actualiza nuevamente cuando finaliza la acción.

#### Conexiones, inspirado por nmtui

**Red y Wi-Fi > Conexiones** enumera todos los perfiles de conexión de NetworkManager, agrupados por tipo, con el dispositivo en el que están activos (`★ Active · eno1`), si es un puerto de un bridge, bond o team, y si es de solo lectura. `Enter` abre el perfil en el editor, `a` lo conecta o desconecta, `/` filtra por nombre y `r` actualiza. **Activar una conexión** es la misma lista donde `Enter` activa o desactiva un perfil, como en `nmtui`. Las páginas de detalles de una interfaz, una red Wi-Fi guardada y una VPN tienen una fila de **Editar conexión** que abre su perfil.

**Añadir conexión** crea perfiles de Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Puente, Bond, Team, VLAN, túnel IP, Loopback y WireGuard, y un perfil de VPN por cada complemento de NetworkManager VPN instalado (OpenVPN tiene su propio formulario; otros complementos editan sus configuraciones como entradas `key=value`). También importa un `.conf` de WireGuard o un archivo de complemento como un `.ovpn` (`nmcli connection import`). Team necesita el paquete `libteam`, DSL necesita `ppp` y una VPN necesita su complemento (por ejemplo `networkmanager-openvpn`); sin ellos, la fila está deshabilitada y dice qué instalar. WEP, InfiniBand, MACsec y Open vSwitch no se ofrecen.

El editor es una lista de secciones plegables: **General** (nombre, interfaz, conectar automáticamente, prioridad, disponible para todos los usuarios, medido), la configuración propia del tipo, **IPv4** y **IPv6** (método, direcciones, puerta de enlace, servidores DNS, dominios de búsqueda, rutas, ignorar DNS o rutas automáticas, nunca usar como ruta predeterminada, métrica de ruta, extensiones de privacidad, generación de direcciones). Las opciones y números cambian con `←/→`; `Enter` escribe un valor; las listas (direcciones, DNS, rutas, IPs permitidas) abren su propia página, donde `Enter` edita una entrada, borrarla elimina la entrada y **Agregar entrada** añade una. Los campos que no se aplican al método elegido se muestran deshabilitados. Un bridge, bond o team elige sus **Puertos** Ethernet en una subpágina; WireGuard edita sus **Peers** en una subpágina y puede **Generar par de claves** (con `wg`, desde `wireguard-tools`, una dependencia del paquete `argvus`); la clave pública de la clave privada se muestra debajo de esta.

Nada llega a NetworkManager hasta **Guardar**, lo cual permanece deshabilitado mientras nada haya cambiado o mientras un campo sea inválido, y luego indica qué campo y por qué. Solo se escriben los ajustes que han cambiado. Guardar una conexión que está activa pregunta primero y luego la reconecta, para que los nuevos ajustes se apliquen de inmediato (esto puede interrumpir brevemente la red). `Esc` con cambios no guardados pregunta antes de descartarlos. **Eliminar conexión**, en la zona de peligro, pide confirmación con el enfoque en Cancelar; eliminar un puente, enlace o equipo también elimina sus puertos.

Las contraseñas, claves privadas y claves precompartidas nunca se leen de nuevo: sus filas dicen *Sin cambios* hasta que se escribe un nuevo valor, y el nuevo valor se muestra enmascarado. Se escriben a través de la entrada estándar de `nmcli connection edit`, nunca como argumentos de comando. Los cambios se ejecutan como el usuario de la sesión: los miembros locales de `wheel` están permitidos por la regla polkit de NetworkManager, y a otros usuarios se les solicita la contraseña de administrador por el agente polkit de la sesión.

### Energía y sesión

Power proporciona las páginas de controles de energía y políticas del sistema disponibles para la instalación, incluido el interruptor Mantener activo. Cuando está habilitado, Mantener activo evita que la política de inactividad de ARGVUS inicie automáticamente los temporizadores de bloqueo de pantalla y de ahorro de energía de la pantalla. Sesión expone el estado de la sesión de ARGVUS, los componentes, el inicio automático, el diagnóstico y los registros. En **Power**, la barra de **Acciones** debajo de la lista contiene **Suspender**, **Hibernar** y **Actualizar** (`r` también actualiza): `Tab` o `←/→` se mueven hacia ella, `Enter` ejecuta el botón enfocado, `↑`/`↓` devuelven el foco a la lista, y `Esc` regresa. Suspender e Hibernar piden confirmación primero. Estas páginas pueden requerir permisos del sistema y deben distinguirse de las acciones rápidas en el Panel de control.

### Herramientas del sistema

La sección del sistema puede exponer información de arranque, paquetes, servicios, almacenamiento, diagnósticos, administración de usuarios/grupos e información del sistema. Estas son herramientas administrativas o de diagnóstico; una página puede ser de solo lectura o requerir autorización según la operación.

**Servicios** tiene cuatro páginas: unidades del sistema, unidades de usuario, unidades fallidas y el diario (Registros). Cada lista de unidades muestra una línea `Filter:`, que `f` cicla, y un cuadro con borde `Search:`, abierto con `/`. Cada unidad muestra su estado Activo y Habilitado como puntos llenos o vacíos. En Registros, `b` cambia entre el arranque actual y el anterior, `u` cicla la unidad y `p` cicla la prioridad; la línea `Filter:` muestra los valores en uso. `Enter` abre una unidad o una entrada de registro.

**Servicios de desarrollo** informa tres fuentes dirigidas al desarrollo en una sola página de solo lectura: los servicios systemd del usuario actual, los puertos TCP que están escuchando actualmente (`ss`) y los contenedores en ejecución de cualquier runtime de contenedores instalado (Podman, Docker). Solo aparece en la cuadrícula de Inicio cuando al menos una de estas está disponible en el sistema: una máquina sin sesión de usuario systemd, sin Podman y sin Docker no muestra la entrada. `r` actualiza las tres fuentes de manera independiente, por lo que una que sea lenta o falle (por ejemplo, un runtime de contenedores que dejó de responder) no bloquea que las demás muestren sus datos.

**Los paquetes** se agrupan por fuente. **Repositorios oficiales** y, cuando se instala un ayudante de AUR (`paru` o `yay`), **AUR (Arch User Repository)**, cada uno abre **Instalar**, **Eliminar** y **Actualizar**; **Caché**, **Historial**, **Revertir** y **Espejos** siguen. Las tres tareas comparten una pantalla: el cuadro de **Buscar** fijo en la parte superior toma las letras que escribes, y los paquetes se listan debajo en el bloque de **Resultados** (**Actualizaciones** en Actualizar), donde una lista vacía indica la razón (por ejemplo, **Sin resultados**, **No hay elementos coincidentes** o **No hay actualizaciones pendientes**). **Instalar** busca en los repositorios (`pacman -Ss`) o en el AUR cuando presionas `Enter` en la barra de búsqueda superior; **Eliminar** lista los paquetes instalados (la lista de AUR muestra los paquetes construidos fuera de los repositorios, `pacman -Qm`, y mientras se muestra el menú de AUR, se dejan los de la lista Oficial); **Actualizar** lista las actualizaciones pendientes (`pacman -Qu`, o `paru -Qua`/`yay -Qua` para el AUR). En un paquete, `Enter` ejecuta la tarea de la pantalla y `→` abre la página del paquete, con **Instalar** (o **Reinstalar**) y, en su **Zona de peligro**, **Eliminar**; `←` o `Esc` regresan. **Repositorios oficiales → Eliminar** comienza con **Huérfanos**, donde `Space` marca paquetes (`[x]`), `Enter` abre uno y **Eliminar marcado** los elimina juntos. Desde la lista de resultados de Actualización, `Tab` se mueve a la barra **Acciones** debajo de la lista (**Actualizar todo** y **Actualizar base de datos**, o **Actualizar todo (AUR)** en el AUR), `←`/`→` se mueven entre sus botones, `Enter` ejecuta el enfocado y `↑`/`↓` (o escribiendo) devuelve el foco a la lista; **Actualizar todo** aparece atenuado y se omite mientras no aparece ninguna actualización. La ruta de navegación y la búsqueda global siguen la misma ruta, por ejemplo **Paquetes > Repositorios oficiales > Eliminar > Huérfanos**. Las limpiezas de caché se encuentran en la zona de peligro de la página Caché y **Espejos → Configurar espejos** establece las opciones del reflector antes. **Generar vista previa**. Cada operación de paquete muestra su plan, o para el AUR la advertencia PKGBUILD, y pide confirmación (`y` confirma, `n` o `Esc` cancela) antes de ejecutarse, y su salida se muestra mientras se ejecuta. Las operaciones del AUR se ejecutan con el ayudante como tu usuario (`-S <package>` para instalar o actualizar uno, `-Sua` para actualizar todos); las eliminaciones del AUR usan el mismo `pacman -Rns` que los paquetes oficiales. Cuando otros paquetes instalados dependen del paquete que eliminas (oficial o AUR), la confirmación los nombra y ofrece dos opciones: **Eliminar solo** el paquete seleccionado (`pacman -Rdd`, que omite las comprobaciones de dependencias, por lo que los paquetes que lo necesiten pueden dejar de funcionar) o **Eliminar con dependientes** (`pacman -Rcns`, con el número de paquetes que elimina). El enfoque comienza en **Cancelar**, que mantiene todos los paquetes; `Enter` ejecuta la fila enfocada y no hay un atajo `y` en esta opción.

La barra superior **Buscar** posee la consulta del paquete. `/` lo enfoca sin borrar el texto; escribir también mueve el foco allí. Edite con las teclas de flecha, `Home`, `End`, `Backspace`, `Delete` o pegue. En **Instalar**, `Enter` envía una consulta de repositorio/AUR incluso cuando existen resultados anteriores y luego mueve el foco a los resultados; las búsquedas oficiales requieren una consulta que no esté vacía y las búsquedas AUR requieren al menos dos caracteres. La edición de una consulta de instalación cancela su solicitud pendiente y borra sus resultados antiguos; una nueva solicitud espera `Enter`. En **Eliminar**, **Actualizar** y **Huérfanos**, la edición filtra la lista cargada inmediatamente sin otra consulta al repositorio. `↓` pasa de la Búsqueda a los resultados, `↑` en el primer resultado regresa a la Búsqueda y `Esc` en la Búsqueda regresa a la lista conservando la consulta. `Enter` en un resultado realiza la acción de ese paquete. `Tab` sigue accesible a la barra de acciones Actualizar y `/` regresa desde allí a la Búsqueda.

Todas las operaciones en **Paquetes** abren una terminal interactiva integrada: paquetes oficiales, AUR, limpieza de caché, espejos y cambios de repositorio/clave. Responda las indicaciones del programa escribiendo y presionando `Enter`; Pacman se ejecuta sin `--noconfirm` en esta terminal. `Esc` oculta la vista y mantiene el proceso en ejecución. `F8` vuelve a abrir la misma sesión desde cualquier página de Paquetes; **Mostrar terminal** también está disponible en sus menús. El pie de página muestra si la transacción se está ejecutando, completada o fallida. Después de un comando exitoso, `Done!` aparece dentro del terminal después de su salida final, incluso cuando el terminal estaba oculto. `Shift+PageUp` / `Shift+PageDown` desplaza el historial, `Shift+End` vuelve a la salida en vivo y `Ctrl+C` envía una interrupción al proceso secundario. Mientras el terminal está enfocado, `q` y `?` son accesos directos de entrada en lugar de globales. La salida, los colores ANSI y el cursor se conservan, y el cambio de tamaño actualiza el terminal secundario. la historia retiene hasta 10,000 líneas hasta la siguiente operación o hasta que la aplicación se cierre. Solo se ejecuta una transacción a la vez; ocultar su terminal no permite que la aplicación se cierre mientras se ejecuta. No se abre ninguna shell libre.

La vista previa de la actualización lee las bases de datos de sincronización existentes (`pacman -Su --print`) sin modificarlas ni requerir root. Tras la confirmación, la actualización completa ejecuta `pacman -Syyu` a través del asistente privilegiado con `pkexec`; la sincronización puede revelar actualizaciones adicionales. La instalación de paquetes, eliminación, actualización de bases de datos, limpieza de caché, degradación, cambios de repositorio/clave y escrituras en espejos también utilizan ese asistente privilegiado. Los asistentes de AUR continúan construyéndose como tu usuario.

**Repositorios oficiales → Repositorios** agrega y administra repositorios no oficiales de pacman. **Agregar** es un borrador: elija la **Fuente** de la configuración (**URL** de un archivo `.conf`, **Texto** escrito o pegado en un editor de varias líneas, o **Formulario** con **Nombre**, **Servidor** y **SigLevel**, escrito en el diseño de Arch Wiki) y, en **Clave de firma**, una **Clave** de una **URL** o **Texto** (o **Ninguno**). Las opciones se mueven con `←`/`→` o `Enter`; Los campos de texto abren un editor donde `Enter` guarda una sola línea, `Ctrl+S` guarda varias líneas, el pegado funciona y `Esc` cancela. Las descargas solo aceptan direcciones `https://`. **Revise y agregue** descargas y verifica todo como su usuario y luego solicita confirmación, enumerando los repositorios, servidores y SigLevel y la identificación de usuario y la huella digital de la clave en grupos de cuatro, para comparar con el que publica el repositorio. Al confirmar se importa la clave y la firma localmente (`pacman-key --add`, `--lsign-key`), escribe `/etc/pacman.d/<name>.conf` (llamado así por su primera sección) y añade `Include = /etc/pacman.d/<name>.conf` a `/etc/pacman.conf` (en su lugar se habilita una línea comentada existente para el mismo archivo). Posteriormente, el Centro de Control ofrece `pacman -Syyu`, por lo que las nuevas bases de datos se sincronizan sin una actualización parcial. Dejar **Agregar** con un borrador solicita confirmación antes de descartarlo. **Lista** muestra cada repositorio no oficial, escrito en `pacman.conf` o incluido desde `/etc/pacman.d/`, habilitado o deshabilitado; las secciones oficiales nunca se listan. Una página de repositorio muestra su archivo, SigLevel, servidores y clave registrada, un interruptor de **Habilitado** y, en la **Zona de peligro**, **Eliminar**. Deshabilitar comenta la línea `Include`, o antepone cada línea de una sección en línea con `#argvus-off `; eliminar borra la línea o la sección y el archivo incluido. Los repositorios del mismo archivo incluido se habilitan, deshabilitan y eliminan juntos. Cuando el Centro de Control importó la clave, **Eliminar** también ofrece eliminarla del llavero de pacman (`pacman-key --delete`). Cada cambio en `/etc/pacman.conf` mantiene una copia de seguridad junto a él (`pacman.conf.argvus-backup-…`), y los paquetes instalados nunca se eliminan.

### Usuarios y grupos

Cuando el proveedor de cuentas esté disponible, abra **Centro de Control → Administración del Sistema → Usuarios y Grupos → Usuarios** o busque **usuarios**. La interfaz puede listar cuentas normales y de sistema, crear un usuario, editar metadatos de la cuenta, gestionar grupos suplementarios y primarios, cambiar o bloquear una contraseña, desbloquearla, requerir un cambio de contraseña en el siguiente inicio de sesión, establecer o eliminar un avatar, y habilitar el inicio de sesión automático.

La creación de usuario solicita un nombre de usuario, nombre completo, shell, grupos suplementarios y una confirmación de contraseña opcional. Una contraseña en blanco crea la cuenta con su contraseña bloqueada; no crea silenciosamente una contraseña utilizable. Una página de usuario agrupa los campos de la cuenta (nombre completo, shell, grupos primario y suplementarios, seguido de **Guardar cambios**), el interruptor de **Inicio de sesión automático**, las acciones de **Contraseña** (cambiar, bloquear, desbloquear, requerir un cambio en el próximo inicio de sesión) y las acciones de **Avatar**. El nombre de usuario, UID / GID y el directorio personal se muestran solo como información. La **Zona de peligro** al final contiene las acciones destructivas separadas para eliminar la cuenta manteniendo su directorio personal o para eliminar también el directorio personal. Lea el texto de confirmación cuidadosamente antes de elegir la última opción. Las operaciones de cuenta requieren privilegios y pueden abrir el aviso de autorización del sistema. Los avatares pueden ser utilizados por el gestor de inicio a través de la cara estándar de la cuenta. integración de imágenes.

**Inicio de sesión automático** inicia la sesión de la cuenta seleccionada en el próximo arranque sin mostrar la pantalla de inicio de sesión, la misma convención que usan GNOME, LightDM y SDDM para esta función: no se almacena ninguna contraseña en ningún lugar. Habilitarlo para una cuenta deshabilita cualquier cuenta previamente configurada (solo una cuenta puede iniciar sesión automáticamente a la vez). Se aplica una vez, en el próximo arranque; después de un cierre de sesión manual, la pantalla de inicio de sesión normal regresa por el resto de esa sesión. El interruptor se aplica inmediatamente, con un mensaje de confirmación, y solo está disponible para administradores.

Los grupos tienen su propia lista y páginas de edición, incluyendo la gestión de miembros y la eliminación de grupos. El Centro de Control no reemplaza la política de cuentas del sistema: las cuentas del sistema protegidas y las operaciones que requieren autorización siguen sujetas a las restricciones del sistema operativo.

### Configuraciones del Centro de Control

**Configuración** es la última entrada del inicio. Su página de **Idioma** establece el idioma del usuario a través de locale.conf, y su página de **Estilo** controla los iconos decorativos y la transparencia del Centro de Control. El estilo no tiene desenfoque: el desenfoque es una configuración global de Hyprland, en Hyprland > Desenfoque. La página de Estilo cambia la presentación de la aplicación de configuración; no es un interruptor que desactive los servicios de ARGVUS ni elimine proveedores de configuración. Guardar los iconos puede requerir autorización, mientras que el cambio visual se aplica a la aplicación actual. Esc en Estilo vuelve a la página de Configuración, y Esc allí vuelve al inicio.

### Acerca de ARGVUS

**Acerca de ARGVUS** se encuentra en **Herramientas y Diagnósticos → Acerca de ARGVUS** (busque `about`, `hardware` o `firmware`, o ejecute `argvus control-center about`). Tiene cinco pestañas, que se cambian con `Tab`/`Shift+Tab`, `←`/`→` o `h`/`l`: **Sistema y Hardware**, **Acerca de**, **Donar**, **Créditos** y **Derechos de Autor**. El encabezado muestra la ruta completa, por ejemplo `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Sistema y Hardware** es de solo lectura y agrupa los datos de esta máquina en tres secciones, con los valores alineados en una columna:

| Sección | Campos |
| --- | --- |
| Arranque | Firmware (proveedor, versión y fecha), Tipo (UEFI o Legacy (BIOS)), Arranque seguro (solo UEFI), Virtualización, Gestor de arranque |
| Sistema | ARGVUS, Nombre del host, Sistema operativo, Distribuidor, Kernel, Arquitectura, Gestor de ventanas (Hyprland), Compositor (Wayland), Controlador de GPU, GTK, Tema (con Oscuro o Claro), Pantallas (una línea por monitor), Audio (PipeWire), Shell (Quickshell), Inicio de sesión (greetd), Init (systemd), Paquetes, Tiempo de actividad, Configuración regional |
| Hardware | Batería (una línea por batería), CPU (con su frecuencia máxima), Hilos, Núcleos, GPU, Memoria (usada / total), Intercambio, Almacenamiento, Modelo (solo cuando difiere de la placa base), Placa base |

Cada valor se lee desde el sistema en ejecución, sin privilegios, y la página funciona igual en cualquier computadora. Un valor que no se puede leer se muestra como `N/A`, y se omite una línea que no aplica:

- **Firmware**, **Modelo** y **Placa base** provienen de DMI (`/sys/class/dmi/id`); se ignoran valores de marcador de posición como `To be filled by O.E.M.`. **Tipo** y **Arranque seguro** provienen de `/sys/firmware/efi`, y **Virtualización** de `systemd-detect-virt`.
- **Cargador de arranque** se lee de la variable EFI `LoaderInfo`, que publican systemd-boot, Limine y otros cargadores de arranque que siguen la Interfaz del Cargador de Arranque. GRUB no lo publica, por lo que es reconocido por `/boot/grub` (o `/efi/grub`) y se muestra con la versión del paquete `grub`.
- Las versiones de ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd y GRUB provienen de una consulta `pacman -Q` y se muestran sin la época y la versión del paquete (`1:25.2.3-1` se convierte en `25.2.3`). La versión de Hyprland proviene de `hyprctl version`, o del paquete cuando ninguna sesión responde.
- **Controlador de GPU** nombra el controlador vinculado a cada tarjeta DRM en `/sys/class/drm`. Un controlador propietario muestra su versión de módulo (por ejemplo `nvidia 580.82.07`); un controlador en el kernel muestra la versión de Mesa (por ejemplo `amdgpu (Mesa 25.2.3)`).
- **Pantallas** lista los monitores activos de `hyprctl monitors -j`: conector, resolución, frecuencia de actualización y escala (la escala se omite si es 1).
- **Paquetes** cuenta todos los paquetes instalados, los instalados explícitamente y los extranjeros (AUR o construidos localmente), de `pacman -Qq`, `-Qqe` y `-Qqm`.
- **Tiempo de actividad** proviene de `/proc/uptime` y es el valor cuando se abrió la página. **Configuración regional** proviene de `LANG` en `/etc/locale.conf`.
- **Batería** lista las baterías del ordenador desde `/sys/class/power_supply`, seleccionadas por tipo, por lo que los periféricos como los ratones inalámbricos no se cuentan. Cada línea muestra la carga, el estado, la salud (capacidad total en comparación con la capacidad de diseño) y los ciclos de carga cuando el firmware los reporta. Una máquina sin batería muestra **No detectada**.
- **CPU** agrega la frecuencia máxima más alta entre todos los núcleos (cpufreq), que es la máxima nominal en procesadores híbridos; se omite donde cpufreq no está disponible, como en la mayoría de máquinas virtuales. **Hilos** y **Núcleos** provienen de `/proc/cpuinfo`.
- **Memoria** y **Swap** provienen de `/proc/meminfo` (zram incluida); un sistema sin swap muestra **Deshabilitado**.
- **Almacenamiento** describe el disco bajo `/`: su modelo, su tipo (NVMe, SSD, HDD o eMMC/SD; los discos virtuales no muestran tipo), el sistema de archivos y el espacio usado y total. El disco se encuentra a través de particiones, LUKS y LVM hasta el dispositivo físico.

Los valores se leen en segundo plano la primera vez que se dibuja la página, por lo que ni el inicio del Centro de Control ni la apertura de la página esperan por ellos; mientras tanto, la pestaña muestra **Cargando información del sistema...**. Presione `c` en esta pestaña para copiar todo el informe como texto plano al portapapeles (con `wl-copy`), por ejemplo, para pegarlo en un informe de errores.

## Búsqueda y capacidades opcionales

La pantalla de inicio tiene un registro de rutas que se puede buscar. Los términos de búsqueda incluyen conceptos orientados al usuario, como apariencia, temas, fondos de pantalla, fuentes, atajos de teclado, ratón, panel táctil, pantallas, red y sesión. Los proveedores opcionales se incluyen solo cuando su función está compilada y se detecta su capacidad en tiempo de ejecución, por lo que el menú visible es intencionalmente consciente de las capacidades.

Mientras la búsqueda de Inicio está activa, se ingresan claves imprimibles en la consulta, incluyendo `j` y `k`; use las teclas de flecha `↑` y `↓` para moverse a través de las rutas coincidentes. Presione `Enter` para abrir la ruta seleccionada, `Backspace` para editar la consulta y `Esc` para borrar la consulta o salir del modo de búsqueda.

## Flujo de trabajo de Apariencia

Abra **Apariencia** para ver los controles visuales integrados. Sus grupos son:

- **Escritorio** — **Tema** (elige una familia incluida y su modo Pegajoso o Espaciado, o exporta, importa, aplica y elimina tus propios temas, consulta [Importar y exportar temas](/docs/components/argvus-themes/import-and-export/)), **Modo de Apariencia** (Pegajoso o Espaciado, independiente del tema), **Color de Acento** (un acento incluido o un color RGB de seis dígitos válido), **Fondo de Pantalla** (un fondo de pantalla incluido o un archivo personalizado) **Iconos** (un tema de iconos instalado: busca en la lista con `/`, elige uno con Espacio o Enter, luego selecciona **Aplicar**; nada cambia antes de Aplicar) y **Fuentes** (los objetivos de fuente y las configuraciones de renderizado de fuentes, abiertos en la página de Fuentes).
- **Estilo de la Aplicación** — la transparencia de la **Terminal** y del **Lanzador**.

La tarjeta Apariencia contiene las superficies que usan estos ajustes: **Telemetría de Widget** (habilitar la superficie de telemetría, seleccionar sus bloques disponibles y reordenarlos con `Shift+Up`/`Shift+Down` en un bloque enfocado) y **Panel de Control** (habilitar, deshabilitar y reordenar tarjetas del panel con `Shift+Up`/`Shift+Down` en una tarjeta enfocada, aplicado inmediatamente a través del mismo primitivo que usa el arrastrar y soltar del panel real).

Estas acciones actualizan el estado lógico ARGVUS y aplican la configuración de ejecución afectada. Ver [Apariencia](/docs/user-guide/appearance/), [Temas](/docs/user-guide/appearance/) y [Ventanas y disposición](/docs/argvus-hyprland/windows-and-layout/).

## Uso del teclado

La aplicación de configuración admite la navegación con el teclado. Cada página es una lista única: use las teclas de flecha o `j`/`k` para moverse, `Enter` para abrir o ejecutar la fila seleccionada, `Space` para alternar, `Esc` para regresar, `/` para buscar listas y `?` para obtener ayuda; el pie de página muestra las claves que se aplican a la fila seleccionada. Las acciones son filas de la lista, no botones, y `Tab` no se mueve hacia ellas, excepto en las páginas con una barra **Acciones** debajo de la lista (aplicaciones predeterminadas, zona horaria, distribución del teclado, configuraciones regionales del sistema y claves SSH, y crear/editar acceso directo). Se omiten las filas que solo muestran información. Mientras un campo de texto, una búsqueda o un filtro de lista está activo, `q` y `?` se escriben como texto en lugar de salir o abrir la ayuda. En **Aplicaciones predeterminadas**, **Restaurar valores predeterminados** es el botón en la barra **Acciones** debajo de la lista: `Tab` o `←/→` lo enfoca y `Enter` solicita el reinicio. **Restablecer valores predeterminados** en las otras páginas y **Restaurar todos los accesos directos** son filas en un **Peligro zona** al final de sus páginas; `r` solicita el mismo reinicio y, en la lista de accesos directos, `r` restaura el acceso directo seleccionado después de una confirmación. En **Idioma y región → Distribución del teclado**, `/` enfoca el cuadro **Buscar** en la parte superior de la página y `Esc` lo borra; El `●` marca el diseño en uso. Sigue la barra de tareas y el método abreviado de teclado que alterna los diseños, y la página se actualiza en aproximadamente un segundo sin escribir nada en la configuración. `Enter` establece el diseño predeterminado, el primero con el que comienza Hyprland y el etiquetado como `Default` en su detalle; `Space` agrega o elimina un diseño (`[x]`). Estas claves solo cambian un borrador, y el botón **Aplicar** en la barra **Acciones** debajo de la lista escribe los diseños y el valor predeterminado en Hyprland en un solo paso. De la lista, `Tab` o `←/→` mueve el foco a esa barra mientras **Aplicar** está habilitado (el pie de página muestra `←/→ Actions` en ese momento); `←/→` recorre sus botones y `↑/↓` regresa al lista. Al salir de la página con cambios no aplicados, primero se solicita confirmación. En **Idioma y región → Zona horaria** el diseño es el mismo: `/` enfoca el cuadro **Buscar** en la parte superior, `Enter` elige una zona en un borrador (marcada con `●`) y el botón **Aplicar** en la barra **Acciones** debajo de la lista la guarda en el `environment.d/20-timezone.conf` del usuario; `Tab`, `←/→` y `↑/↓` funcionan como en la distribución del teclado. Salir con una zona que no fue aplicada pide confirmación primero. En **Configuraciones regionales del sistema**, el diseño es el mismo que el de Zona horaria: `/` enfoca el cuadro **Buscar** en la parte superior, `Enter` o `Space` marca o desmarca una configuración regional (`[x]`) en un borrador, y el botón **Aplicar** en la barra **Acciones** debajo de la lista escribe la selección en `/etc/locale.gen` y ejecuta `locale-gen`. Debido a que eso regenera las configuraciones regionales del sistema, **Aplicar** solicita confirmación primero; `Tab`, `←/→` y `↑/↓` funcionan como los otros dos. En **Escritorio y entrada → Atajos de teclado → Todos Atajos** la caja de búsqueda también se encuentra en la parte superior, con `/` enfocándola, pero la página no tiene barra de **Acciones**: su lista mantiene las secciones de atajos y la **Zona de peligro**, y el pie de página lista `r Restore default` solo en la lista en sí, no en **Editar atajo**.

Las páginas que recopilan cambios antes de guardarlos (un usuario, un grupo, creación de cuenta, cambio de contraseña y configuración del cortafuegos) terminan ese bloque con una fila de **Guardar** o **Aplicar** que permanece atenuada hasta que algo cambie. Si al regresar se descartarían los cambios no guardados, el Centro de Control pregunta primero. Cada confirmación muestra **Confirmar** y **Cancelar** como dos filas con el enfoque en **Cancelar**: `Enter` ejecuta la fila enfocada, `y` confirma y `n` o `Esc` cancelan.

En **Apariencia**, cada página es una lista única: el pie de página muestra solo las teclas que corresponden a la fila seleccionada, y `Space` también abre o selecciona la fila. Las páginas que recopilan cambios antes de aplicarlos (Barra de tareas, Telemetría de widgets, Panel de control y los deslizadores de efecto) terminan con una fila de **Aplicar**, que se mantiene atenuada hasta que algo cambia. Seleccionar **Aplicar** muestra la pantalla de carga solo cuando el cambio recrea una superficie: un cambio de altura de la barra de tareas, grupo de utilidades, icono o formato de fecha y hora reinicia la barra de tareas; activar o desactivar la Telemetría de widgets o cambiar sus bloques o su orden reinicia el widget; activar o desactivar el Panel de control, o cambiar sus tarjetas mientras está activo, reinicia el panel. La pantalla se mantiene hasta que se aplica el último cambio de la página, por lo que ningún cambio aparece antes de eso, y la superficie se reinicia una sola vez. Los bordes, transparencia, desenfoque, animaciones, espacios, fuentes, fondo de escritorio y demás configuraciones se aplican en el lugar, sin el pantalla de carga. En las filas de porcentaje, `←`/`→` (o `+`/`-`) cambian el valor en pasos de 5 y `Enter` te permite escribirlo; usa `Esc` para volver. Si volver descartaría cambios que no se aplicaron, el Centro de Control pregunta primero. Eliminar un tema personalizado (`d`) o reemplazar uno importado pide confirmación; el enfoque comienza en **Cancelar**, `y` confirma y `n` o `Esc` cancelan.

## Persistencia y reinicio

Las páginas de configuración aplican cambios a través de su proveedor correspondiente y guardan el estado de usuario compatible. Los cambios de apariencia se almacenan en la configuración canónica, y `argvus-config` es el único componente que escribe los archivos derivados de usuario bajo `~/.config/argvus/data/generated/`; esos archivos generados no son el lugar para hacer una edición permanente. Las páginas de entrada y asignación de teclas tienen su propio estado persistente y acciones de reinicio. Otros cambios del sistema pueden requerir permisos o una recarga del servicio.

No existe un restablecimiento global para cada configuración de ARGVUS. Restaura un cambio desde la página que lo posee, o utiliza el procedimiento de recuperación documentado de la función relevante.

El soporte de restablecimiento es deliberadamente por dominio:

- **Apariencia** proporciona un restablecimiento para el color de resaltado al valor predeterminado del tema activo. No proporciona un restablecimiento único de apariencia global; restaura otros valores de apariencia desde sus propios controles.
- **Fuentes** puede restaurar todas las configuraciones de fuente, un objetivo o una configuración individual.
- **Aplicaciones predeterminadas** puede restaurar todos los valores predeterminados, una categoría o un selector individual.
- **Atajos de teclado** puede restaurar una vinculación o todas las vinculaciones y luego recargar las vinculaciones de sesión generadas.
- **Reglas de ventana** se encuentran en **Escritorio y Entrada → Hyprland → Ventanas → Reglas** (busca `window rules` o ejecuta `argvus control-center window-rules`). Cada regla tiene una fila de **Espacio de trabajo** (`Enter` o `←/→` avanza del 1 al 10) y una fila de **Clases de ventana** que abre un campo de texto con expresiones regulares separadas por comas. **Agregar regla** crea `rule-N` con espacio de trabajo 1 y sin clases. El **Eliminar regla** de cada regla se encuentra en la **Zona de peligro** y solicita confirmación. Cada cambio se escribe en `hyprland.window_rules` en `argvus config` y recarga la configuración inmediatamente; los patrones de clase inválidos son rechazados y mostrados como error.
- **Proyectos** se encuentra en **Herramientas y Diagnósticos → Proyectos** (busque `projects` o ejecute `argvus control-center projects`). **Agregar carpeta de proyecto** y **Agregar carpeta raíz** se ubican en la parte superior de la página y abren un campo de texto con la ruta; el lanzador verifica que sea un directorio y muestra el error de lo contrario, incluyendo cuando la carpeta haría que el conteo efectivo de proyectos supere 9 (`SUPER + CTRL + 1..9` no puede manejar más). Cada carpeta configurada, marcada como raíz (sus subcarpetas son proyectos) o un proyecto individual, obtiene su propio bloque debajo del formulario con una fila **Eliminar** que pide confirmación. La página invoca `argvus-projects`, por lo que los cambios son escritos por el lanzador y se aplican la próxima vez que se ejecute `SUPER + O` o `SUPER + CTRL + 1..9`. Si `argvus-projects` no está instalado, la página lo indica.
- **Otras páginas del sistema** muestran acciones de reinicio, aplicar, eliminar o restaurar solo cuando el proveedor subyacente las soporta.

Eliminar una preferencia de usuario puede hacer que un proveedor vuelva a su estado predeterminado/paquete, pero borrar archivos manualmente no es un procedimiento general de recuperación. Prefiera la acción de restablecer de la página o el comando específico de la función documentada.

## Puntos de entrada CLI

El ayudante es `argvus-control-center`, ejecutado a través de `argvus control-center`. Puede abrir áreas específicas directamente, por ejemplo:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```
Ejecute `argvus control-center --help` para las rutas disponibles en la versión instalada. La ruta de la línea de comandos es un acceso directo a la misma aplicación de configuración, no un sistema de configuración separado.

Las rutas directas útiles incluyen `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (con las páginas `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` y `proxy`), `audio`, `bluetooth`, `boot`, `packages` (con las páginas `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` y `mirrors`), `storage`, `system-doctor` (`diagnostics` y `diagnostic` permanecen alias), `about` (con las pestañas `system`, `about`, `donate`, `credits` y `copyright`), `power`, `session`, `displays` y `appearance`. La apariencia también acepta rutas enfocadas como `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` y `widget-telemetry`; la pantalla acepta `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` y `hdr`. Use la salida `--help` instalada al crear scripts porque la disponibilidad todavía depende de la versión instalada.

## Relacionado

- [Primera configuración](/docs/getting-started/)
- Panel de control
- [Apariencia](/docs/user-guide/appearance/)
- [Atajos de teclado](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Ratón y panel táctil](/docs/argvus-hyprland/input/)

### Doctor del sistema

Abra **Herramientas y Diagnósticos → Doctor del sistema**, búsquelo o ejecute `argvus control-center system-doctor`. Las rutas y selectores de categoría anteriores `diagnostics` y `diagnostic` siguen estando disponibles. Presione `r` para actualizar, Enter para inspeccionar una verificación, `o` en un detalle para abrir su página relacionada, y Esc para regresar. La recopilación se ejecuta en segundo plano y es de solo lectura.

Las verificaciones utilizan los conteos de unidades fallidas de systemd, herramientas gráficas y la instantánea de almacenamiento para el uso del sistema de archivos y la salud SMART. Un problema requiere evidencia; las herramientas no disponibles o los datos SMART inaccesibles se muestran como no disponibles, y una sonda fallida o con tiempo de espera es un error. Las categorías sin evidencia recopilada no implican una verificación exitosa. Las actualizaciones de paquetes utilizan la base de datos local de pacman y no sincronizan repositorios; un archivo de bloqueo puede estar obsoleto y no prueba que se esté ejecutando una operación. SMART puede requerir permisos y no se solicita acceso elevado. El resumen registra la última colección completada como tiempo Unix.

Storage y System Doctor comparten una instantánea en memoria durante hasta 10 segundos. Presionar `r` recopila explícitamente datos frescos; las solicitudes concurrentes reutilizan una colección que acaba de completarse. La salida de los subprocesos de Diagnostic y Storage está limitada a 1 MiB por flujo, y la salida de tamaño excesivo se trata como una colección fallida.

## Luz nocturna

Abra **Escritorio y Entrada > Pantallas > Luz Nocturna** o ejecute `argvus control-center displays night-light`. Configure la temperatura, la activación manual y los horarios locales fijos a través de `argvus config display night-light`. Vea [Luz Nocturna](/docs/user-guide/night-light/) para comandos, estado y requisitos de tiempo de ejecución.

## Instantánea

Abra **Centro de Control > Administración del Sistema > Instantánea**, busque `Snapper`, o ejecute `argvus control-center snapshot`. La función `snapshot` Cargo está incluida en `full` (por defecto), y es independiente de `storage`. La página está marcada como **Experimental** porque la función todavía está en desarrollo y puede estar incompleta.

Esta página gestiona únicamente una configuración existente de Snapper cuyo `SUBVOLUME` es `/` y cuyo `FSTYPE` es `btrfs`. Verifica el montaje que contiene `/` con `findmnt`; no se acepta una configuración llamada `root` que apunte a otro lugar. Varias configuraciones coincidentes requieren una selección. El paquete Arch `snapper` es opcional y debe instalarse manualmente. Snapper debe admitir `--jsonout` y columnas activas/predeterminadas explícitas. No se crea ninguna configuración automáticamente.

En una raíz que no es Btrfs, la página muestra **Este sistema de archivos no es Btrfs** y no ofrece operaciones de instantáneas. La falla de detección, la falta de Snapper, la falta de configuración de la raíz, la denegación de permisos y los errores del backend son estados separados. Si se niegan las consultas de usuario normal, presione **a — Autorizar** para solicitar acceso a Polkit. Las acciones administrativas se ejecutan en el asistente restringido `system-settings`, no en un TUI elevado; la autenticación puede solicitarse nuevamente en consultas posteriores. Se requiere un agente gráfico de Polkit existente para la autenticación.

Teclas: **↑/↓** seleccionar, **Enter** detalles/configuración, **c** crear, **d** eliminar, **p** políticas de retención, **r** actualizar, **Esc** regresar, **?** ayuda. En la vista de retención, **↑/↓** seleccionar una política, **e** editar, **c** alternar algoritmos de limpieza y **Enter** abrir confirmación. La creación abre una descripción opcional (hasta 256 caracteres imprimibles); una descripción vacía usa “Snapshot manual de ARGVUS”. Tanto la creación como la eliminación requieren confirmación, inicialmente centrada en Cancelar. La eliminación se aplica a un solo snapshot seleccionado y rechaza el snapshot 0, el snapshot actualmente activo y el snapshot predeterminado. El asistente vuelve a comprobar estas condiciones antes de invocar a Snapper. Las listas se actualizan después de las operaciones; una operación interrumpida nunca se reintenta automáticamente. Primero inspeccione la lista actualizada.

Para comparar snapshots, seleccione la base y presione **b**, luego seleccione un destino diferente y presione **v**. La página muestra los ID ordenados, conteos de creados,
rutas eliminadas y modificadas, y los seis indicadores de estado de Snapper. Seleccione una ruta y presione **Enter** para solicitar la diferencia de contenido; **Esc** regresa a las rutas cambiadas y luego a la lista de instantáneas. Los indicadores codifican creación/eliminación o contenido/tipo en la posición 1, permisos en 2, propietario en 3, grupo en 4, atributos extendidos en 5 y ACL en 6. Un punto significa que no hay cambio en esa posición. Estado lista rutas; diff muestra contenido textual. La salida de diff binaria/no UTF-8 no se renderiza. La salida de diff superior a 256 KiB se informa como truncada y no se muestra. Las líneas de estado ambiguas o malformadas son rechazadas. El formato de línea de Snapper no puede representar rutas con saltos de línea de manera inequívoca; tales nombres no son compatibles. Los errores de permisos se pueden reintentar mediante **a — Autorizar**; una instantánea eliminada o una configuración raíz cambiada requiere actualizar la lista. Las pruebas del backend utilizan un `ProcessRunner` falso y no requieren Snapper, Btrfs ni root.

A partir de la comparación, seleccione una ruta y presione **r** para revisar la restauración del archivo desde la instantánea de destino. El asistente verifica que ambas instantáneas sigan perteneciendo a la configuración raíz, que la ruta todavía esté en la comparación y que la fuente sea un archivo regular en la vista estándar `/.snapshots/<id>/snapshot` de Snapper. La diferencia acotada de Snapper se muestra cuando es textual; las diferencias binarias o de gran tamaño se identifican sin cargar su contenido completo en la interfaz de usuario. **Enter** abre una confirmación separada que nombra la fuente y el destino; **Esc** cancela sin cambiar el archivo. La confirmación crea primero una instantánea de seguridad `single`, luego utiliza Snapper `undochange` solo para esa ruta y verifica que coincida con la instantánea seleccionada. Se informa el ID de la instantánea de seguridad y la instantánea se conserva. La restauración comparte el bloqueo usado por las actualizaciones de paquetes del paquete ARGVUS, por lo que no puede superponerse con una transacción oficial de pacman del Centro de Control.

La primera versión acepta solo archivos regulares propiedad de root bajo directorios padres propiedad de root que no sean escribibles por grupo u otros. Rechaza directorios, enlaces simbólicos, enlaces duros, archivos especiales, puntos de montaje/subvolúmenes, el directorio interno de Snapper y ubicaciones escribibles por el usuario. Los archivos que faltan en la instantánea seleccionada nunca se eliminan. `undochange` de Snapper no garantiza la consistencia de la aplicación o la base de datos y no es atómico; la instantánea de seguridad es un punto de recuperación, no una garantía. Snapper conserva solo los metadatos soportados por su compilación. Subvolúmenes separados y archivos fuera de la raíz configurada permanecen fuera de la operación.

Las instantáneas manuales son `single`, de solo lectura por defecto de Snapper, sin algoritmo de limpieza asignado por esta página. Los temporizadores existentes de Snapper y operaciones externas continúan aplicándose. Las instantáneas no son imágenes de sistema completo: subvolúmenes separados,
incluyendo un `/home` separado, no están incluidos. Esta versión **no proporciona reversión completa del sistema, instalación ni creación de una configuración de Snapper**. Presione **p** en una configuración raíz compatible para inspeccionar los ajustes de retención conocidos. Los valores faltantes se etiquetan como valores predeterminados ARGVUS, no como valores leídos de la plantilla de creación de Snapper instalada; las claves desconocidas son solo para diagnóstico. Presione **e** para editar un valor compatible. La confirmación muestra los valores antiguos y nuevos; después de guardar, la página lee la configuración nuevamente y la verifica. Presione **c** para ciclar los algoritmos de limpieza y **Enter** para revisar y confirmar uno. La limpieza utiliza el algoritmo oficial de Snapper; no hay una vista previa exacta, y el recuento de elementos eliminados se mide comparando listas después de la ejecución. Arch Snapper 0.13.2 proporciona `snapper-timeline.timer` y `snapper-cleanup.timer`; la creación de la línea de tiempo requiere
el temporizador de la línea de tiempo y las políticas de limpieza configuradas no se ejecutan sin la programación de limpieza. La página solo lee el estado de la unidad y no habilita temporizadores. Los límites de espacio requieren qgroups Btrfs configurados y no se exponen aquí.

### Operaciones de paquetes y snapshots

La instalación de paquetes, actualizaciones y eliminación no requieren Snapper, Btrfs o una configuración de snapshots. **Actualizar todo** ejecuta `pacman -Syyu` a través del ayudante con privilegios con `pkexec`; las actualizaciones oficiales individuales ejecutan `pacman -S <package>`. ARGVUS no crea snapshots automáticos para estas operaciones. La página de Snapshots permanece independiente y conserva sus propios requisitos de Btrfs/Snapper. Los hooks configurados por separado en pacman permanecen controlados por el sistema.

### Validación manual

- En una raíz ext4/xfs, abra la página en inglés y portugués. En portugués, verifique el mensaje exacto **Este sistema de arquivos não é Btrfs** y verifique que c/d no abran una operación.
- En una VM desechable Btrfs, pruebe Snapper ausente, Snapper sin una configuración de raíz y una configuración llamada `root` apuntando a otro subvolumen.
- Con una configuración de raíz existente, liste las instantáneas, abra detalles, cree una instantánea con espacios/caracteres especiales, cancele una eliminación y luego elimine solo la instantánea recién creada después de confirmar su número y descripción.
- Verifique la autenticación denegada/cancelada, las instantáneas activas/protegidas por defecto, refresque, navegación por teclado, cambio de tamaño y ambos idiomas. No pruebe eliminación contra las instantáneas necesarias para la recuperación.

Solo el paquete `argvus` cambia. Reconstruya desde `de/argvus` con `make build` para producir el paquete `.pkg.tar.zst`; instalarlo es una acción separada del operador.

El idioma bajo Idioma y Región guarda la configuración regional en el perfil del usuario (`~/.config/locale.conf`) sin cambiar `/etc/locale.conf`. La sesión utiliza esta preferencia después de iniciar sesión nuevamente. Durante Bienvenida, F6 Siguiente o el botón Siguiente del pie de página registran la revisión y avanzan; Shift+Tab enfoca el botón y Enter lo activa, incluyendo en las páginas de información.

Bluetooth aparece en Bienvenida solo cuando el Centro de Control detecta un adaptador. El progreso existente para un paso de Bluetooth oculto se conserva.

El idioma y la distribución del teclado de la sesión son preferencias personales y no requieren privilegios de administrador. Los perfiles nuevos predeterminadamente usan inglés y la distribución de teclado de EE. UU.; las preferencias personales existentes se conservan. La fecha/hora, NTP, la generación de la configuración regional del sistema y el mapa de teclado de la consola (TTY) mantienen su comportamiento administrativo a nivel del sistema.

Para un teclado de sesión brasileño, seleccione `br`, aplique el diseño, luego seleccione `abnt2` en Variante. Esto escribe la configuración personal de Hyprland sin autorización administrativa; el mapa de teclas de la consola es separado. ABNT2 usa teclas muertas: `~` seguido de `a` produce `ã`, y `~` seguido de Espacio produce `~`. Aplicar la variante activa su diseño predeterminado configurado; los fallos de activación se reportan.

La Zona Horaria guarda TZ bajo `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` sin autorización. Cierre sesión y vuelva a iniciarla para que las aplicaciones lo hereden. Fecha y Hora continúa mostrando y editando el reloj del sistema en la zona horaria del sistema, con autorización administrativa; NTP no cambia.

Los perfiles de pantalla y las asignaciones de espacios de trabajo se aplican a través de la configuración Lua generada y una recarga de Hyprland. La energía del monitor utiliza el despachador Lua DPMS soportado por Hyprland 0.56.2.

La potencia de renderizado de sombras está limitada de 1 a 4 (valor por defecto 4) para la compatibilidad con Hyprland 0.57. Los valores antiguos almacenados por encima de 4 se ajustan al cargar la configuración.

## Oscilación de ventana

Abre **Escritorio y Entrada → Hyprland → Animaciones → Ventanas** y usa los controles de
**Oscilación**. Busca `wobble`, o abre la página directamente:

```sh
argvus control-center hyprland animations windows
```
El vaivén está desactivado por defecto. Sus valores predeterminados de ARGVUS son rigidez `100`, amortiguamiento `10`, masa `1`, intensidad `1` y resolución de malla `8`. La rigidez y la masa aceptan `0.0001–1000`, amortiguamiento `0–1000`, intensidad `0–2`, y la malla un número entero `2–32`. Una mayor resolución de malla incrementa el costo de renderizado. Usa las teclas de flecha para ajustar los valores o Enter para escribirlos. **Restaurar predeterminados de Vaivén** cambia el borrador; **Aplicar** guarda todas las ediciones de animación pendientes y recarga Hyprland una vez, sin reiniciar la sesión. Volver dentro de Animaciones mantiene el borrador compartido.

El interruptor global de Animaciones suprime el Vaivén mientras retiene sus configuraciones individuales. Los valores canónicos se encuentran bajo `effects.wobble_*` en el estado de ARGVUS; `hypr/state.lua` se genera y no debe editarse manualmente.

Esto requiere Hyprland **0.57.0 o más reciente**, impuesto por la dependencia del paquete de `argvus-hyprland`. La versión 0.56 no es compatible con esta configuración. El máximo de intensidad sigue el código fuente [lanzado 0.57.0 ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), que lo limita a `2`, aunque la documentación más reciente puede listar `3`.

Bienvenido se abre con el logo de ARGVUS y WELCOME. Presione F6 Siguiente para mostrar la lista de verificación sin completar un paso.

Las instalaciones nuevas predeterminadamente usan la zona horaria personal `America/New_York` y solo el diseño de teclado `us` habilitado. Los ajustes personales existentes se conservan; la zona horaria del sistema no cambia.

Las opciones del módulo de la barra de tareas para Batería y Bluetooth aparecen solo cuando se detecta el hardware correspondiente. Ocultar Mantener despierto oculta su indicador sin cambiar el inhibidor de suspensión. Utilities admite un cajón desplegable colapsado. La presentación de íconos del espacio de trabajo muestra íconos de las ventanas abiertas; la bandeja del sistema controla los íconos de aplicaciones en segundo plano.

La página de atajos de teclado incluye **selector de agente de IA (paquete argvus-ai opcional)** con la combinación predeterminada SUPER + A. Usa el ID del catálogo app.ai, por lo que la reasignación y desactivación siguen la misma configuración canónica de keyboard_shortcuts que otras entradas. Su enlace ejecutable en Lua lo proporciona argvus-ai; la entrada del catálogo por sí sola no lo activa sin ese paquete opcional.


Centro de Control > Apariencia > Barra de tareas > Íconos > Módulos opcionales ahora incluye **Agentes de IA**. Está habilitado por defecto y se guarda en /taskbar/icons/ai_agents_enabled. Actívalo en el borrador de la barra de tareas y haz clic en Aplicar para conservarlo y regenerar el grupo de utilidades; deshabilitarlo no cambia el atajo SUPER + A ni las entradas del lanzador de aplicaciones.

Los interruptores de inicio automático editan la entrada `.desktop` del usuario o crean una copia de usuario de una entrada del sistema. Los nombres, comandos y banderas de activación provienen de `[Desktop Entry]`; los grupos adicionales de acciones de escritorio permanecen sin cambios.

Las confirmaciones de servicio conservan el alcance usuario/sistema seleccionado junto con la unidad y la acción. Una unidad con el mismo nombre en otro alcance no puede redirigir una acción confirmada; las páginas de detalle también conservan el alcance original.

El almacenamiento incluye puntos de montaje anidados como `/home`, `/boot` y `/boot/efi` del inventario del sistema de archivos. El diagnóstico utiliza el mismo inventario para sus comprobaciones existentes de uso del sistema de archivos root y home.

Los nuevos snapshots de seguridad para restauración utilizan el algoritmo de limpieza `number` de Snapper. Su eliminación sigue la política de retención configurada cuando se ejecuta la limpieza. Los snapshots existentes sin un algoritmo de limpieza no se modifican, y la creación manual mantiene su política existente.

Las comparaciones de snapshots reportan registros de rutas no compatibles por separado. Los archivos válidos permanecen disponibles para revisión; las rutas con codificación inválida, caracteres de control o componentes no compatibles no ofrecen acciones de diferencia o restauración.

La configuración de energía escribe el complemento ARGVUS logind bajo `[Login]`, creando su directorio cuando es necesario. Se informan los fallos de recarga. El comportamiento de la tapa de la batería se asigna a `HandleLidSwitch`; el contexto heredado `default` es un alias, mientras que el comportamiento con CA y en estación de acoplamiento utiliza sus propias claves logind.

La aplicación Mirrorlist acepta solo comentarios, líneas en blanco y entradas `Server` HTTP/HTTPS. Las cargas útiles están limitadas a 64 KiB y se verifican antes de la elevación y nuevamente por el asistente privilegiado; las directivas de pacman como `SigLevel` y `XferCommand` son rechazadas.

La limpieza de caché para paquetes desinstalados no retiene versiones en caché. Las operaciones de repositorio rechazan includes compartidos por múltiples secciones del repositorio. Se rechazan nombres de host con espacios en blanco alrededor y una selección de locales generados vacía. Las escrituras de archivos de configuración rechazan destinos con enlaces simbólicos y archivos temporales existentes.

Las búsquedas de paquetes y la carga de páginas descartan los resultados de una página o consulta anterior. La página actual se carga nuevamente, por lo que un resultado antiguo no puede marcar una vista diferente como recién actualizada.

La salida de la transacción se sanitiza y se limita. La pantalla retiene hasta 10,000 líneas o 4 MiB e informa la salida omitida. Los flujos de procesos capturados respetan su límite configurado, por defecto 8 MiB para la ejecución en vivo; el exceso de salida se drena y se informa después de que el proceso termina.

La lectura del inventario de arranque utiliza el kernel `pkgbase` de Arch para localizar imágenes de kernel, archivos initramfs y preajustes de mkinitcpio mientras se muestra la versión del kernel por separado. Se leen arrays literales de mkinitcpio sin incluir sus paréntesis ni ejecutar la configuración.

Los cambios de energía para la tapa, el botón de encendido y mantener activo se guardan en la configuración canónica solo después de que la acción en tiempo de ejecución tenga éxito. Cancelar la autorización conserva el valor guardado previamente intacto. Cuando ningún drop-in de logind de ARGVUS proporciona un valor, se lee la configuración principal de logind como respaldo.

Después de que un cambio de retención falla, la página vuelve a leer la configuración actual antes de mostrar sus valores nuevamente. La selección de configuración comienza en la primera entrada.

Las operaciones del paquete ARGVUS comparten un bloqueo exclusivo con la restauración selectiva. La instalación, eliminación, reinstalación, degradación, actualización de metadatos, limpieza de caché y cambios en el repositorio o espejo se rechazan mientras se mantiene ese bloqueo. Esta coordinación cubre las operaciones iniciadas a través del asistente de ARGVUS; las herramientas externas pacman y AUR no usan este bloqueo.

Mientras una operación en segundo plano aplica cambios, `q` y `Ctrl+C` mantienen abierto el Centro de Control y muestran un aviso para esperar. Esto también se aplica después de navegar fuera de la página que inició la acción. Espere su resultado y luego solicite la salida nuevamente. Las actualizaciones ordinarias de solo lectura no bloquean la salida.

Los campos de contraseña Wi-Fi, DNS manual y búsqueda de red aceptan pegado entre corchetes. Los controles pegados y los saltos de línea se omiten. Los atajos de caracteres no compatibles solo con Ctrl o solo con Alt no insertan letras. Los editores aceptan eventos imprimibles de AltGr codificados como Ctrl+Alt; Ctrl+S guarda solo sin Alt. El editor de texto compartido mantiene la navegación vertical en columnas visuales y alinea el desplazamiento horizontal para caracteres anchos.

Los cambios de volumen de audio y de silencio para micrófonos y salidas no predeterminadas ya no sobrescriben las preferencias de salida predeterminada guardadas. El backend conserva esas preferencias solo cuando puede identificar el nodo seleccionado como la salida predeterminada actual. La página muestra volúmenes amplificados por encima del 100 % según lo informado por PipeWire; las ediciones de volumen explícitas continúan limitadas al 0–100 %.

La descripción general del hardware identifica los modelos de GPU por separado de su dirección PCI y clase de dispositivo, preservando los nombres comerciales entre corchetes. Su resumen muestra el renderizador OpenGL consultado para la sesión y todos los nombres de dispositivos devueltos por la prueba de Vulkan, incluidos los dispositivos de software. Estos resultados globales no se presentan como capacidades de cada GPU física.

Cuando un administrador bloquea o elimina la cuenta configurada para inicio de sesión automático, el proveedor de cuentas primero desactiva su entrada de inicio de sesión automático en greetd. Si esa limpieza falla, se rechaza la operación de la cuenta. Si posteriormente falla el bloqueo o la eliminación, el inicio de sesión automático permanece desactivado. El inicio de sesión automático para otra cuenta se conserva; desbloquear una cuenta no habilita el inicio de sesión automático. Las sesiones existentes no se cierran y greetd no se reinicia.

## Crear y gestionar atajos de teclado

**Escritorio e ingreso → Atajos de teclado** abre un menú con **Crear atajos** y **Todos los atajos**. Los alias de CLI existentes (`keybindings`, `shortcuts`, `keyboard`) abren este menú; agregar `create` o `list` abre un submenú directamente. La búsqueda global también encuentra a ambos submenús.

**Crear accesos directos** solicita una descripción, combinación de teclas y tipo de acción. Seleccione **Acceso directo actual** y presione `Enter`, luego presione la combinación de teclas deseada. **Cambiar acceso directo** comienza la misma captura; ninguno abre un editor de texto. Capturar cambios solo modifica el borrador. Si una combinación ya está en uso, el conflicto se muestra de inmediato, incluso antes de completar la descripción y el comando. Confirmar **Reemplazar** selecciona la combinación y registra su elección; no crea el acceso directo ni desactiva otro hasta **Crear**/**Guardar**. Cancelar el conflicto preserva la combinación anterior. Un **Comando** se ejecuta cuando se presiona el acceso directo; la creación nunca lo ejecuta. Un **despachador de Hyprland** usa un nombre de despachador Lua compatible, como `window.float`, con argumentos opcionales en forma de objeto JSON, por ejemplo `{"action":"toggle"}`. Submapas, enlaces de ratón y banderas avanzadas permanecen fuera de este formulario.

Usa `Tab` para alternar entre el formulario y **Acciones**, `←/→` para moverte entre **Crear** y **Cancelar**, y `Enter` para ejecutar la acción seleccionada. Cancelar o `Esc` descarta el borrador. Después de crear, **Todos los Atajos** se abre con el nuevo elemento seleccionado y visible. Las entradas personalizadas aparecen en **Atajos Personalizados**; la búsqueda coincide con la descripción, las teclas y la acción. `Enter` edita la descripción, la acción y las teclas, `Space` habilita/deshabilita, y seleccionar la fila de **Eliminar** y presionar `Enter` abre una confirmación enfocada en Cancelar. Confirmar elimina el registro y regresa a la lista actualizada; cancelar lo mantiene.

Los atajos habilitados en el mismo contexto no pueden reutilizar una combinación silenciosamente: los conflictos configurables requieren confirmación para reemplazar, y los atajos protegidos no pueden ser reemplazados. El reemplazo guarda el nuevo atajo y deshabilita los conflictos en una única transacción de configuración. **Restaurar predeterminado** se aplica solo a entradas oficiales; **Restaurar todos los atajos** conserva los registros personalizados. Si la restauración chocaría con un atajo personalizado activo, primero cambia o deshabilita ese atajo personalizado.

Guardar y aplicar se ejecutan en segundo plano. Si el guardado falla, el borrador permanece disponible. Si la aplicación a la sesión falla después de guardar, la página indica que se guardó y mantiene el mismo ID para reintentos. Los registros personalizados se encuentran en `/hyprland/custom_shortcuts` en la configuración canónica, proyectados a `data/generated/hypr/custom-shortcuts.lua`; no edites archivos generados.

Los errores de atajos permanecen dentro del TUI: la ventana emergente ajusta su tamaño al mensaje envuelto y mantiene **Enter OK** visible. El diagnóstico técnico de atajos va al diario local (`journalctl -t argvus-control-center`) en lugar de stdout/stderr. Una eliminación que se guarda pero no se puede aplicar informa esa distinción; recargue la configuración de la sesión para eliminar la vinculación previamente activa.

**Estado de actualización** en **Barra de tareas → Iconos → Módulos opcionales** muestra la cantidad de actualizaciones disponibles y está habilitado por defecto. **Comprobar AUR** está deshabilitado por defecto y requiere `paru` o `yay`; aplique el borrador de la barra de tareas para guardar cualquiera de las configuraciones. Las comprobaciones se ejecutan al inicio y cada hora; hacer clic en el indicador abre la página oficial de actualizaciones de paquetes.

## Modo de ventana global

Abre **Centro de Control → Escritorio y Entrada → Hyprland → Ventanas → Modo** y selecciona **Ventanas en mosaico (Predeterminado)** o **Ventanas flotantes**. El mismo selector está disponible desde el ícono de modo de ventana en el grupo de utilidades de la barra de tareas.

La preferencia se aplica a cada monitor y espacio de trabajo y se recuerda después de cerrar sesión. Flotar convierte las ventanas de aplicaciones existentes y las ventanas recién abiertas. El mosaico solo restaura las ventanas que esta política convirtió; los cuadros de diálogo y otras ventanas que ya estaban flotando permanecen flotantes. Una ventana fijada explícitamente mientras Flotar está activo también permanece flotante para preservar su fijación. El diseño en mosaico se recalcula; su árbol de división exacto anterior no se restaura.

`SUPER + SHIFT + Space` mantiene su comportamiento normal por ventana en Tile. En Floating no cambia una ventana de nuevo a tiling. Los paneles, la barra de tareas y las notificaciones son superficies de layer-shell y no se ven afectados. Pantalla completa y maximización continúan usando el comportamiento normal de Hyprland.

La preferencia canónica es `/hyprland/window_mode` en `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Su proyección generada es `data/generated/hypr/window-mode.lua`; no la edite. El registro de restauración por ventana se encuentra bajo `$XDG_RUNTIME_DIR/argvus`, con alcance en la instancia del compositor, y sobrevive a recargas de configuración. No se reutiliza en una nueva sesión.

Si la aplicación está pendiente o falla, la interfaz distingue esto de la preferencia guardada. Seleccione la opción nuevamente para reintentar. Sin un compositor, la preferencia se guarda para el próximo inicio de sesión.

## Botones de ventana y barra de aplicaciones

El modo flotante agrega una barra de título ARGVUS con **Minimizar**, **Maximizar / Restaurar** y **Cerrar** a las ventanas de aplicaciones normales. Hacer doble clic en la barra de título alterna la maximización. La decoración adicional está oculta en Mosaico y pantalla completa. Las aplicaciones que dibujan sus propios controles pueden mostrar ambas barras; los controles uniformes de ARGVUS son intencionales.

Un segundo Waybar aparece en cada monitor, en el borde opuesto a la barra de tareas principal. Listará únicamente las ventanas del espacio de trabajo actual de ese monitor, incluidas sus ventanas minimizadas. Las pestañas de grupo ocultas no crean entradas adicionales. Las entradas minimizadas mantienen su asociación original con el monitor/espacio de trabajo. Haz clic en una ventana inactiva para activarla, en una ventana activa para minimizarla o en una ventana minimizada para restaurarla; su menú contextual ofrece restaurar, minimizar, maximizar/restaurar y cerrar. Los íconos de aplicaciones desconocidas se reemplazan por un símbolo de ventana.

`SUPER + Alt + Up` coloca la barra principal en la parte superior y la barra de ventana en la parte inferior. `SUPER + Alt + Down` hace lo contrario. Repetir una dirección deja las posiciones sin cambios. Tile solo tiene la barra principal. Volver a Tile restaura todas las ventanas minimizadas antes de quitar la barra de ventana y luego preserva la política original de mosaico/flotante.

La minimización utiliza el espacio de trabajo reservado `special:argvus-minimized`. No use ese espacio de trabajo manualmente. El proceso `argvus windows service` registra la identidad, el espacio de trabajo/monitor original, la geometría, el estado de pantalla completa/maximización y el estado de fijación en un registro de tiempo de ejecución específico de la instancia del compositor bajo `$XDG_RUNTIME_DIR/argvus`. Los registros sobreviven a un reinicio del servicio o recarga de configuración, pero no cruzan sesiones del compositor. Cerrar una ventana elimina su registro. Si un monitor desaparece, la restauración utiliza una salida disponible. Una barra de recuperación faltante o con fallo evita nuevas minimizaciones; la restauración fallida conserva el registro y la barra de recuperación.

La sesión posee `argvus-windows.service` y `argvus-window-bar.service`. Esta última mantiene su hijo Waybar inactivo en Tile. El plugin hyprbars está incluido por `argvus-hyprland`, construido para la ABI de Hyprland 0.57.0 en x86_64; no se compila en el directorio de inicio del usuario. Reconstruya el plugin con una actualización de ABI del compositor.
Comandos: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` y `minimize|restore|activate|maximize|close ADDRESS`. Las direcciones de ventana provienen de la lista. `position top --persist` almacena/proyecta la preferencia sin reiniciar la barra principal. Los detalles de configuración y errores en tiempo de ejecución se escriben en el diario; las fallas visibles de acción también muestran una notificación traducida.

## Submenú de ventanas y cierre de la aplicación

**Escritorio y Entrada → Hyprland → Ventanas** agrupa **Reglas**, **Espacios** y **Modo**.
Reglas edita la colocación por aplicación; Espacios edita los espacios internos y externos; Modo contiene el selector inmediato de Tile/Flotante, separado del formulario de reglas.
Buscar abre las mismas páginas, y Atrás devuelve a Ventanas antes de Hyprland.

Las rutas directas son `argvus control-center hyprland windows`, seguidas opcionalmente por `rules`, `spaces` o `mode`. Las rutas existentes `window-rules` y `appearance window-spaces` permanecen como alias y muestran las nuevas migas de pan.

`SUPER + Q` cierra la ventana dedicada del Centro de Control inmediatamente, sin la confirmación de la ventana del sistema de Kitty. Los borradores no aplicados se descartan cuando la ventana se cierra de esta manera. La anulación está limitada a la clase de ventana `argvus-control-center`; los terminales regulares y otras TUI mantienen su política de confirmación al cerrar.

`SUPER + Alt + Space` alterna el modo global de Azulejo/Flotante, utilizando la misma preferencia que la barra de tareas y el Centro de Control. El comando equivalente es `argvus config window-mode toggle`. Las invocaciones concurrentes se serializan; alternar utiliza la preferencia guardada incluso cuando la aplicación está pendiente. El atajo se puede cambiar o desactivar en Atajos de Teclado; `SUPER + Shift + Space` mantiene su acción de ventana individual.
