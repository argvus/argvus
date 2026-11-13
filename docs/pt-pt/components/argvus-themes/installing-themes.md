---
title: Instalando temas
description: Instale os pacotes de tema oficiais do ARGVUS com pacman.
---

Os temas oficiais são publicados como pacotes do Arch Linux. Cada tema é um pacote próprio chamado `argvus-theme-<NOME_DO_TEMA>`.

## De onde obter os temas

A origem de cada tema oficial fica na organização de projetos do ARGVUS:

- Cada tema é um repositório chamado `argvus-theme-<NOME_DO_TEMA>` (por exemplo `argvus-theme-dracula`), empacotado com o mesmo nome.

Os pacotes de tema dependem de `argvus-appearance`. Instale o ARGVUS primeiro, conforme descrito em [Instalação](/pt/docs/getting-started/installation/), para que o componente de aparência esteja presente.


A instalação por Git lê os ficheiros do tema apenas em `src/`. A pasta `packaging/` é opcional para instalar e serve para criar pacotes Arch Linux.

## Instalar um tema

Instale um tema pelo nome do pacote:

```bash
sudo pacman -S argvus-theme-<NOME_DO_TEMA>
```

Por exemplo:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

O pacote instala o manifesto do tema, `/usr/share/argvus/themes/<id-do-tema>/theme.toml` (nome, categoria, wallpaper e paleta, com `overrides/` opcionais), e o wallpaper do tema em `/usr/share/backgrounds/argvus/`. Os ficheiros de cada componente são renderizados a partir da paleta quando o tema é aplicado. Ele não grava no seu diretório pessoal e não altera o tema atual. Selecione-o depois pelo Control Center, veja [Temas oficiais](/pt/docs/components/argvus-themes/official-themes/).

## Instalar vários temas

Para instalar todos os temas oficiais de uma vez, instale o metapacote `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```

Ele depende de todos os pacotes publicados `argvus-theme-*`. ARGVUS Dark e ARGVUS
Light já vêm em `argvus-appearance`.

Para instalar somente alguns temas, use os pacotes individuais:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Pacotes disponíveis

Os pacotes abaixo são os temas oficiais publicados hoje.

**Temas escuros**

| Pacote | Tema |
| --- | --- |
| `argvus-theme-one-dark` | Um Escuro |
| `argvus-theme-dracula` | Drácula |
| `argvus-theme-silver-dark` | Prata Escuro |
| `argvus-theme-slate-dark` | Ardósia Escura |
| `argvus-theme-universe` | Universo |
| `argvus-theme-gruvbox-high-dark` | Gruvbox alto escuro |
| `argvus-theme-gruvbox-dark` | Gruvbox Escuro |
| `argvus-theme-rose-pine` | Pinho Rosé |
| `argvus-theme-tokyo-night` | Noite de Tóquio |
| `argvus-theme-solitude` | Solidão |
| `argvus-theme-sunset` | Pôr do sol |
| `argvus-theme-matrix` | Matriz |
| `argvus-theme-monokai-dark` | Monokai Sombrio |

**Temas claros**

| Pacote | Tema |
| --- | --- |
| `argvus-theme-github-light` | Luz GitHub |
| `argvus-theme-solarized-light` | Luz Solarizada |
| `argvus-theme-nord-light` | Luz Norte |
| `argvus-theme-one-light` | Uma Luz |
| `argvus-theme-everforest-light` | Luz da Floresta Ever |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Luz Gruvbox |
| `argvus-theme-watercolor` | Aguarela |

Os temas nativos **ARGVUS Dark** e **ARGVUS Light** não precisam de pacote separado; eles vêm com `argvus-appearance`.

## Verificar o que está instalado

Liste os pacotes de tema instalados com:

```bash
pacman -Qs argvus-theme
```

Abra o Control Center e vá até **Aparência → Temas → Temas oficiais** para ver os temas disponíveis no seletor. Um tema instalado que não aparece nessa lista indica um problema de empacotamento; veja a seção de solução de problemas do [guia de temas](/pt/docs/components/argvus-themes/).

## Adicionar um tema seu

Um tema é um manifesto, então um tema seu não precisa de pacote. Crie
`~/.local/share/argvus/themes/<id>/theme.toml`, em que o nome do diretório é o
ID do tema (letras minúsculas, dígitos e hífens simples):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

Só `bg`, `fg` e `accent` são obrigatórios: todas as outras cores têm um padrão
derivado delas, e o manifesto define uma só para mudá-la. Confira o tema e
aplique-o como qualquer outro:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

O `argvus theme list` mostra seus temas ao lado dos oficiais (um tema oficial
com o mesmo ID vence). Seus temas aparecem no menu de temas (`SUPER + Shift + T`);
o greeter, que roda antes do login, não consegue ler o seu diretório pessoal,
então continua com o último tema oficial que você aplicou enquanto um tema seu está ativo.

As cópias por tema que versões antigas do ARGVUS gravavam em
`~/.config/argvus/data/<app>/themes/` não são mais lidas. A primeira troca de
tema depois da atualização as move para
`~/.config/argvus/data/backups/theme-copies-<hora>/`; para manter um ficheiro que
você editou, coloque-o num tema seu em `overrides/` e indique-o na tabela
`[overrides]` do manifesto.

## Remover um tema

Remova um pacote de tema com:

```bash
sudo pacman -R argvus-theme-<NOME_DO_TEMA>
```

Remover um pacote de tema não apaga seus perfis importados. Se o tema removido for o que está aplicado, selecione outro tema pelo Control Center antes de remover.

## Criar um tema com o skeleton

Comece pelo repositório `argvus-skeleton-theme` para criar um tema compatível. Execute `make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark` e edite a paleta e o papel de parede. Valide com `make check` e experimente com `make try`. Depois de publicar o repositório, instale com `argvus theme install URL`. Os ficheiros do tema ficam em `src/`; `packaging/` serve apenas para criar pacotes Arch.

```sh
git clone https://github.com/argvus/argvus-skeleton-theme.git argvus-theme-ocean-night
cd argvus-theme-ocean-night
make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark
make check
make try
argvus theme apply ocean-night
```

## Instalar temas da comunidade por Git

Instale o repositório do skeleton por HTTPS ou SSH com as credenciais do Git. A instalação é por utilizador e não ativa o tema. `--ref` seleciona um ramo ou etiqueta; `--replace` substitui um tema personalizado existente. Os IDs oficiais instalados estão protegidos.

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

O tema aparece em **Custom themes**; `r` atualiza a página aberta. Aplique outro tema antes de remover o ativo no CLI ou Control Center. Volte a aplicar o tema após o substituir.

É necessário um único tema schema 2 em `src/usr/share/argvus/themes/<id>/`, com ID correspondente. Inclua as imagens no tema ou em `src/usr/share/`. Os scripts não são executados. Submódulos e Git LFS não são suportados. Instale `git` e `openssh` para SSH. Destino: `$XDG_DATA_HOME/argvus/themes/<id>`. Os pacotes oficiais usam pacman.
