---
title: Instalar o ARGVUS
description: Instale, atualize, remova e configure o ARGVUS.
slug: pt/0.4.0/docs/user-guide/installation/install
---

O ARGVUS é distribuído como um pacote Arch Linux pelo repositório oficial de pacotes do ARGVUS.

## Instalação rápida

Em uma instalação nova do Arch Linux, você pode instalar o ARGVUS diretamente de um TTY, sem instalar outro ambiente desktop antes.

Execute:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

O instalador:

1. Importa e assina localmente a chave do repositório ARGVUS.
2. Configura o repositório de pacotes do ARGVUS.
3. Atualiza o banco de dados de pacotes.
4. Instala o pacote `argvus` e suas dependências.

Depois da instalação, saia da sessão e selecione **ARGVUS** no seu gerenciador de login.

> **Nota:** o instalador rápido requer `curl`, `sudo` e as ferramentas padrão de gerenciamento de pacotes do Arch Linux.

## Configurar somente o repositório

Para configurar o repositório de pacotes do ARGVUS sem instalar o desktop, execute:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

Depois, instale o ARGVUS manualmente:

```sh
sudo pacman -Syu argvus
```

## Instalação manual

Se preferir configurar o repositório passo a passo, use os comandos abaixo.

### 1. Baixe a chave de assinatura do repositório

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. Importe a chave de assinatura

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. Assine a chave localmente

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. Configure o repositório ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

Adicione a configuração do repositório a `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

A configuração habilita os repositórios do ARGVUS, incluindo `argvus` e `argvus-extras`, com `SigLevel = Required TrustedOnly`. O Pacman aceita pacotes somente quando suas assinaturas são válidas e confiáveis pelo chaveiro local.

### 5. Instale o ARGVUS

```sh
sudo pacman -Syu argvus
```

Depois da instalação, saia da sessão e selecione **ARGVUS** no seu gerenciador de login.

## Atualização

Atualize o ARGVUS junto com o restante do Arch Linux:

```sh
sudo pacman -Syu
```

Para atualizar somente o pacote ARGVUS:

```sh
sudo pacman -S argvus
```

> **Recomendação:** no Arch Linux, geralmente é recomendável fazer uma atualização completa com `sudo pacman -Syu` em vez de atualizar pacotes individuais.

## Remover o ARGVUS

Remova o ARGVUS mantendo as dependências que ainda são necessárias para outros pacotes instalados:

```sh
sudo pacman -R argvus
```

Remova o ARGVUS e as dependências que não são mais necessárias para nenhum pacote instalado:

```sh
sudo pacman -Rns argvus
```

### Remover o repositório ARGVUS

Se também quiser remover o repositório de pacotes, remova sua configuração:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

Depois, remova a linha correspondente de `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```

Você pode editar o arquivo com:

```sh
sudo nano /etc/pacman.conf
```

### Remover a chave de assinatura

Para remover a chave do repositório ARGVUS do chaveiro do pacman, liste primeiro as chaves:

```sh
sudo pacman-key --list
```

Depois, remova a chave do ARGVUS usando seu ID:

```sh
sudo pacman-key --delete KEY_ID
```

Substitua `KEY_ID` pelo ID da chave de assinatura do ARGVUS.

> **Atenção:** remova essa chave somente se também estiver removendo o repositório ARGVUS. Não remova chaves que ainda sejam necessárias para outros repositórios.

## Configuração do usuário

A instalação do pacote não grava configurações em `$HOME`. O ARGVUS lê os padrões empacotados de `/usr/share/argvus` e funciona sem copiá-los para sua pasta pessoal.

A configuração de runtime segue esta prioridade geral:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  padrões do aplicativo
```

O `argvus-config` é o único componente que escreve em `~/.config/argvus/data/generated/`, e essa projeção é a camada ativa: uma cópia gerenciada ou nativa só entra em vigor enquanto o arquivo gerado correspondente estiver ausente.

A configuração gerenciada pelo ARGVUS fica em `~/.config/argvus/`. Um override opcional em `$XDG_CONFIG_HOME/<app>` continua disponível para aplicativos que oferecem configuração nativa do usuário.

O dispatcher `argvus --setup` é opcional e serve para overrides manuais
explícitos, não para recovery. Se `~/.config/argvus` for apagado,
`SUPER + SHIFT + R` recria o perfil canônico e o estado de runtime gerado pelo
fluxo normal de reload.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all` é uma allowlist explícita dos arquivos de perfil TUI suportados; não
clona `/usr/share/argvus` e não copia scripts, serviços, assets, arquivos
gerados ou `config.json`. Execute `argvus --setup --help` para consultar as
opções atuais. Depois de copiados, os arquivos passam a ser overrides
pertencentes ao usuário e não são substituídos por atualizações do pacote.

Para substituir uma configuração já copiada, use `--force`. O ARGVUS cria um backup com timestamp antes de substituí-la:

```sh
argvus --setup --copy <app> --force
```

## Overrides do Hyprland

Para o guia completo de overrides do Hyprland, incluindo a diferença entre uma configuração nativa completa e os fragmentos incrementais do ARGVUS, veja [Overrides do Hyprland](/docs/argvus-hyprland/user-guide/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### Fallback para máquinas virtuais

Algumas máquinas virtuais podem apresentar problemas de renderização com Hyprland, Kitty ou Quickshell. O ARGVUS detecta máquinas virtuais que usam o driver gráfico `vmwgfx` e habilita renderização por software quando necessário.

Se a detecção não funcionar na sua máquina virtual, habilite o fallback manualmente em `user.lua`:

```lua
-- Compatibilidade com máquina virtual
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Depois de criar ou modificar `user.lua`, saia do Hyprland e entre novamente para que a variável seja aplicada aos aplicativos iniciados pela sessão.

Verifique com:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

A saída esperada é:

```text
1
```

> **Nota:** `LIBGL_ALWAYS_SOFTWARE=1` força aplicativos Mesa/OpenGL a usar renderização por software, normalmente através do `llvmpipe`. Isso pode reduzir o desempenho gráfico e deve ser usado como fallback para máquinas virtuais afetadas.
