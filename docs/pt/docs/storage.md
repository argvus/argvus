---
title: Armazenamento Removível
description: O módulo de armazenamento removível em Rust para a Waybar.
---

O pacote `argvus-storage` entrega um programa em Rust que observa armazenamento removível via UDisks2 (D-Bus, sem polling) e alimenta um módulo compacto da Waybar.

## Uso

O módulo já vem pré-configurado no grupo de gravação/microfone/áudio. A barra mostra um único ícone Font Awesome quando existem dispositivos removíveis; sem dispositivos o módulo se oculta. O tray da Waybar fica reservado para aplicativos que publicam ícones de tray, como Telegram e Steam.

| Botão | Ação |
|---|---|
| Esquerdo | Escolher um dispositivo e abri-lo no gerenciador de arquivos |
| Direito | Menu de contexto (Open/Mount/Unmount/Eject/Power Off/Unlock/Lock/Copy) |

## Comandos

```sh
argvus-storage once      # imprime uma linha JSON e sai
argvus-storage list      # lista volumes removíveis
argvus-storage devices   # escolhe um dispositivo e abre no gerenciador de arquivos
argvus-storage menu      # menu de contexto (rofi/wofi/dmenu)
argvus-storage unmount   # ações: open mount unmount eject poweroff lock unlock copy
```

## Configuração

Os defaults ficam em `/etc/argvus-storage/config.json` e `/etc/argvus-storage/theme.css`. Para personalizar por usuário, copie-os para `$XDG_CONFIG_HOME/argvus-storage/`. Opções: `show_name`, `show_capacity`, `hide_when_empty`, `show_hidden`, `max_devices`, `sort` (`mount_time` | `insertion` | `name` | `size`), `separator`, `format`, `tooltip_format`, `file_manager_command`, `open_command` (alias de compatibilidade), `copy_command`, `unlock_command`, `menu` (`rofi` | `wofi` | `dmenu`), `menu_flags` e `icons`.

Para volumes LUKS, `unlock_command` (padrão `kitty -e`) abre um terminal para a senha. Dependências de runtime: `udisks2`, `glib2`, `rofi`/`wofi`/`dmenu`, `wl-clipboard` e `libnotify`.

## Temas

O pacote do sistema é dono de:

```text
/etc/argvus-storage/theme.css
/etc/argvus-storage/themes/argvus-dark-aether.css
/etc/argvus-storage/themes/argvus-dark-silver.css
/etc/argvus-storage/themes/argvus-light-veil.css
/etc/argvus-storage/themes/argvus-dark-slate.css
```

Quando o usuário muda o tema do desktop, o ARGVUS prepara o tema correspondente do storage na configuração de runtime gerada. Arquivos do usuário em `$XDG_CONFIG_HOME/argvus-storage/` continuam tendo prioridade sobre os defaults do sistema.

## Integração com o ARGVUS

O módulo da Waybar é configurado no `argvus` como `custom/storage`:

```json
{
  "exec": "argvus-storage watch",
  "return-type": "json",
  "format": "{}",
  "tooltip": true,
  "hide-empty-text": true,
  "on-click": "/usr/share/argvus/hypr/scripts/storage-menu.sh"
}
```

`storage-menu.sh` ancora o menu GTK na posição do ponteiro quando possível. O pacote `argvus` depende de `argvus-storage`, mas o pacote de storage é dono do binário, configuração, temas e licença.
