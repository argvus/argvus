---
title: Instalação
description: Instale e atualize o ARGVUS no Arch Linux.
---

O ARGVUS é distribuído como um [pacote Arch](https://wiki.archlinux.org/title/Pacman) no repositório oficial do projeto. Nenhum AUR helper é necessário.

## Adicionar o repositório

O repositório de pacotes do ARGVUS é publicado no repositório público [argvus/packages](https://github.com/argvus/packages), servido em `https://argvus.github.io/packages/`.

Importe e assine localmente a chave pública do ARGVUS:

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
sudo pacman-key --add /tmp/argvus.gpg
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

Adicione o seguinte ao seu `/etc/pacman.conf`:

```ini
[argvus]
SigLevel = Required
Server = https://argvus.github.io/packages/arch/$arch
```

Depois sincronize os bancos:

```sh
sudo pacman -Syu
```

## Instalar

O pacote `argvus` depende dos pacotes vendored `pwvucontrol`, `snappy-switcher` e `wlogout` e dos pacotes de componentes oficiais `argvus-appearance`, `argvus-calendar`, `argvus-greeter`, `argvus-session` e `argvus-storage`, todos publicados no mesmo repositório.

```sh
sudo pacman -Syu argvus
```

Depois da instalação, consulte a [seção de sessões](../sessions/gdm/) para iniciar o desktop.

## Remover

```sh
sudo pacman -Rns argvus
```

## Configuração do usuário

A instalação do pacote nunca escreve em `$HOME`. O ARGVUS funciona diretamente ao ler os padrões empacotados de `/usr/share/argvus`.

A configuração de runtime usa esta prioridade:

```text
$XDG_CONFIG_HOME/<app>  ->  $XDG_STATE_HOME/argvus/config/<app>  ->  /usr/share/argvus/<app>  ->  defaults upstream
```

Se `XDG_STATE_HOME` não estiver definido, o fallback é `~/.local/state`. As ferramentas de tema, acento e espaçamento geram configuração de runtime de aplicações em `~/.local/state/argvus/config` em vez de copiar árvores completas de configuração de aplicações para `$HOME`.

Preferências pequenas do ARGVUS ficam em `$XDG_CONFIG_HOME/argvus`, e overrides explícitos e completos de aplicações ficam em `$XDG_CONFIG_HOME/<app>`.

`argvus-setup` é opcional. Use-o apenas quando quiser copiar os padrões empacotados para `$XDG_CONFIG_HOME` para personalizar:

```sh
argvus-setup --copy hypr
argvus-setup --copy waybar
argvus-setup --copy-all
```

Depois de copiados, os arquivos viram overrides do usuário e upgrades de pacote nunca os substituem.

Para atualizar uma configuração existente com os padrões atuais do pacote:

```sh
argvus-setup --copy <app> --force
```

O modo `--force` faz backup do diretório atual como `$XDG_CONFIG_HOME/<app>.bak-<timestamp>` antes de copiar os novos padrões.
