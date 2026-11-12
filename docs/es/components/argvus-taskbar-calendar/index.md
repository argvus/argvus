---
title: Calendario
description: Usa el calendario de la barra de tareas ARGVUS.
---
`argvus-taskbar-calendar` es el calendario emergente de GTK4 abierto desde el módulo de fecha de Waybar o con:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
Admite eventos locales, recordatorios e importación/exportación ICS. Su servicio de usuario es `argvus-taskbar-calendar.service`. Use `argvus taskbar-calendar --help` para las opciones compatibles con la versión instalada.

El calendario sigue el tema activo ARGVUS, incluyendo las variantes Sticky y Spaced. Los cambios de tema se aplican mediante la caché de usuario generada; use `argvus taskbar-calendar reload` si un popup en ejecución necesita una actualización explícita.

Cuando se abre desde el módulo de fecha de la barra de tareas, el popup se ancla al borde de la barra: se abre debajo de una barra de tareas superior y encima de una barra de tareas inferior, alineado al borde derecho del módulo de fecha. Los lanzamientos directos sin geometría de la barra de tareas usan la ubicación del puntero o la monitorización como alternativa.

## Comandos

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## Eventos y recordatorios

Los eventos están desactivados por defecto. Actívalos en la ventana emergente bajo **Configuración > Eventos**; la elección se guarda en el directorio de estado y también habilita e inicia `argvus-taskbar-calendar.service`, que muestra los recordatorios. Desactivarlos detiene el servicio.
- **+ EVENTO** (o `n`) abre el editor para el día seleccionado. Los eventos se pueden crear y modificar desde hoy en adelante; un evento de un día pasado aún se puede abrir y eliminar.
- Un nuevo evento comienza a las 09:00 y dura la longitud predeterminada establecida en Configuración. Mover la hora de inicio mantiene la duración; una hora de finalización anterior a la hora de inicio significa que el evento termina al día siguiente.
- Cada evento muestra una notificación con sonido al inicio, sin importar lo que digan las casillas de **Recordatorio** y **Repetir**. Un evento guardado después de haber empezado (un evento de todo el día creado durante el día, por ejemplo) notifica en un plazo de 30 segundos.
- **Recordatorio** agrega una notificación anticipada, el tiempo elegido antes del inicio (al inicio si ese tiempo ya ha pasado).
- En un evento de todo el día, el campo de hora se convierte en **NOTIFICAR A LAS**: el evento cubre todo el día y notifica a esa hora (00:00 notifica al inicio del día). La hora se mantiene cuando se reabre el evento.
- Para eventos de todo el día, **Repetir** notifica nuevamente en el intervalo elegido después de esa hora.
- Editar un evento sin mover su inicio no repite los recordatorios ya mostrados.
- Las notificaciones siguen el idioma del calendario (Configuración > Idioma).
- Un día muestra un punto (`•`) cuando hay un evento en él, incluyendo cada día de un evento durante la noche o de varios días. Los días siguen la zona horaria local.
- Los eventos locales se eliminan automáticamente 10 minutos después de que terminan. Si un evento ya ha terminado cuando se guarda, permanece disponible al menos 10 minutos después de guardarlo.

Los archivos importados `.ics` conservan sus fechas, zonas horarias (`TZID`) y alarmas (`VALARM`). Los eventos importados también notifican al inicio, además de sus propias alarmas. Los eventos recurrentes (`RRULE`) se almacenan y exportan, pero solo se muestra y recuerda su primera ocurrencia. Después de una actualización del paquete, el servicio de recordatorios finaliza y systemd inicia la nueva versión en un plazo de 30 segundos. `sync` solo informa los calendarios remotos configurados; la sincronización CalDAV aún no está implementada.

## Rutas

- Configuración: `/etc/argvus/taskbar/calendar/config.toml` (a nivel del sistema; un valor predeterminado se incluye con el paquete).
- Estilos: `/etc/argvus/taskbar/calendar/style.css` y `theme.css`; los colores del tema activo ARGVUS provienen del tema construido a través de la caché a continuación.
- Datos: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (o `~/.local/share/argvus-taskbar-calendar/`).
- Estado: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (o `~/.local/state/...`); el interruptor de eventos se mantiene allí.
- Caché: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (o `~/.cache/argvus-taskbar-calendar/`).

El color de resaltado se lee de `~/.config/argvus/data/.accent-color` y se aplica en tiempo de ejecución
encima de los colores del tema.

La selección automática de idioma sigue el `locale.conf` del usuario, con inglés como predeterminado. Las preferencias explícitas de idioma del calendario aún reemplazan la selección automática.

La barra de tareas del calendario, ARGVUS Focus, ARGVUS Clock y el Panel de Control son mutuamente excluyentes: abrir uno cierra los demás.

Los fallos de lectura del calendario se muestran por separado de una agenda vacía. Abrir la base de datos, cargar eventos diarios y cargar indicadores mensuales tienen mensajes distintos. Seleccione un día nuevamente para reintentar. Una actualización fallida conserva únicamente los datos previamente cargados para el mismo día o mes; los resultados válidos de la otra consulta siguen estando disponibles.

## Catálogos de idioma compartidos

La interfaz utiliza los catálogos compartidos `argvus-i18n` para los diez idiomas. Los textos se almacenan en `resources/locales/<locale>/taskbar-calendar.json` en el espacio de trabajo maestro y se instalan mediante `argvus`. El inglés es el idioma de respaldo; los nuevos idiomas ya no requieren un diccionario bilingüe local.
