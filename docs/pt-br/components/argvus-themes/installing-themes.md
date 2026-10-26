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

Não há um metapacote que instale todos os temas. Instale os temas que quiser em um único comando:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Pacotes disponíveis

Os pacotes abaixo são os temas oficiais publicados hoje.

**Temas escuros**

| Pacote | Tema |
| --- | --- |
| `argvus-theme-one-dark` | One Dark |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silver Dark |
| `argvus-theme-slate-dark` | Slate Dark |
| `argvus-theme-universe` | Universe |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé Pine |
| `argvus-theme-tokyo-night` | Tokyo Night |
| `argvus-theme-solitude` | Solitude |
| `argvus-theme-sunset` | Sunset |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**Temas claros**

| Pacote | Tema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Light |
| `argvus-theme-solarized-light` | Solarized Light |
| `argvus-theme-nord-light` | Nord Light |
| `argvus-theme-one-light` | One Light |
| `argvus-theme-everforest-light` | Everforest Light |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Light |
| `argvus-theme-watercolor` | Watercolor |

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
