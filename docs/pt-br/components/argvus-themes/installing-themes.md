---
title: Instalando temas
description: Instale os pacotes de tema oficiais do ARGVUS com pacman.
---

Os temas oficiais são publicados como pacotes do Arch Linux. Cada tema é um pacote próprio chamado `argvus-theme-<NOME_DO_TEMA>`.

## De onde obter os temas

A origem de cada tema oficial fica na organização de projetos do ARGVUS:

- Cada tema é um repositório chamado `argvus-theme-<NOME_DO_TEMA>` (por exemplo `argvus-theme-dracula`), empacotado com o mesmo nome.

Os pacotes de tema dependem de `argvus-appearance`. Instale o ARGVUS primeiro, conforme descrito em [Instalação](/pt/docs/getting-started/installation/), para que o componente de aparência esteja presente.

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

O pacote instala o manifesto do tema, `/usr/share/argvus/themes/<id-do-tema>/theme.toml` (nome, categoria, wallpaper e paleta, com `overrides/` opcionais), e o wallpaper do tema em `/usr/share/backgrounds/argvus/`. Os arquivos de cada componente são renderizados a partir da paleta quando o tema é aplicado. Ele não grava no seu diretório pessoal e não altera o tema atual. Selecione-o depois pelo Control Center, veja [Temas oficiais](/pt/docs/components/argvus-themes/official-themes/).

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
| `argvus-theme-watercolor` | Aquarela |

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
`~/.config/argvus/data/backups/theme-copies-<hora>/`; para manter um arquivo que
você editou, coloque-o num tema seu em `overrides/` e indique-o na tabela
`[overrides]` do manifesto.

## Remover um tema

Remova um pacote de tema com:

```bash
sudo pacman -R argvus-theme-<NOME_DO_TEMA>
```

Remover um pacote de tema não apaga seus perfis importados. Se o tema removido for o que está aplicado, selecione outro tema pelo Control Center antes de remover.

## Instalar temas da comunidade por Git

Instale o repositório criado com o skeleton por HTTPS ou SSH. O Git usa suas credenciais existentes. A instalação é por usuário e não ativa o tema. Use `--ref` para escolher uma branch ou tag e `--replace` para substituir um ID personalizado existente. IDs oficiais instalados são protegidos.

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

O tema aparece em **Custom themes** no Control Center; pressione `r` para atualizar a página aberta. Aplique outro tema antes de remover o ativo, tanto no CLI quanto no Control Center. Após substituir o tema ativo, aplique-o novamente.

O repositório deve conter exatamente um tema schema 2 em `src/usr/share/argvus/themes/<id>/`, com o mesmo ID no manifesto e no diretório. Wallpaper e preview devem ser arquivos versionados: relativos ao tema ou incluídos em `src/usr/share/`. Scripts do repositório não são executados. Submódulos e Git LFS não são suportados. Instale `git` e, para SSH, `openssh`. O destino é `$XDG_DATA_HOME/argvus/themes/<id>`. Pacotes oficiais continuam gerenciados pelo pacman.
