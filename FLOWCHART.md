# FLOWCHART — Como o ARGVUS funciona

Este documento explica o fluxo do ARGVUS: onde as configurações moram, como o
desktop inicia, o que acontece no primeiro login e como o usuário altera temas,
cor de destaque, espaçamentos e outros comportamentos.

> **Resumo em uma frase:** o ARGVUS não copia nada por padrão — o desktop lê
> da base empacotada em `/usr/share/argvus`, e `~/.config/argvus` guarda apenas
> as alterações e o estado do usuário, materializados sob demanda.

---

## 1. Os locais de configuração

| Local | Papel | Quem escreve |
| --- | --- | --- |
| `/usr/share/argvus` | Defaults empacotados (imutáveis). Fonte base de leitura. | pacman (upgrade) |
| `~/.config/argvus/<app>/` | Config de cada app tocada pelo usuário/runtime (waybar, hypr, quickshell, kitty...). | `theme-switch`, `accent-switch`, `argvus --setup` |
| `~/.config/argvus/generated/` | Config de runtime reconstruída a partir dos defaults + preferências. | runtime |
| `~/.config/argvus/state/` | Estado do usuário (antigo `~/.local/state/argvus`). | runtime |
| `~/.config/argvus/` | Preferências pequenas: `.active-theme`, `.accent-color`, `.spaces`, `defaults.json`. | runtime |
| `~/.config/<app>/` | **Override livre** do usuário para um app específico (fica intocado por padrão). | usuário (manual) |
| `~/.cache/argvus/` | Cache volátil (wallpaper do Hyprlock, logs, daemon state). | runtime |

### Cadeia de precedência (leitura)

```text
~/.config/<app>  ->  ~/.config/argvus/<app>  ->  ~/.config/argvus/generated/<app>  ->  /usr/share/argvus/<app>
```

A configuração é resolvida do mais específico para o mais genérico: se o
usuário tem um arquivo em `~/.config/<app>`, ele vence como override manual;
senão, usa a camada Argvus em `~/.config/argvus/<app>`; senão, o gerado; e por
fim o empacotado.

### Por que não é uma cópia

O ARGVUS **não** faz `cp -R /usr/share/argvus ~/.config/argvus` na instalação.
O `~/.config/argvus` só passa a existir quando algo precisa ser gravado ou
materializado (tema, accent, etc.). Quem nunca precisa ser alterado é lido
diretamente de `/usr/share/argvus`.

---

## 2. O fluxo do inicialização (boot)

```text
Login (tty/gdm)
   │
   ▼
argvus-session  ──prepara o ambiente──►  XDG_CONFIG_DIRS=/usr/share/argvus:/etc/xdg
   │
   ▼
start-hyprland (Hyprland)
   │
   ▼
hyprland.lua
   │  • resolve tema/accent/spacing via cadeia de precedência
   │  • registra binds (teclas)
   ▼
hypr-init.sh --started   (primeira ação do DE)
   │
   ├──► 1º login? ──NÃO──► apenas lê o estado existente
   │          │SIM
   │          ▼
   │   apply default theme (theme-switch) ──materializa configs mutáveis
   │          │
   │          ▼
   ├── set_gsettings  (GTK/ícone/cursor)
   ├── start_wallpaper (hyprpaper/swaybg)
   ├── run_hypridle ── hypridle -c <hypridle.conf>
   ├── apply spaces + monitor layout
   ├── run_waybars (waybar + sysinfo bar)
   └── quickshell sidebar + daemons (cliphist, keyboard-layout, dunst, ...)
```

### Ponto-chave

- **Sem `~/.config/argvus`?** O DE ainda inicia normalmente, lendo tudo de
  `/usr/share/argvus`.
- **Primeiro login:** o `hypr-init.sh --started` detecta que não há
  `~/.config/argvus/.active-theme` e aplica o tema padrão empacotado
  (`ARGVUS_NO_RUNTIME=1 ... theme-switch.sh`), materializando sob demanda os
  arquivos que o tema precisa modificar.

---

## 3. O que são as "configs mutáveis" e por que elas são materializadas

Algumas configurações precisam ser **reescritas** em tempo de execução para
refletir o tema ativo, a cor de destaque ou os espaçamentos. Para reescrever um
arquivo com `sed -i`, o runtime precisa de uma cópia **real** editável.

Sempre que um script roda com `ARGVUS_MUTABLE_CONFIG=1` (por exemplo
`theme-switch.sh`, `accent-switch.sh`, `spaces-switch.sh`, `monitor-switch.sh`):

```text
paths_config("<app>/arquivo")
   │
   ▼ (MUTABLE_CONFIG=1)
o default é copiado de /usr/share/argvus/<app>/arquivo
      │
      ▼
  ~/.config/argvus/<app>/arquivo   (cópia do usuário)
      │
      ▼
  script edita a cópia (sed -i) e relança o serviço
```

Esse é o mecanismo que permite, ao pressionar `SUPER + SHIFT + A`, o accent
color se espalhar por GTK, terminal, rofi, Waybar e Hyprland **sem** que o
usuário precise editar nada manualmente.

> A cadeia de precedência existe justamente para isso: o runtime sempre lê a
> config mais específica, e escreve apenas na camada do usuário
> (`~/.config/argvus/`), nunca em `/usr/share/argvus`.

---

## 4. Tema, accent e espaçamento

### Tema (`SUPER + SHIFT + T`)

1. O tema ativo fica gravado em `~/.config/argvus/.active-theme`.
2. `theme-switch.sh <tema>` é chamado com `ARGVUS_MUTABLE_CONFIG=1`.
3. Ele reescreve as configs de app (waybar, rofi, kitty, papéis de parede,
   etc.) na camada do usuário e reinicia os serviços afetados.
4. Na próxima leitura, o `hyprland.lua` encontra o novo tema na precedência.

### Cor de destaque (`SUPER + SHIFT + A`)

1. A cor fica gravada em `~/.config/argvus/.accent-color`.
2. `accent-switch.sh` materializa e reescreve as paletas das apps
   (quickshell, waybar, rofi, gtk) e notifica o runtime para recarregar.

### Espaçamentos (`SUPER + SHIFT + T` → opção de espacamento / `.spaces`)

1. O valor fica em `~/.config/argvus/.spaces`.
2. `spaces-switch.sh --apply-static` ajusta as margens/bordas da waybar e do
   quickshell na camada do usuário.

---

## 5. Default apps (app padrão)

O aplicativo padrão por categoria (terminal, editor, navegador, etc.) é gravado
pelo `argvus-default-apps` em `~/.config/argvus/defaults.json`.

O `get-default.sh <categoria>` resolve com a mesma ordem de precedência:

```text
~/.config/argvus/defaults.json  ->  /usr/share/argvus/defaults.json
```

> Um `~/.local/state/argvus/defaults.json` legado, escrito por versões antigas,
> ainda é honrado durante a migração.

---

## 6. `argvus --setup` (opcional)

O `argvus --setup` **não é necessário** para o DE funcionar. Ele existe apenas
para quem quer **materializar as configs** para editar manualmente:

```sh
argvus --setup --copy <app>        # copia /usr/share/argvus/<app> para ~/.config/argvus/<app>
argvus --setup --copy-all          # copia todos os apps; o app `argvus` é mesclado na raiz
argvus --setup --copy-all --force  # substitui o existente com backup
```

Depois de copiado, o arquivo vira override do usuário e **upgrades de pacote
não o substituem**.

---

## 7. Exemplo realista de ciclo (1º boot até o usuário mudar o tema)

```text
1. Instalação:  /usr/share/argvus  <-- pacote (nada em ~/.config/argvus ainda)
2. 1º login:    hypr-init.sh --started
                    • .active-theme não existe → aplica tema padrão
                    • theme-switch materializa waybar/rofi/kitty/qt6ct na camada do usuário
                    • gsettings define GTK/ícone/cursor; wallpaper inicia; bars/sidebar sobem
3. Uso:         tudo lido de ~/.config/argvus/... (vence) ou /usr/share/argvus (base)
4. SUPER+SHIFT+A: accent-switch reescreve paletas em ~/.config/argvus e recarrega
5. Reboot:      hyprland.lua lê tema+accent+spacing da precedência → sessão igual
```

---

## 8. Regra de ouro

- **`/usr/share/argvus`** = somente leitura (base empacotada).
- **`~/.config/argvus`** = alterações e estado do usuário (tema atual, accent,
  espaçamento, defaults, configs materializadas).
- **`~/.config/<app>`** = override manual livre (opcional).
- **`~/.cache/argvus`** = descartável (logs, wallpaper, daemon state).
- Arquivos **`scripts/`** e **`docs/`** são sempre **somente leitura** na camada
  do usuário (não são materializados/sobrescritos pelo runtime).
- Apps TUI com busca de config própria (`superfile`, `btop`, `bottom`, `yazi`)
  devem ser abertos pelos subcomandos `argvus --spf`, `argvus --btop`,
  `argvus --btm` e `argvus --yazy`, que respeitam `~/.config/<app>` como
  override nativo.
- O Yazi usa somente flavors nativos em
  `config/yazi/flavors/<tema>.yazi/flavor.toml`. Não existe mais uma árvore
  paralela `config/yazi/themes`; a troca de tema grava `theme.toml` com o
  flavor Argvus ativo e completa flavors empacotados ausentes em
  `~/.config/argvus/yazi` quando necessário.
- Temas do btop devem usar a sintaxe nativa `theme[chave]="valor"`. O runtime
  recopia o tema Argvus ativo para `~/.config/argvus/btop`, corrigindo temas
  materializados por versões antigas depois de upgrades.
- O Foot troca de tema pelo `include` em `foot/foot.ini`, apontando para
  `foot/themes/<tema>/theme.ini`. Quando `foot` é o terminal padrão, o
  Hyprland inicia com `foot -c <foot.ini resolvido>` para usar a árvore Argvus.
