---
title: Redes y Bluetooth
description: Administra NetworkManager y Bluetooth a través de ARGVUS.
---
El subcomando `argvus network` informa y controla NetworkManager, y `argvus bluetooth` informa y controla el estado y la disponibilidad del adaptador Bluetooth. Ambos se ejecutan en proceso dentro de `argvus`; `networkctl` y `bluetoothctl` permanecen como alias. El Panel de Control y Waybar llaman a estos comandos.

Use `--help` en cualquiera de los comandos para los subcomandos instalados. La integración de la interfaz de usuario de Red y Bluetooth es independiente de gestores avanzados opcionales como Blueman.

El Centro de Control separa el estado de la red de las páginas de configuración: **Red → Estado** resume el estado actual, mientras que interfaces, Wi-Fi, Ethernet, VPN, DNS, proxy y firewall son rutas separadas cuando sus proveedores están disponibles. **Bluetooth → Estado**, **Dispositivos** y **Emparejar** cubren el estado del adaptador y las acciones de los dispositivos. Estas páginas realizan acciones del sistema respaldadas por proveedores; la barra de tareas y el Panel de Control muestran la superficie rápida de estado/acciones.

Los cambios en la red y en Bluetooth pueden requerir autorización y pueden aplicarse de inmediato a través de NetworkManager o del servicio de Bluetooth. No son lo mismo que cambiar el indicador de la barra de tareas ARGVUS.

El Centro de Control envía las contraseñas de Wi-Fi a nmcli a través de la entrada estándar con `--ask`; las contraseñas se excluyen de los argumentos del proceso y se ocultan en los errores de comando devueltos.

## Gestión de red desde la línea de comandos

Use `argvus networkctl` para consultar y controlar la red sin la interfaz gráfica:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## Gestión de Bluetooth desde la línea de comandos

Usa `argvus bluetoothctl` para el adaptador Bluetooth:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
Cuando el kernel no lista ningún controlador en `/sys/class/bluetooth`, `status` responde a `status=no-controller` inmediatamente, sin consultar a `bluetoothctl` (lo cual de otro modo esperaría a BlueZ cuando el servicio está detenido). El módulo de la barra de tareas y las tarjetas del Panel de Control leen el mismo estado.

Use `--help` en cualquiera de los comandos para ver opciones adicionales y subcomandos específicos de su versión instalada.

Mientras el cuadro de diálogo de la contraseña de Wi-Fi está abierto, las actualizaciones de escaneo preservan el SSID y el tipo de seguridad seleccionados. Si esa red desaparece o cambia su seguridad, la confirmación se detiene con un error y requiere una nueva selección.

El DNS manual envía servidores IPv4 e IPv6 a sus respectivas configuraciones de NetworkManager antes de actualizar el archivo del resolvedor. Los enlaces simbólicos del resolvedor gestionados por el servicio se preservan al negarse a reemplazar el archivo. Un retorno exitoso al DNS automático restaura y elimina la copia de seguridad, por lo que un ciclo manual posterior guarda el estado actual del resolvedor.

Cerrar el editor de perfiles de red espera a que su trabajo actual termine, por lo que guardar o eliminar un perfil no puede ser interrumpido al descartar el editor. Las actualizaciones en segundo plano siguen siendo cancelables.

Las solicitudes de conexión Wi-Fi comparten la protección de acciones de red: una acción en curso o un trabajo del editor de perfiles se completa antes de que otra mutación pueda comenzar. Los mensajes de progreso y finalización siguen el idioma seleccionado.

La confirmación de eliminación de Bluetooth mantiene la dirección original del dispositivo incluso cuando se actualiza la lista de dispositivos. Se reconocen los porcentajes de batería provenientes de bluetoothctl, y las consultas de detalles del dispositivo se ejecutan con concurrencia limitada y tiempos de espera individuales.

La contraseña de Wi-Fi, los campos de búsqueda de red y el DNS manual aceptan pegado entre corchetes. Se omiten los controles y saltos de línea pegados. Los atajos de caracteres no soportados solo con Ctrl o solo con Alt no insertan letras. Los editores aceptan eventos AltGr imprimibles codificados como Ctrl+Alt; Ctrl+S guarda únicamente sin Alt. El editor de texto compartido mantiene la navegación vertical en columnas visuales y alinea el desplazamiento horizontal para caracteres anchos.
