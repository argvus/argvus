---
title: Variables de entorno
description: Contratos de entorno usados por los componentes de la sesión de ARGVUS.
---
La sesión establece la integración del entorno XDG y Wayland y puede exportar variables de compatibilidad con la virtualización cuando se detecten. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` y `ARGVUS_BOOTSTRAP` son utilizados por los asistentes de sesión y despachadores para resolver raíces de configuración.

Trate las variables de entorno como contratos de ejecución: verifique el `--help` del asistente instalado o la fuente antes de agregar una anulación persistente.

El estado derivado vive bajo la raíz de estado en lugar de la raíz de configuración. `XDG_STATE_HOME` (normalmente `~/.local/state`) recibe `argvus/config-projection.json`, el manifiesto de proyección `argvus-config` se reconstruye y `argvus sessionctl reload` se aplica, además de `argvus/session.log`. Ninguno es una fuente de verdad.

## Entorno de idioma

La única preferencia de idioma general es `LANG` en `$XDG_CONFIG_HOME/locale.conf` (predeterminado `~/.config/locale.conf`). Sin un valor válido, ARGVUS selecciona inglés (`en-US`), incluso cuando `/etc/locale.conf`, PAM o el gestor de inicio de sesión suministran portugués. Valores como `pt_BR.UTF-8` se normalizan a `pt-BR` para los catálogos.

La sesión aplica esta preferencia antes de iniciar los componentes del escritorio e importar los entornos de activación de usuario de systemd y D-Bus. Reemplaza `LANG`, `LC_MESSAGES`, `LANGUAGE` heredados y borra `LC_ALL`; se preservan los ajustes regionales `LC_*` en el mismo archivo. El inglés utiliza `en_US.UTF-8` cuando se genera, de lo contrario `C.UTF-8`. No se crea automáticamente un archivo de preferencia. Cierra sesión y vuelve a iniciarla después de cambiar el idioma.

`session.language`, `argvus/data/language` y `argvus/data/internal/language` son datos heredados ignorados. Ni se importan ni se regeneran; los archivos existentes se preservan. `argvus config set` y `patch` rechazan nuevas escrituras en `session.language` y dirigen a los que llaman directamente a `locale.conf`.

Para el diagnóstico, compare `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` y `systemctl --user show-environment`. `localectl status` describe el sistema, no el idioma ARGVUS del usuario. `ARGVUS_I18N_DIR` selecciona un directorio de catálogo para desarrollo, no el idioma.
