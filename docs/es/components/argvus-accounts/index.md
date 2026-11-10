---
title: Cuentas
description: Perfiles de usuario locales, nombres para mostrar, avatares y membresía en grupos, a través de cuentas argvus.
---
`argvus accounts` lista y edita las cuentas de usuario locales del escritorio ARGVUS. Es un ejecutable privado del paquete `argvus`, instalado en `/usr/lib/argvus/commands/accounts`. El comando `argvus` lo inicia, y solo las operaciones que cambian otras cuentas o datos del sistema se ejecutan como root.

## Comandos

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```
## Nombres para mostrar y avatares

El nombre para mostrar se almacena en el campo estándar GECOS. El nombre de usuario de Unix nunca se cambia.

Un avatar es una imagen en formato PNG, JPEG o WebP, de hasta 20 MiB. La imagen se redimensiona a 256×256 píxeles y se guarda como PNG con el modo `0644` en `~/.face`, la ubicación freedesktop leída por el greeter y por la mayoría de los gestores de pantalla. Tu propio avatar se escribe dentro de tu directorio personal y no requiere autorización.

## Autorización

Solo `list`, `show`, `self`, `self groups` y tu propio avatar funcionan sin autorización. Cualquier otro cambio solicita autorización al agente polkit del escritorio. La regla ARGVUS (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) decide quién debe autenticarse:

- un usuario en un grupo de administradores (`wheel`, el predeterminado de Arch, o `sudo`) confirma con su propia contraseña;
- cualquier otro usuario confirma con la contraseña de root.

| Operación | Sin autorización | Con autorización |
| --- | --- | --- |
| Listar usuarios, mostrar una cuenta, listar grupos | ✓ | ✓ |
| Cambiar tu propio avatar | ✓ | ✓ |
| Cambiar tu propio nombre para mostrar | — | ✓ |
| Cambiar tu propia contraseña | — | ✓ (después de comprobar la contraseña actual) |
| Cambiar el nombre, avatar, grupos o contraseña de otro usuario | — | ✓ |

Cambiar tu propia contraseña primero verifica la contraseña actual. Una contraseña incorrecta se rechaza antes de que aparezca el aviso de autorización.

No hay necesidad de ejecutar el comando con `sudo`. El comando solicita autorización a polkit cuando la necesita.

## Límites de seguridad

- `passwd` lee la contraseña actual, la nueva y la de confirmación desde la entrada estándar, una por línea, y nunca las toma como argumentos. La línea de la contraseña actual puede estar vacía cuando un administrador restablece la contraseña de otro usuario. Ejemplo:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  De manera interactiva, escriba las tres líneas y presione Enter después de cada una. La entrada se muestra en el
  terminal mientras se escribe.
- Editar una cuenta con UID inferior a 1000 (y `nobody`) está prohibido, al igual que bloquearla, eliminarla
  o expirar su contraseña. Su propia cuenta no puede ser bloqueada ni eliminada.
- Renombrar o eliminar un grupo con GID inferior a 1000, o un grupo de administradores (`wheel`, `sudo`), está
  prohibido; sus miembros aún pueden cambiarse, excepto que no puede eliminarse a sí mismo de un
  grupo de administradores.
- La pertenencia a administradores es el grupo `wheel` o `sudo`. El Centro de Control añade nuevos
  administradores a `wheel` cuando el sistema lo tiene, y a `sudo` en caso contrario. Quitar su propia
  pertenencia de administrador mediante una edición está prohibido.

## Ejemplos

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```
## Solución de problemas

- `pkexec was not found`: instale `polkit`. El paquete `argvus` depende de él.
- El aviso de autorización no aparece: verifique que un agente de autenticación polkit esté ejecutándose en su sesión. El paquete `argvus` depende de `hyprpolkitagent`, que proporciona uno.
- Errores de `unix_chkpwd` al cambiar una contraseña: el paquete `pam` proporciona el ayudante.
- Ejecute `argvus accounts --verbose ...` para imprimir los pasos del comando en stderr.

Para notas de desarrollador sobre el límite de privilegios, consulte `docs/internal/argvus-accounts/`.

Cuando un administrador bloquea o elimina la cuenta configurada para inicio de sesión automático, el proveedor de la cuenta primero desactiva su entrada de inicio de sesión automático en greetd. Si esa limpieza falla, se rechaza la operación de la cuenta. Si el bloqueo o la eliminación fallan más tarde, el inicio de sesión automático permanece desactivado. El inicio de sesión automático para otra cuenta se conserva; desbloquear una cuenta no habilita el inicio de sesión automático. Las sesiones existentes no se cierran y greetd no se reinicia.
