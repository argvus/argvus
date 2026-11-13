---
title: Localización
description: Usa ARGVUS en inglés o portugués.
---

`argvus-i18n` proporciona los catálogos compartidos, el `argvus i18n validate` el comando y el QML módulo `org.argvus.i18n`. ARGVUS barcos 18 idiomas de interfaz: árabe, bengalí, alemán, inglés, español, francés, hindi, indonesio, italiano, japonés, coreano, portugués brasileño, portugués europeo, ruso, turco, urdu, vietnamita y chino simplificado.

Native ARGVUS aplicaciones y componentes de concha consumen los catálogos compartidos. Por lo tanto, una cadena traducida puede ser utilizada por más de un repositorio. árabe y urdu utilizan el diseño derecho a izquierda en el soporte GTK, Quickshell e interfaces terminales.

Use **Control Center → Preferencias → Idioma & Region → Idioma** para seleccionar el ARGVUS lenguaje de interfaz. Esto escribe `ARGVUS_LANGUAGE` a `$XDG_CONFIG_HOME/locale.conf` (normalmente `~/.config/locale.conf`). Por ejemplo, `ARGVUS_LANGUAGE=pt-PT` selecciona Portugués Europeo. Portugués brasileño`pt-BR`) es una opción separada. El catálogo de idiomas está instalado con ARGVUS, así que no depende de `locale-gen`.

El Panel de Control se reinicia automáticamente después de que el lenguaje de interfaz cambie. Iniciar una nueva sesión para aplicarla a los componentes del período de sesiones. La configuración separada **Regional Locale** escribe `LANG` y controla formatos regionales y aplicaciones de terceros. Requiere un sistema generado locale; permitiendo uno en `/etc/locale.gen` carreras `locale-gen` con autorización de administrador. El diseño de teclado, la variante y el teclado de consola son ajustes separados.

## When environment variables override the change

ARGVUS Solicitudes `ARGVUS_LANGUAGE` del usuario `locale.conf`. Si está ausente, resuelven la región `LANG` valor a un catálogo instalado. Las variables locales heredadas no anulan un explícito `ARGVUS_LANGUAGE` preferencia. Las aplicaciones de terceros siguen utilizando el sistema local regional.

Inspeccione los valores efectivos con:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Mira eso. `~/.config/locale.conf` contiene un válido `ARGVUS_LANGUAGE=<catalog>` valor. Para cambiar el formato regional o el lenguaje de aplicación de terceros, utilice **Regional Locale** y seleccione un local generado. El Panel de Control se reinicia después de los cambios de preferencia; inicie sesión y en si otros componentes de sesión todavía muestran el lenguaje de interfaz antiguo. Véase [Las variables de entorno](/docs/reference/environment-variables/) para los contratos técnicos y la precedencia.
