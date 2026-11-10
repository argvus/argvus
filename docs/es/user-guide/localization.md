---
title: Localización
description: Usa ARGVUS en inglés o portugués.
---
`argvus-i18n` proporciona los catálogos compartidos, el comando `argvus i18n validate` y el módulo QML `org.argvus.i18n`. ARGVUS actualmente incluye catálogos en inglés y portugués.

Las aplicaciones nativas y los componentes de shell consumen los catálogos compartidos. Por lo tanto, una cadena traducida puede ser utilizada por más de un repositorio.

Use **Centro de Control → Idioma y Región** para configurar el idioma, la localidad regional, las localidades del sistema generadas, la zona horaria, la fecha/hora y el teclado. Los catálogos actuales de ARGVUS proporcionan inglés y portugués; esto es independiente de la lista de localidades del sistema que ha generado el sistema operativo.

Cambiar el idioma de visualización guarda la preferencia de ARGVUS y reinicia automáticamente `argvus-control-panel.service`, por lo que el Panel de Control refleja el cambio de inmediato. Otras aplicaciones y superficies del sistema pueden requerir una recarga de sesión o una nueva sesión. Generar o modificar entradas de `/etc/locale.gen` requiere autorización y ejecuta `locale-gen`; seleccionar una configuración regional que no esté generada es rechazado. La disposición del teclado, la variante y el mapa de teclas de la consola son configuraciones separadas y no deben confundirse con el idioma de la interfaz. 

## Cuando las variables de entorno anulan el cambio

Las aplicaciones ARGVUS detectan el idioma desde el archivo de idioma persistente de ARGVUS primero, luego `LC_ALL`, `LC_MESSAGES` y `LANG`. Por lo tanto, las variables de entorno locales pueden hacer que el Centro de Control parezca ignorar un cambio de idioma, especialmente cuando un servicio de usuario o una sesión gráfica conserva un entorno anterior.

Inspeccione los valores efectivos con:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
Si `~/.config/locale.conf` existe y define `LANG`, `LC_ALL` o `LC_MESSAGES`, cámbiele el nombre temporalmente para que ya no vuelva a introducir esos valores. Mantenga una copia de seguridad en lugar de eliminarlo:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
Aplica el idioma nuevamente en **Centro de Control → Idioma y Región**. El Panel de Control se reinicia automáticamente; si otros componentes aún muestran el idioma anterior, cierra sesión y vuelve a iniciarla para que la sesión gráfica y `systemd --user` reciban el nuevo entorno. Repite los comandos de diagnóstico después. Consulta [Variables de entorno](/docs/reference/environment-variables/) para los contratos técnicos y la precedencia.
