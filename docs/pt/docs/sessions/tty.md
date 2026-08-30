---
title: Sessões TTY
description: Inicie o ARGVUS a partir de um TTY.
---

O ARGVUS pode ser iniciado diretamente de um TTY sem GDM ou qualquer outro display manager. O pacote não inicia o Hyprland automaticamente após o login no TTY; o usuário decide quando iniciar a sessão.

## Como funciona

Depois de logar no TTY com seu usuário e senha, execute:

```sh
argvus-tty
```

O `argvus-tty` é fornecido pelo pacote `argvus-session` e faz tudo o que um display manager faria antes de iniciar o Hyprland:

- Registra a sessão no logind (`loginctl open-session`)
- Inicia o session bus do D-Bus
- Define as variáveis de ambiente (XDG, Qt, Electron, Wayland)
- Inicia a sessão ARGVUS (`argvus-session`)

Ao sair, o script limpa tudo (D-Bus, logind) e retorna ao TTY.

## Setup

Para usar sem display manager, desabilite o GDM se estiver ativo e reinicie:

```sh
sudo systemctl disable gdm
sudo reboot
```

Loge com seu usuário e senha na tela de login do TTY. Depois inicie o ambiente manualmente:

```sh
argvus-tty
```

Para diagnóstico:

```sh
argvus-tty --status
```

## Auto-start opcional

O pacote não configura autologin no TTY e não inicia o Hyprland automaticamente por padrão. Se quiser que o ambiente inicie automaticamente depois de logar manualmente no TTY, adicione sua própria regra em `~/.bash_profile` ou `~/.zprofile`:

```sh
# ~/.bash_profile
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ "${XDG_VTNR:-}" = "1" ]; then
  exec argvus-tty
fi
```

Existe também um profile opt-in empacotado em `/usr/share/argvus/argvus/profile`. Copie-o para `$XDG_CONFIG_HOME/argvus/profile` com `argvus-setup --copy argvus` e carregue-o:

```sh
# ~/.bash_profile
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/profile" ] && . "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/profile"
```

O profile inicia o `argvus-tty` automaticamente no VT configurado (padrão `1`) quando nenhum display manager está rodando. Para desabilitar, exporte `ARGVUS_TTY_DISABLE=1`; para usar outro VT, exporte `ARGVUS_TTY_VT=2`. Ele também pula o auto-start quando o greetd controla o fluxo de login.

## Logs

Os logs da sessão TTY ficam em:

```sh
cat ~/.local/state/argvus/tty.log
cat ~/.local/state/argvus/session.log
```
