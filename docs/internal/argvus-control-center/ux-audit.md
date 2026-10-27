# Auditoria UX/UI do ARGVUS Control Center

Documento de trabalho do refactor de padronização de menus, ícones e ações
(skill `argvus-control-center-ux`). **Fase 0: auditoria, sem mudança de
comportamento.**

- Data: 2026-10-04
- Base auditada: `argvus-control-center` em `8d020c7`; `argvus-tui` em `d6f40c6`
  (resolvido por path: `../../../argvus-tui/crates/argvus-tui`, ou seja,
  `de/argvus-tui`, com fonte acessível e editável). Histórico: hoje o crate está
  em `de/argvus/crates/argvus-tui` e o repositório antigo foi aposentado em `de/retiree`.
- Catálogo i18n: `de/argvus-i18n/locales/{en-US,pt-BR}/control-center.json`
  (1256 chaves em cada idioma). Histórico: hoje está em `de/argvus/resources/locales/`.
- Fonte de ícones verificada: `/usr/share/fonts/TTF/SymbolsNerdFontMono-Regular.ttf`
  (nomes de glyph lidos da própria fonte com `fontTools`).

Legenda de tipos de linha usada abaixo: **Info**, **Action**, **Submenu**,
**Toggle**, **Choice**, **Value**, **Destructive** (ver seção 4).

Legenda de momento do efeito: **imediato** (job dispara ao ativar),
**rascunho** (só vale após `Apply`), **confirmado** (passa pela confirmação e
então dispara).

---

## Resumo dos achados

1. **Catálogo de ícones com codepoints errados.** 15 das 54 constantes de
   `argvus-tui::icons` não desenham o glyph que o comentário declara (ex.:
   `NETWORK` desenha `message_cog`, `VPN` desenha `walk`, `BOOT` desenha
   `assistant`, `FONTS` desenha `crop_free`). Detalhes na seção 1.1. Isto afeta
   todas as telas e precisa ser corrigido na Fase 1.
2. **Barra de botões em 11 crates** (`appearance`, `boot`, `packages`,
   `network`, `audio`, `bluetooth`, `hardware`, `services`, `displays`,
   `power`, `session`) e **uma variante própria** no `settings` (botões
   `argvus_tui::buttons` no fim da lista + linhas de texto `[ ... ]`). O
   diagnóstico inicial listava 7; `displays`, `power` e `session` também têm o
   trio `on_buttons`/`button_selected`/`button_from` (com enums
   `DisplayButton`, `PowerButton`, `SessionButton`).
3. **Confirmações: quatro mecanismos**, não três: (a) `ConfirmationState` +
   `draw_confirmation` compartilhados (9 crates); (b) páginas-lista
   `ThemeDeleteConfirm`/`ThemeImportConfirm` no `appearance`; (c) popup próprio
   `popup::draw_confirm` + `confirm_apply_selected` no `settings`; (d) contagem
   regressiva de reversão do `displays` (`RevertState`, teclas `y`/`n`). O
   componente compartilhado atual não tem `y`/`n` e começa com `Cancel`
   selecionado.
4. **Linhas Info recebem foco/realce** em várias páginas desenhadas com
   `page::list` (ex.: Network > DNS e Proxy, Boot > detalhes, Audio > Resumo,
   Bluetooth > Estado, Session > Diagnóstico). Ver seção 3.
5. **Ícone escolhido por índice** na Home (`home_icon_for_item(usize)`) e por
   comparação de texto traduzido nos cabeçalhos (`home_icon_for_header`).
6. **`q` e `?` globais interceptam digitação**: `event.rs:70-77` trata `q`/`?`
   antes de qualquer campo de texto (só o editor de administração e a busca da
   Home são protegidos). Digitar `q` na busca de pacotes, na busca do settings,
   em prompts do appearance/displays/boot encerra o app. Fora do escopo estrito
   de apresentação; listado em Pontos de decisão.
7. **Mouse só funciona na Home** (`event.rs:187`). As páginas de domínio não
   tratam clique; o critério "funciona com mouse" exige decisão (seção 6).
8. **Rodapé**: 43 chaves i18n de ajuda distintas, mais 4 hints montados com
   texto inglês fixo no `appearance` (`ui.rs:2527`, `2552`, `2769`, `2774`).

---

## 0. Inventário de botões e opções (anti-regressão)

Formato: `tela > botão/opção atual > ação executada (momento) > nova linha
equivalente (tipo) > atalho`. A coluna **Status** é marcada nas fases
seguintes (`pendente` → `migrado`). Nenhum item pode ficar sem equivalente.

Atalhos globais preservados em todas as telas: `↑↓/jk`, `PgUp/PgDn`,
`Home/End`, `Enter/→` ativar, `Esc/←` voltar, `?` ajuda, `q` sair,
`Ctrl+C` sair.

### 0.1 `argvus-control-center` (Home, Configuração, roteamento)

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Cartões com itens (22 ações, `action: usize`) | Abre a rota do domínio | Submenu por item, com `ItemId` estável no lugar do número | `Enter`, clique | pendente |
| Home | Navegação em grade | `Tab`/`BackTab` = próximo/anterior item; `←/→/h/l` = categoria anterior/seguinte | Mantida (Home é grade, não lista); ver decisão D7 | `Tab`, `←→hl` | pendente |
| Home | Busca global | `/` abre busca; Enter abre resultado | Mantida | `/` | pendente |
| Home | Atalho para Configuração | Abre Configuração | Mantido | `s` | pendente |
| Configuração | Ícones | Liga/desliga ícones (imediato) | Toggle | `Enter`/`Space` | pendente |
| Configuração | Transparência do Control Center | Ajuste ±, salva (imediato) | Value (`←/→`) | `←→` | pendente |
| About | Abas | Troca de aba | Mantido (`Tab` legítimo para abas) | `Tab/BackTab/h/l/←→` | pendente |

### 0.2 `argvus-control-center-appearance`

Páginas com **rascunho** (`surface_draft`/`effect_draft`, aplicação só via
`Apply`): Taskbar, TaskbarIcons, TaskbarDate, TaskbarDateFormat, TaskbarTime,
TaskbarTimeFormat, WidgetTelemetry, ControlPanel, SurfaceSection{...} e as
páginas de editor de efeito (`effect_spec`: Blur global, TerminalTransparency,
TransparencySurface). Até a Fase 2, `Esc` descartava o rascunho **sem
perguntar**.

**Fase 2 (2026-10-04): todas as linhas abaixo migradas.** Commits
`2f57d07` (lista única), `75d3dc1` (`Apply` como linha) e `4737a4c`
(`ConfirmDialog`). O crate não tem mais `Button`, `ActionButton` nem
`on_buttons`.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Tema · família [modo] | Abre Temas | Submenu | `Enter` | migrado |
| Home | Cor de destaque · valor | Abre Destaques | Submenu | `Enter` | migrado |
| Home | Wallpaper · ativo | Abre Wallpapers | Submenu | `Enter` | migrado |
| Home | Espaços/Bordas/Posição | Abre página | Submenu | `Enter` | migrado |
| Home | Taskbar | Abre página | Submenu | `Enter` | migrado |
| Home | Efeitos | Abre página | Submenu | `Enter` | migrado |
| Home | Widget Telemetry | Abre página | Submenu | `Enter` | migrado |
| Home | Control Panel | Abre página | Submenu | `Enter` | migrado |
| Home | Terminal | Abre página | Submenu | `Enter` | migrado |
| Home | Launcher | Abre página | Submenu | `Enter` | migrado |
| Home | Modo de aparência · valor | Abre Modo | Submenu | `Enter` | migrado |
| Global | Recarregar | Recarrega estado | Atalho mantido | `r` | migrado |
| Temas | Oficiais › | Abre categorias | Submenu | `Enter` | migrado |
| Temas | Personalizados › | Abre lista | Submenu | `Enter` | migrado |
| Temas | Exportar | Prompt de nome → exporta (imediato) | Action (abre Value) | `Enter`, `e` | migrado |
| Temas | Importar | Abre lista de arquivos | Submenu | `Enter`, `i` | migrado |
| Temas oficiais | Categorias › | Abre famílias | Submenu | `Enter` | migrado |
| Famílias | Tema (· atual) | Aplica tema (imediato) | Choice (`●` no atual) | `Enter` | migrado |
| Temas personalizados | Tema (· atual) | Aplica tema (imediato) | Choice | `Enter` | migrado |
| Temas personalizados | Excluir tema | Abre `ThemeDeleteConfirm` | Atalho `d` na linha do item abre o componente único de confirmação (danger, foco em Cancel); mostrado no rodapé | `d` | migrado |
| ThemeDeleteConfirm | `Delete` / `Cancel` (linhas) | Exclui / volta | **Substituído** pelo componente de confirmação (Confirm/Cancel, `y`/`n`) | `Enter`/`Esc` | migrado |
| Importar | Arquivo `.zip` | Importa (imediato, ou pede substituição) | Action por arquivo | `Enter` | migrado |
| ThemeImportConfirm | `Replace` / `Cancel` (linhas) | Substitui / volta | **Substituído** pelo componente de confirmação | `Enter`/`Esc` | migrado |
| Prompts (exportar/importar, espaços, bordas) | Campo numérico/texto | Enter confirma (imediato), Esc volta | Value (edição inline/popup) | `0-9`, `Enter`, `Esc` | migrado |
| Modo | Sticky / Float (· atual) | Troca modo (imediato) | Choice | `Enter` | migrado |
| Wallpapers | Escolher imagem da home | Abre seletor de arquivo | Action | `Enter` | migrado |
| Wallpapers | Coleções › | Abre modos | Submenu | `Enter` | migrado |
| Modos de wallpaper | Modo › | Abre itens | Submenu | `Enter` | migrado |
| Itens de wallpaper | Arquivo (· atual) | Aplica wallpaper (imediato) | Choice | `Enter` | migrado |
| Destaques | Editar cor: `#hex` | Abre AccentEdit (hex; Enter aplica, imediato) | Value | `Enter` | migrado |
| Destaques | Restaurar padrão do tema | Reseta acento (imediato) | Action | `Enter` | migrado |
| Efeitos | `[x] Animações` | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Efeitos | `[x] Blur` | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Efeitos | Blur › N% | Abre editor de efeito | Submenu | `Enter` | migrado |
| Editor de efeito (Blur global, Terminal/Launcher transparência) | Valor N% | Ajusta rascunho ±5 / 0 / 100 | Value (passo 5) no rascunho; Enter abre prompt 0–100 que grava só no rascunho (substitui `Home/End` = 0/100, D14) | `←→ h l + -`, `Enter` | migrado |
| Editor de efeito | **`[ Apply ]`** (botão, `Tab`) | Aplica valor (rascunho → imediato) | Action `Apply` no fim da lista, desabilitada sem mudança | `Enter` na linha | migrado |
| Terminal | `[x] Transparência` | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Terminal | Transparência › N% | Abre editor de efeito | Submenu | `Enter` | migrado |
| Launcher | `[x] Transparência` | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Launcher | Transparência › N% | Abre editor de efeito | Submenu | `Enter` | migrado |
| Espaços/Bordas/Posição | Posição da taskbar / Espaços da taskbar / Espaços das janelas / Bordas gerais / Espessura | Abre página | Submenu ×5 | `Enter` | migrado |
| Posição da taskbar | Topo / Base (· atual) | Move taskbar (imediato) | Choice | `Enter` | migrado |
| Espaços da taskbar | Topo/Esquerda/Direita/Base · valor | Prompt (imediato ao confirmar) | Value ×4 | `Enter` | migrado |
| Espaços das janelas | Gap interno, gaps externos ×4 | Prompt (imediato) | Value ×5 | `Enter` | migrado |
| Bordas gerais | `[x] Arredondado` | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Bordas gerais | Arredondamento · valor (· desativado) | Prompt (imediato) | Value; **desabilitada e pulada** quando Arredondado está desligado | `Enter` | migrado |
| Espessura | Espessura · valor | Prompt (imediato) | Value | `Enter` | migrado |
| Taskbar | Transparência ›, Ícones ›, Data ›, Hora › | Abre páginas | Submenu ×4 | `Enter` | migrado |
| Taskbar e subpáginas | **`[ Apply ]`** | Aplica rascunho da taskbar | Action `Apply` no fim da lista | `Enter` na linha | migrado |
| Ícones da taskbar | `[x]` player de áudio, widgets utilitários | Alterna no rascunho | Toggle (rascunho) | `Enter`/`Space` | migrado |
| Ícones da taskbar | Launcher › | Abre página (Enable + Custom, ver "Seção: launcher") | Submenu | `Enter` | migrado |
| Ícones da taskbar | Utilitários › | Abre agrupamento | Submenu | `Enter` | migrado |
| Seção: launcher | `[x] Ligar` | Alterna no rascunho | Toggle (rascunho) | `Enter`/`Space` | migrado |
| Seção: launcher | Personalizar... · caminho atual (ou "Nenhum") | Abre o File Manager padrão para escolher um ícone PNG/SVG (imediato, fora do rascunho) | Action com detalhe (caminho) | `Enter` | migrado |
| Data | Formato › | Abre formatos | Submenu | `Enter` | migrado |
| Formato de data/hora | Formato (· atual) | Seleciona no rascunho | Choice (rascunho) | `Enter` | migrado |
| Hora | `[x] Segundos` | Alterna no rascunho | Toggle (rascunho) | `Enter`/`Space` | migrado |
| Widget Telemetry | `[x] Ativar`, Sessões ›, Transparência › | Rascunho / abre páginas | Toggle + Submenu ×2 | `Enter` | migrado |
| Widget Telemetry | **`[ Apply ]`** | Aplica rascunho | Action `Apply` | `Enter` | migrado |
| Control Panel | `[x] Ativar`, Sessões ›, Transparência › | Rascunho / abre páginas | Toggle + Submenu ×2 | `Enter` | migrado |
| Control Panel | **`[ Apply ]`** | Aplica rascunho | Action `Apply` | `Enter` | migrado |
| Seção: ícones utilitários | Sempre expandido / Automático | Seleciona no rascunho | Choice (rascunho) | `Enter` | migrado |
| Seção: sessões | `[x]` blocos/cards | Alterna no rascunho | Toggle (rascunho) | `Enter`/`Space` | migrado |
| Seção: transparência/blur | `[x] Ativar` | Alterna no rascunho | Toggle (rascunho) | `Enter`/`Space` | migrado |
| Seção: transparência/blur | `Valor > N%` | Ajusta rascunho ±5 / 0 / 100 | Value (passo 5) no rascunho; Enter abre prompt 0–100 (D14) | `←→ h l + -`, `Enter` | migrado |
| Seção: * | **`[ Apply ]`** | Aplica rascunho | Action `Apply` | `Enter` | migrado |
| Páginas com rascunho | `Esc` | Descarta rascunho em silêncio | `Esc` pede confirmação (Descartar/Cancelar, foco em Cancelar) só quando a volta descarta o rascunho (sair de Taskbar, Widget Telemetry, Control Panel ou de um editor de efeito); entre as subpáginas da Taskbar o rascunho continua mantido. Sem linha `Cancel` (D2) | `Esc` | migrado |
| Páginas com rascunho | Recarregar (`r` ou automático ao entrar na página) | Trocava o rascunho pelo estado recarregado | Rascunho com alterações é mantido; rascunho limpo é reconstruído (D15) | `r` | migrado |
| Todas as páginas | `Space` | Ativava a linha (como Enter) | Mantido: ativa Action, Submenu e Choice e abre o editor de Value sem passo; em Toggle alterna (D16) | `Space` | migrado |
| Linhas Value com passo (efeitos, seções) | `←/→` | `←` voltava; `→` ajustava só nos editores de efeito | `←/→` ajustam o valor (D3); `Esc` é o jeito de voltar nesses editores; o rodapé mostra `←/→ Ajustar` e `Esc Voltar` | `←→`, `Esc` | migrado |

Páginas mortas (`#[allow(dead_code)]`, inalcançáveis pela navegação):
`Transparency`, `TransparencySurface` (exceto Launchers), `Blur` como página de
lista e `BlurSurface`. Ficam para a Fase 4 se confirmadas.

### 0.3 `argvus-control-center-settings`

Até a Fase 3, o `settings` desenhava a própria lista (`ui/list.rs`) com
`row_selectable(index)`, e os botões (`page_buttons`) ficavam **depois** das
linhas no mesmo cursor (`selected - rows.len()`), desenhados com
`argvus_tui::buttons::draw`; `Tab` alternava entre lista e botões.

**Fase 3 (2026-10-04): todas as linhas abaixo migradas.** Commits
`2f694e3` (lista única), `c423f33` (linhas de rascunho) e `14e51c1`
(`ConfirmDialog`); `argvus-tui` `0ee7a52` (títulos de seção) e `908e747`
(ícones); `argvus-i18n` `c49a792`. O crate não tem mais `Button`,
`ActionButton`, `on_buttons` nem linhas `[ ... ]`.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Aplicativos padrão, Fontes, seletores de app/fonte/ajuste | **`[ Reset Defaults ]`** | `PendingAction::Reset*` → confirmação → reset | Destructive `Restaurar padrões` na seção Zona de perigo, no fim da página (D6) | `r` (mantido, D6), `Enter` na linha | migrado |
| Atalhos de teclado | **`[ Restore all shortcuts ]`** | `ResetKeybindings` → confirmação | Destructive `Restaurar todos os atalhos` na Zona de perigo | `Enter` na linha | migrado |
| Atalhos de teclado | `r` na lista | Restaurava **o atalho selecionado**, imediato e sem confirmação | Mesmo efeito (só o selecionado), agora pela confirmação única (D6, D19) | `r` | migrado |
| Aplicativos padrão | Categorias (11) | Abre seletor | Submenu (com valor) | `Enter` | migrado |
| Seletor de app | App (atual) | Define app padrão (imediato) | Choice | `Enter`, `/` busca | migrado |
| Fontes | Alvos de fonte (7) + ajustes (4) | Abre seletor | Submenu ×11, nas seções Fontes e Renderização | `Enter` | migrado |
| Fontes | Tamanho | Ajusta tamanho | Value (passo 1, 8–32) no topo do seletor; Enter abre prompt 8–32; vale ao aplicar uma fonte, como antes (D21) | `←→`, `+ = -`, `Enter` | migrado |
| Seletor de fonte/ajuste | Opção (atual) | Aplica (imediato) | Choice | `Enter`, `/` | migrado |
| Localidade e região | Fuso, Data/hora, Locale regional, Locales do sistema, Teclado | Abre páginas | Submenu ×5 | `Enter` | migrado |
| Data e hora | Data/hora local (só sem NTP) | Edita | Value (desabilitada com NTP ligado, como hoje) | `Enter` | migrado |
| Data e hora | Fuso horário | Info | Info | — | migrado |
| Data e hora | NTP automático | `SetNtp` → confirmação | Toggle confirmado | `Enter`/`Space` | migrado |
| Locales do sistema | Locales `[x]` | Marca; aplicar = `ApplySystemLocales` → confirmação | Toggle (rascunho; Enter também alterna) + `Aplicar` na barra Ações (`locales_action_rows`, apagada sem mudança, confirmada) (D20) | `Space`/`Enter`, `/`, `Tab` | migrado |
| Teclado | Layout, Variante, Mapa do console | Abre seletores | Submenu ×3 | `Enter` | migrado |
| Teclado | Modelo, Opções | Info (já não selecionáveis) | Info | — | migrado |
| Layouts | Layout | Enter define o padrão; Space liga/desliga na lista | Toggle (`[x]` = na lista; valor "· Padrão" no padrão); Enter mantém "definir padrão"; o rodapé mostra `Enter Definir padrão` e `Space Alternar` (D22) | `Enter`, `Space` | migrado |
| Atalhos de teclado | Atalho (linhas com detalhe) | Abre edição | Submenu, agrupado por seção | `Enter`, `/`, `Space` (liga/desliga) | migrado |
| Edição de atalho | **`[ Change shortcut ]`** | Inicia captura | Action `Alterar atalho` | `Enter`, `e` | migrado |
| Edição de atalho | **`[ Disable ]`** | Desativa atalho (imediato) | Action `Desativar` | `Enter` | migrado |
| Edição de atalho | **`[ Restore default ]`** | Restaura padrão (imediato) | Action `Restaurar padrão` (imediata, como antes) | `Enter` | migrado |
| Edição de atalho | `r` | Restaurava o atalho editado e voltava, sem confirmação | Mesmo efeito, pela confirmação única (D19) | `r` | migrado |
| Captura de atalho | **`[ Apply ]` / `[ Try again ]`** | Aplica atalho capturado / recaptura | Action `Aplicar` (com tecla detectada) ou `Tentar de novo` (mesma linha) | `Enter` | migrado |
| Captura de atalho | **`[ Cancel ]`** | Cancela captura | Action `Cancelar` | `Enter`, `Esc` | migrado |
| Conflito de atalho | Popup | Substitui / cancela | Componente de confirmação (`Substituir`/`Cancelar`) | `r`, `y`, `Enter`, `Esc`/`n` | migrado |
| Mouse e touchpad | Velocidade, aceleração, rolagem, mão esquerda, toque, ratbag (dispositivo, perfil, DPI, polling) | Ciclo de valores (imediato) | Value com passo / Toggle, nas seções Mouse, Touchpad e Mouse de hardware | `←→ h l`, `Space`, `Enter` | migrado |
| Idioma | Idiomas | Troca idioma (imediato) | Choice (sem bandeira emoji) | `Enter` | migrado |
| Sistema | Hostname | Página Hostname → popup de edição → aplica | Value; Enter abre o popup direto (a página continua na rota CLI) (D21) | `Enter` | migrado |
| Sistema | Usuários, Grupos | Abre páginas | Submenu | `Enter` | migrado |
| Sistema | Não perturbe | Alterna (imediato) | Toggle | `Enter`/`Space` | migrado |
| Sistema | Recarregar | `refresh_system` | Atalho mantido | `r` | migrado |
| Usuários / Grupos | Criar, Lista, Contas do sistema | Abre páginas | Submenu | `Enter` | migrado |
| Lista de usuários / contas do sistema | **`[ Reload ]`** | Recarrega | Action `Recarregar` (primeira linha, como já era na lista de grupos) | `Enter`, `r` (D6) | migrado |
| Usuário | Usuário, UID / GID, Home | Info | Info na seção Conta (não selecionáveis) | — | migrado |
| Usuário | Campos (nome completo, shell, grupo principal, grupos suplementares) | Edita no rascunho | Value (nome) / Submenu (shell, grupos) na seção Conta | `Enter` | migrado |
| Usuário | **`[ Save changes ]`** | `edit` → confirmação | `Salvar alterações` (`draft_actions`) logo após os campos, apagada sem mudança; Esc com rascunho pede descarte | `Enter` | migrado |
| Usuário | **`[ Change password ]`** | Abre página de senha | Submenu na seção Senha | `Enter` | migrado |
| Usuário | **`[ Lock password ]` / `[ Unlock password ]`** | `lock` → confirmação | Action ×2 na seção Senha. **Não fundidas:** o snapshot de `argvus accounts` não informa o estado de bloqueio (D17) | `Enter` | migrado |
| Usuário | **`[ Require password change at login ]`** | `expire-password` → confirmação | Action (confirmada) na seção Senha | `Enter` | migrado |
| Usuário | **`[ Avatar image ]`** | Editor de caminho → confirmação | Action na seção Avatar (abre o mesmo editor) | `Enter` | migrado |
| Usuário | **`[ Remove avatar ]`** | `avatar ""` → confirmação | Action na seção Avatar, confirmação sem estilo de perigo (D18) | `Enter` | migrado |
| Usuário | **`[ Delete user (keep home) ]`** | `delete` → confirmação | Destructive na Zona de perigo, confirmação com estilo de perigo | `Enter` | migrado |
| Usuário | **`[ Delete user and home ]`** | `delete remove_home` → confirmação | Destructive na Zona de perigo, confirmação com estilo de perigo | `Enter` | migrado |
| Criar usuário | Campos | Rascunho | Value / Submenu na seção Conta | `Enter` | migrado |
| Criar usuário | **`[ Create account (locked / with password) ]`** | `create` → confirmação | `draft_actions` (rótulo dinâmico, apagada com o formulário vazio) | `Enter` | migrado |
| Senha do usuário | Campos + Salvar senha | `password` → confirmação | Value ×3 + `Salvar senha` (`draft_actions`) | `Enter` | migrado |
| Shell / Grupo principal | Opção `[x]` | Escolhe no rascunho e volta | Choice (`●`) | `Enter` | migrado |
| Grupos suplementares / Membros | Opção `[x]` | Alterna no rascunho | Toggle; o grupo principal fica desabilitado | `Enter`/`Space` | migrado |
| Criar grupo | **`[ Create group ]`** | `create-group` → confirmação | `draft_actions` (apagada sem nome) | `Enter` | migrado |
| Grupo | **`[ Save changes ]`** | `edit-group` → confirmação | `draft_actions` após Nome/GID/Membros | `Enter` | migrado |
| Grupo | **`[ Edit members ]`** | Abre membros | Submenu `Membros` (com a lista no valor) | `Enter` | migrado |
| Grupo | **`[ Delete group ]`** | `delete-group` → confirmação | Destructive na Zona de perigo, confirmação com estilo de perigo | `Enter` | migrado |
| Firewall | Serviço (Ativo/Parado), Iniciar no boot | Linhas que já pediam confirmação | Linha de estado `Serviço · Ativo/Parado` (Enter alterna, confirmado) + Toggle `Iniciar no boot` (confirmado), na seção Serviço (D23) | `Enter` | migrado |
| Firewall | Configuração (`y`/`n` → `[x]`) | Alterna no rascunho | Toggle (rascunho); nível de proteção = Value com passo; demais = Value | `Space`/`Enter`, `←→` | migrado |
| Firewall | **`[ Save configuration ]`** | `save-config` → confirmação | `draft_actions` no fim da seção Configuração | `Enter` | migrado |
| Firewall | **`[ Add iptables rules ]`** | Editor de regras → confirmação | Action na seção Regras (abre o editor) | `Enter` | migrado |
| Firewall | **`[ Apply saved rules ]`** | `restart` → confirmação | Action confirmada na seção Regras, fora da Zona de perigo; a confirmação avisa que o firewall será reiniciado (D18) | `Enter` | migrado |
| Firewall | **`[ Cancel ]`** (Danger) | `admin.load(true)`: descarta rascunho recarregando | Action `Cancelar alterações` logo após Salvar (recarrega e permanece, D2) | `Enter` | migrado |
| Páginas com rascunho (Usuário, Criar usuário, Senha, Grupo, Criar grupo, Firewall, Locales) | `Esc` | Saía em silêncio (o rascunho ficava na memória ou se perdia) | `Esc`/`←` pede confirmação (Descartar/Cancelar, foco em Cancelar); descartar volta ao estado carregado; subpáginas do mesmo rascunho não perguntam | `Esc` | migrado |
| Página Usuário / Grupo | Recarregar (snapshot) | Trocava o rascunho pelo snapshot | Rascunho com alterações é mantido (D15) | `r` | migrado |
| Editor de administração | Texto multilinha | Salva | Mantido | `Ctrl+S`, `Esc` | migrado |

### 0.4 `argvus-control-center-displays`

**Correção (Fase 3):** a versão anterior desta seção dizia que o detalhe do
monitor editava `self.config` como rascunho e que `Apply` aplicava. O código
não tem rascunho: cada opção do seletor e cada prompt (posição, escala, SDR)
aplicam **na hora** (`apply_picker_selection`, `commit_prompt`). As opções
"arriscadas" (`option_is_risky`: resolução/taxa, posição, espelho,
desativar, profundidade de cor) e o prompt de posição armam a contagem de
reversão de 15 s. `[ Apply ]` só reaplica a configuração salva, também com a
contagem (cujo destino é a mesma configuração). `[ Default ]` remove os
ajustes salvos do monitor e aplica, **sem** contagem.

**Fase 3 (2026-10-04): todas as linhas abaixo migradas.** Commits
`545bcfe` (lista única) e `c394639` (`ConfirmDialog` com prazo);
`argvus-tui` `106013b` (ícones); `argvus-i18n` `bb784d3`. O crate não tem
mais `Button`, `ActionButton`, `on_buttons` nem linhas `[ ... ]`.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Monitores conectados / configs antigas | Abre detalhe | Submenu com estado à direita; cada config antiga abre a **própria** config (antes abria sempre a primeira) | `Enter` | migrado |
| Home | Perfis | Abre perfis (linha só aparecia com perfis ou sem monitores) | Submenu `Perfis`, sempre visível | `Enter` | migrado |
| Home | **`[ Refresh ]`** | Recarrega | Action `Atualizar`, última linha | `Enter`, `r` | migrado |
| Home | **`[ Profiles ]`** | Abre perfis | Sem linha própria. **Equivalente: Enter na linha do item** `Perfis` (Submenu) (D5) | `Enter` | migrado |
| Detalhe (conectado) | Informações do monitor | Bloco readonly | Info na seção Monitor | — | migrado |
| Detalhe (conectado) | Resolução, taxa, escala, posição, orientação, ativo, espelho, profundidade, VRR, HDR, DPMS, brilho/saturação SDR, workspaces, primário | Abre seletor/prompt; aplica **na hora** (arriscadas com contagem) | Submenu por ajuste (ícone próprio, valor atual) na seção Configuração; o seletor usa Choice com `●` no valor em vigor, Toggle nos workspaces (imediato, como antes) e Value no valor personalizado | `Enter` | migrado |
| Detalhe (conectado) | **`[ Apply ]`** | Reaplica a configuração salva + contagem de 15 s | Action `Aplicar` na seção Ações, sempre habilitada, mesmo efeito; sem `draft_actions`, porque não há rascunho (D25) | `Enter` | migrado |
| Detalhe (conectado) | **`[ Default ]`** | `reset_button`: remove os ajustes salvos e aplica, imediato e **sem** contagem | Action `Restaurar padrão` na seção Ações, imediata e sem contagem, como antes (D26) | `Enter` | migrado |
| Detalhe (desconectado) | **`[ Remove config ]`** | Remove config persistida (imediato, sem confirmação) | Destructive `Remover configuração` na Zona de perigo, com confirmação em estilo de perigo | `Enter` | migrado |
| Perfis | Perfil | — (Enter só agia em "+ Novo") | Submenu que abre a página do perfil (D4); o ativo leva o valor "Ativo" | `Enter` | migrado |
| Perfis | **`[ New ]`** / "+ Novo perfil" | Prompt de nome → salva perfil | Action `Novo perfil` (abre o prompt) | `Enter` | migrado |
| Perfis | **`[ Apply ]`** | Aplica perfil selecionado | Action `Aplicar perfil` na página do perfil (D4) | `Enter` | migrado |
| Perfis | **`[ Rename ]`** | Prompt de nome | Action `Renomear` na página do perfil (abre o prompt) | `Enter` | migrado |
| Perfis | **`[ Delete ]`** | `confirm_profile` → exclui | Destructive `Excluir perfil` na Zona de perigo da página do perfil, componente único em estilo de perigo | `Enter` | migrado |
| Perfis | `r` | Recarrega o estado dos perfis | Mantido (também na página do perfil) | `r` | migrado |
| Picker | Opção | Aplica e volta ao detalhe / cancela | Choice (ou Toggle/Value, ver acima); continua voltando ao detalhe | `Enter`; `r`/`q` cancelam (`q` é capturado pelo sair global) | migrado |
| Reversão | "Manter configuração?" (faixa no topo) | Mantém / reverte; sem resposta em 15 s reverte para a config anterior (`apply_all`, "Revertido <nome>") | Componente de confirmação com prazo (`Manter`/`Reverter`, segundos restantes); prazo, destino e mensagem idênticos; foco em Reverter (D24) | `y` mantém; `n`/`Esc`/Enter (em Reverter) revertem | migrado |

### 0.5 `argvus-control-center-boot` (todas as ações passam pela confirmação)

Conferido no código antigo (Fase 3): toda ação do boot pede confirmação e
só então roda `<exe> system-settings boot <ação>`, elevado por `pkexec`
(`ensure_root` em `settings/src/system/command.rs`), com o painel de saída
ao vivo aberto na hora. Não há rascunho: o efeito é imediato após
confirmar. O timeout e a linha do kernel abrem um campo antes da
confirmação.

**Fase 3 (2026-10-05): todas as linhas abaixo migradas.** Commits
`93bc44f` (campo da linha do kernel), `1d04196` (lista única) e `a82ada7`
(`ConfirmDialog`); `argvus-i18n` `2e6be4e`. O crate não tem mais
`Button`, `ActionButton`, `on_buttons` nem linhas `[ ... ]`.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Resumo, Kernels, Bootloader, Initramfs, Plymouth | Abre páginas (e recarrega) | Submenu ×5 com o estado à direita | `Enter` | migrado |
| Global | Recarregar | `reload` | Atalho mantido | `r` | migrado |
| Resumo | Bloco readonly | — | Info em seções (Sistema base, Bootloader, Componentes); página só rola | `↑↓` | migrado |
| Kernels | Kernel | Abre detalhe | Submenu (valor "Atual · Padrão") | `Enter` | migrado |
| Kernels / detalhe | **`[ Default ]`** | `SystemdDefault` (confirmado) | Action `Definir padrão` na seção Ações da página do kernel; desabilitada com o motivo à direita quando não há entrada systemd-boot mapeável ou o bootloader é desconhecido (D28) | `Enter` | migrado |
| Bootloader | Entrada | Abre detalhe | Submenu na seção Entradas (valor "Padrão") | `Enter` | migrado |
| Bootloader / detalhe | **`[ Default ]`** (systemd-boot) | `SystemdDefault` (confirmado) | Action `Definir padrão` na seção Ações da página da entrada | `Enter` | migrado |
| Bootloader (GRUB) | **`[ Default ]`** | Sempre dava erro "não foi possível mapear" (com GRUB a lista de entradas vem vazia) | **Sem efeito no código antigo**; sem linha equivalente (D27) | — | migrado |
| Bootloader / detalhe | **`[ Timeout ]`** | Campo 0–60 → confirmado | Value `Timeout` (valor atual) na seção Configuração da página Bootloader; desabilitada com o motivo quando o bootloader é desconhecido (D28) | `Enter` | migrado |
| Bootloader (GRUB) | **`[ Kernel command line ]`** | Campo → `GrubCmdline` (confirmado) | Value `Linha do kernel` (valor atual) na seção Configuração; o campo aceita texto (D30) | `Enter` | migrado |
| Bootloader (GRUB) | **`[ Regenerate ]`** | `GrubRegenerate` (confirmado) | Destructive `Regenerar GRUB` na Zona de perigo, confirmação em estilo de perigo | `Enter` | migrado |
| Initramfs | Preset | Abre detalhe | Submenu na seção Presets; o detalhe só tem Info | `Enter` | migrado |
| Initramfs | Última linha "Regenerar todas as imagens" | `Initramfs` (confirmado) | Destructive na Zona de perigo, confirmação em estilo de perigo | `Enter` | migrado |
| Initramfs / detalhe | **`[ Regenerate ]`** | `Initramfs` (confirmado) | Sem linha própria. **Equivalente: Enter na linha do item** `Regenerar todas as imagens` (Destructive, mesma ação `Initramfs`) (D5) | `Enter` | migrado |
| Plymouth | Tema | Aplica (confirmado) | Choice confirmada (`●` no atual) | `Enter` | migrado |
| Plymouth | **`[ Apply theme ]`** | Igual ao Enter no tema | Sem linha própria. **Equivalente: Enter na linha do item** (Choice do tema) (D5) | `Enter` | migrado |
| Detalhes de kernel/entrada/preset | Linhas "d Definir padrão", "t Timeout", "g Regenerar" | Nenhuma: as teclas `d`/`t`/`g` nunca foram tratadas | Removidas (as ações viraram linhas) | — | migrado |
| Log da transação | Visualizador ao vivo | Rolagem e follow | Mantido sem mudança | `↑↓ jk PgUp/PgDn Home/End`, `Esc` | migrado |

### 0.6 `argvus-control-center-packages`

Conferido no código antigo (Fase 3): Install, Remove, Reinstall e Atualizar
tudo calculam o plano (`pacman --print`) antes da confirmação, que mostra só
contagens (instalar, remover, download); não existia página de plano.
Atualizar um pacote, Atualizar banco, as três limpezas, Downgrade e Aplicar
mirrors vão direto para a confirmação. Tudo roda elevado por `pkexec`
(`SystemSettingsOperation`), exceto a instalação pelo AUR, que roda
`paru`/`yay` como usuário normal com o aviso do PKGBUILD; o painel de saída
ao vivo abre na hora. A seleção múltipla só existia em Órfãos, pelo botão
`[ Select ]`; na lista, Space era texto do filtro e as marcas não eram
desenhadas.

**Fase 3 (2026-10-05): todas as linhas abaixo migradas.** Commits
`33a033f` (lista única) e `63689aa` (`ConfirmDialog`); `argvus-tui`
`6beda24` (glyphs `SORT` e `COUNTER`); `argvus-i18n` `ef69e70`. O crate
não tem mais `Button`, `ActionButton`, `on_buttons` nem linhas `[ ... ]`.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Instalar/Oficial, Instalar/AUR, Instalados, Órfãos, Atualizações, Cache, Histórico, Downgrade, Mirrors | Abre páginas | Submenu ×9 com o contador à direita (AUR só com paru/yay) | `Enter` | migrado |
| Listas (busca, instalados, órfãos, atualizações, AUR) | Digitar | Filtra (`query`) | Mantido (D8): letras, inclusive `j`/`k`/`r`/`q`/`?`, viram texto; setas, PgUp/PgDn, Home/End, Enter e Esc navegam; o filtro aparece na caixa `Buscar` fixa no topo e a lista no bloco `Resultados` (`Atualizações` na página Atualizações), sem a antiga linha Info "Buscar" (2026-10-24) | letras, `/` | migrado |
| Busca / AUR | Enter sem resultado | Busca remota (mínimo 1/2 caracteres) | Mantido | `Enter` | migrado |
| Listas | Pacote | Abre detalhes | Submenu (Busca, Instalados, AUR); em Órfãos, Toggle cuja Enter abre os detalhes | `Enter` | migrado |
| Instalados | **`[ Reinstall ]`** | `install_selected` (plano → confirmado) | Action `Reinstalar` na página do pacote (D4) | `Enter` | migrado |
| Instalados / busca / AUR / detalhes | **`[ Remove ]`** | `remove_selected` (plano → confirmado) | Destructive `Remover` na Zona de perigo da página do pacote (D4); desabilitada se o pacote não está instalado (antes não fazia nada) | `Enter` | migrado |
| Busca / AUR / detalhes | **`[ Install ]`** | `install_selected` (plano → confirmado; AUR só confirmado) | Action `Instalar` (`Reinstalar` quando instalado) na seção Ações da página do pacote (D4) | `Enter` | migrado |
| Atualizações | Pacote | `UpgradePackage` (confirmado) | Action por item (como hoje) | `Enter` | migrado |
| Atualizações | **`[ Update ]`** | `UpgradePackage` do selecionado | Sem linha própria. **Equivalente: Enter na linha do item** (pacote) (D5) | `Enter` | migrado |
| Atualizações | **`[ Upgrade all ]`** | `begin_plan(Upgrade)` → confirmação | Botão `Atualizar tudo` na barra `Ações` sob a lista, como em Fuso horário (2026-10-24); desabilitado e pulado pelas setas sem atualizações visíveis | `←/→`, `Enter` | migrado |
| Atualizações | **`[ Refresh database ]`** | `RefreshDatabase` (confirmado) | Botão `Atualizar banco` na barra `Ações` sob a lista (2026-10-24) | `←/→`, `Enter` | migrado |
| Órfãos | **`[ Select ]`** | `toggle_multi` (marca múltiplos) | Toggle por item, coluna `[x]` (D31) | `Space` | migrado |
| Órfãos | **`[ Remove ]`** | Remove marcados, ou o selecionado sem marcas (plano → confirmado) | Destructive `Remover marcados (N)` na Zona de perigo, desabilitada sem marcas; o pacote sozinho sai pela página dele | `Enter` | migrado |
| Cache | Lista de arquivos | Navegável, Enter sem efeito | Resumo (Info) + Submenu `Arquivos em cache`, página só de rolagem (D32) | `Enter` | migrado |
| Cache | **`[ Clean cache ]`** | `CleanCache keep-three` (confirmado) | Destructive `Limpar cache (manter 3 versões)` na Zona de perigo | `Enter` | migrado |
| Cache | **`[ Keep one ]`** | `CleanCache keep-one` (confirmado) | Destructive `Limpar cache (manter 1 versão)` na Zona de perigo | `Enter` | migrado |
| Cache | **`[ Uninstalled ]`** | `CleanCache uninstalled` (confirmado) | Destructive `Remover do cache pacotes não instalados` na Zona de perigo | `Enter` | migrado |
| Downgrade | **`[ Downgrade ]`** | `Downgrade(path)` (confirmado) | Destructive por item (o arquivo em cache), confirmação em estilo de perigo | `Enter` | migrado |
| Mirrors | Lista (readonly) | Enter abre editor reflector | Submenu `Configurar mirrors` (desabilitado com o motivo sem reflector) + seção Servidores (Info) | `Enter` | migrado |
| Editor de mirrors | País, Protocolo, Idade, Quantidade, Ordenação | Ciclo/ajuste com `←→`/Space | Página com Value ×5 (`←/→` ajustam, Space avança, Enter avança ou abre campo numérico) (D34) | `←→`, `Space`, `Enter` | migrado |
| Editor de mirrors | "Gerar prévia" | Prévia do reflector → confirmação com resumo → `mirror-apply` | Action `Gerar preview` (Primary), última linha; sem rascunho | `Enter` | migrado |
| Histórico | Entrada | **Sem efeito no código antigo** (Enter não era tratado; `HistoryDetails` inalcançável) | Info; página só de rolagem, sem página de detalhes (D33) | `↑↓` | migrado |
| Busca | `/` | Campo de busca (em qualquer página) | Mantido só nas 5 listas com filtro | `/` | migrado |
| Confirmação | Popup Apply/Cancel | — | `ConfirmDialog` (foco em Cancelar, `y`/`n`); perigo em Remove, limpezas e Downgrade; contagens do plano com download legível; AUR só com o aviso do PKGBUILD (D35) | `y`, `n`, `Esc` | migrado |
| Log da transação | Visualizador ao vivo | Rolagem e follow | Mantido sem mudança | `↑↓ jk PgUp/PgDn Home/End`, `Esc` | migrado |
| Global | Recarregar | `reload_force` | Atalho mantido (nas listas com filtro `r` é texto, como antes) | `r` | migrado |

**2026-10-24: caminhos e Atualizações no padrão Fuso horário.** O breadcrumb
de cada lista repete a linha da Home sem a barra:
`System Administration > Packages > Install Official`, `> Install AUR`,
`> Installed Official`, `> Installed Orphans` e `> Updates Official`; a página
de um pacote acrescenta o nome dele ao caminho da lista de origem. A busca
global usa os mesmos rótulos. Atualizações ganhou a caixa `Buscar` fixa no
topo, a lista com borda e a barra `Ações` (`Atualizar tudo`, `Atualizar
banco`): `↑/↓` navegam na lista, `←/→` e `Tab` entram na barra e andam entre
os botões, `Enter`/`Space` executam o botão focado, e digitar com a barra
focada devolve o foco ao filtro. Lista vazia mostra "Nenhuma atualização
pendente" (ou "Nenhum item encontrado" com filtro). As outras quatro listas (Instalar Official, Instalar AUR, Instalados Official, Instalados Órfãos) usam o mesmo layout, sem a barra: caixa `Buscar` no topo e bloco `Resultados`; vazias, mostram "Digite / para buscar…" (Busca e AUR sem filtro), "Nenhum resultado" ou "Nenhum item encontrado". Em Órfãos, `Remover marcados` continua na Zona de perigo dentro do bloco. O rodapé dessa página
mostra `Enter Executar`, `←/→ Ações` e `Esc Voltar`, porque as setas laterais
pertencem à barra.

### 0.7 `argvus-control-center-network`

**Fase 3 (2026-10-05): todas as linhas abaixo migradas.** Commits `83a0461`
(lista única) e `d5df88d` (`ConfirmDialog`); `argvus-tui` `541d788` (glyphs
`PROXY` e `AUTORENEW`); `argvus-i18n` `b46d99b`. O crate não tem mais
`Button`, `ActionButton`, `on_buttons` nem linhas `[ ... ]`. Acréscimo: atalho
`i` (Detalhes) em Wi-Fi e VPN (D36).

Conferido no código antigo: Enter em Wi-Fi e VPN conecta; Enter em interface
abre o detalhe; DNS manual abre o prompt e DNS automático aplica sem
confirmação; Esquecer é o único fluxo com confirmação; a senha é mascarada; o
helper elevado do DNS (`SystemSettingsOperation`) não muda.

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Status, Interfaces, Ethernet, Wi-Fi, VPN, DNS, Proxy, Firewall | Abre páginas | Submenu ×páginas disponíveis, com o estado à direita | `Enter` | migrado |
| Status | **`[ Enable Wi-Fi ]` / `[ Disable Wi-Fi ]`** | `toggle_wifi` (imediato) | Toggle `Wi-Fi` (`[x]`/`[ ]`); desabilitada sem estado de rádio (D37) | `Enter`/`Space` | migrado |
| Status, Interfaces, Ethernet, VPN, Wi-Fi, DNS, Proxy, Detalhe | **`[ Refresh ]`** | `start_refresh` (Wi-Fi com varredura) | Action `Atualizar`: no topo em Status, DNS, Proxy e detalhes; no fim das listas (D42) | `r` | migrado |
| Interfaces / Ethernet | Interface | Abre detalhe | Submenu | `Enter`, `/` filtro | migrado |
| Interfaces / Ethernet | **`[ Connect ]`** | `connect_selected` (imediato) | Action `Conectar` no detalhe da interface (D39) | `Enter` | migrado |
| Interfaces / Ethernet | **`[ Disconnect ]`** | `disconnect_selected` (imediato) | Action `Desconectar` no detalhe (D39, D40) | `Enter` | migrado |
| Wi-Fi | Rede | `connect_wifi` (senha se preciso) | Action; Enter mantém a conexão | `Enter` | migrado |
| Wi-Fi | **`[ Connect ]` / `[ Disconnect ]`** | imediato | Actions no detalhe da rede, aberto com `i` (D36) | `i`, `Enter` | migrado |
| Wi-Fi | **`[ Forget ]`** | `confirm_forget` → esquece | Destructive `Esquecer` no fim do detalhe (Zona de perigo), `ConfirmDialog` (D38) | `i`, `Enter`, `y` | migrado |
| VPN | Conexão | `connect_selected` (imediato) | Action; Enter conecta | `Enter` | migrado |
| VPN | **`[ Connect ]` / `[ Disconnect ]`** | imediato | Actions no detalhe da VPN, aberto com `i` (D36, D39) | `i`, `Enter` | migrado |
| DNS | **`[ Manual DNS ]`** | Input → aplica | Value `DNS manual` (abre o prompt) | `Enter` | migrado |
| DNS | **`[ Automatic DNS ]`** | `apply_dns("")` (imediato) | Action `DNS automático` (ícone `AUTORENEW`) | `Enter` | migrado |
| Proxy | Refresh | `start_refresh` | Action `Atualizar` | `r`, `Enter` | migrado |
| Firewall | Página do settings embutida | Igual a 0.3 | Igual a 0.3, sem mudança | — | migrado |
| Home | **Conexões** (novo, 2026-10-27) | Abre a lista de perfis | Submenu `Conexões`, com as conexões ativas à direita | `Enter` | adição (D68) |
| Home | **Ativar uma conexão** (novo) | Abre a lista de ativação | Submenu | `Enter` | adição (D68) |
| Conexões | Perfil | Abre o editor | Submenu por perfil, agrupado por tipo | `Enter`, `/` filtro | adição (D68) |
| Conexões | Conectar/desconectar o perfil | `connection up/down uuid` (imediato) | Atalho `a` no perfil (rodapé mostra Conectar/Desconectar) | `a` | adição (D68) |
| Conexões | Adicionar conexão | Abre os tipos | Submenu no topo (Ações) | `Enter` | adição |
| Ativar uma conexão | Perfil | `connection up/down uuid` (imediato) | Action por perfil | `Enter` | adição |
| Adicionar conexão | Tipo, plugin VPN, importar arquivo | Abre o editor com o rascunho novo; importar pede o caminho | Action por tipo; desabilitada com motivo sem `teamd`/`pppd`/plugin | `Enter` | adição |
| Editor | Campos | Rascunho | Value (←/→ em escolhas, números e dispositivos), Toggle, Submenu para listas | `Enter`, `←/→`, `Space` | adição |
| Editor | **Salvar** | `nmcli connection add/modify` + segredos pela stdin; reaplica se ativa (confirmação) | `draft_actions`: desabilitada sem mudança ou com erro, motivo no detalhe | `Enter` | adição (D70) |
| Editor | Cancelar | Descarta (confirma se houver mudança) | Action | `Enter`, `Esc` | adição |
| Editor | Conectar/Desconectar | `connection up/down uuid` (imediato) | Action na seção Conexão | `Enter` | adição |
| Editor | **Excluir conexão** | `connection delete uuid` (+ portas) | Destructive na Zona de perigo, `ConfirmDialog` com foco em Cancelar | `Enter`, `y` | adição (D71) |
| Detalhe de interface, Wi-Fi salva, VPN | **Editar conexão** | Abre o perfil no editor | Submenu nas Ações | `Enter` | adição |

### 0.8 `argvus-control-center-audio`

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Resumo, Saídas, Entradas, Dispositivos | Abre páginas | Submenu ×4 | `Enter` | pendente |
| Saídas / Entradas | Dispositivo | `request_default` (confirmado) | Choice confirmada (`●` no padrão) | `Enter` | migrado como linha de texto com `● Padrão` (não é tipo Choice ainda) |
| Saídas / Entradas | **`[ Default ]`** | Igual ao Enter | Sem linha própria. **Equivalente: Enter na linha do item**; vira rascunho e só vale no `s Salvar` (D5) | `Enter`, `s` | migrado |
| Saídas / Entradas | **`[ Volume + ]` / `[ Volume - ]`** | `adjust_volume(±5)` (rascunho, aplicado no `s`) | Sem linha própria; `←/→` (e `+`/`-`) ajustam ±5 no rascunho do dispositivo selecionado | `←→ + -`, `s` | migrado |
| Saídas / Entradas | **`[ Mute ]`** | `toggle_mute` (rascunho, aplicado no `s`) | Sem linha própria; `Space` alterna o mudo no rascunho (decisão: o mudo também segue o `s`) | `Space`, `s` | migrado |
| Saídas / Entradas | Salvar | Aplica todo o rascunho (`set-default`, `set-volume`, `set-mute`) | Nova linha de rodapé `s Salvar`; sem rascunho mostra "Nenhuma alteração para salvar"; `Esc` com rascunho pede confirmação para descartar | `s`, `Esc` | migrado |
| Saídas / Entradas | **`[ Value ]`** | Input numérico de volume | Sem linha própria; tecla `v` abre o campo 0–100 (fora do rodapé pedido; a confirmar) | `v` | migrado (a confirmar) |
| Global | Recarregar | `reload` | Atalho mantido | `r` | migrado |

### 0.9 `argvus-control-center-bluetooth`

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Estado, Dispositivos, Parear | Abre páginas | Submenu ×3 | `Enter` | pendente |
| Estado | **`[ Power ]`** | Liga/desliga adaptador (imediato) | Toggle `Ligado` | `Enter`/`Space` | pendente |
| Estado | **`[ Discoverable ]`** | Alterna visibilidade (imediato) | Toggle `Visível` | `Enter`/`Space` | pendente |
| Dispositivos | Dispositivo | Conecta/desconecta/pareia conforme estado | Action (Enter mantém conectar/desconectar/parear); ações secundárias (Trust, Remove) na página de detalhe do dispositivo (D4) | `Enter` | pendente |
| Dispositivos / Parear | **`[ Connect ]`**, **`[ Disconnect ]`** | `act_device` (imediato) | Action ×2 | `Enter` | pendente |
| Dispositivos / Parear | **`[ Trust ]`** | trust/untrust (imediato) | Toggle `Confiável` | `Space` | pendente |
| Dispositivos / Parear | **`[ Remove ]`** | `confirm_remove` → remove | Destructive | `Enter` | pendente |
| Parear | Dispositivo / **`[ Pair ]`** | `act_pair` (imediato, agente) | Action `Parear` | `Enter` | pendente |
| Global | Recarregar | `reload` | Atalho mantido | `r` | pendente |

### 0.10 `argvus-control-center-hardware`

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Resumo, CPU, GPU, Memória, Energia, Dispositivos | Abre páginas | Submenu ×6 | `Enter` | pendente |
| CPU | Governor (★ ativo) | `SetGovernor` (confirmado) | Choice confirmada | `Enter`, `g` | pendente |
| CPU | **`[ Apply Governor ]`** | Igual ao Enter | Sem linha própria. **Equivalente: Enter na linha do item** (Choice) (D5) | `Enter` | pendente |
| Energia | Perfil (★ ativo) | `SetProfile` (confirmado) | Choice confirmada | `Enter`, `e` | pendente |
| Energia | **`[ Apply Profile ]`** | Igual ao Enter | Sem linha própria. **Equivalente: Enter na linha do item** (Choice) (D5) | `Enter` | pendente |
| GPU / Dispositivos | Item | Abre detalhe | Submenu | `Enter` | pendente |
| Global | Atualizar | `refresh` | Atalho mantido | `r` | pendente |

### 0.11 `argvus-control-center-services`

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Sistema, Usuário, Falhos, Logs | Abre páginas | Submenu ×4 | `Enter` | pendente |
| Sistema / Usuário / Falhos | Unidade | Abre detalhe (que **já** lista ações como linhas) | Submenu | `Enter`, `/` busca | pendente |
| Sistema / Usuário / Falhos | **`[ Start ]`**, **`[ Stop ]`**, **`[ Restart ]`** | `request` (confirmado) | Action ×3 no detalhe (já existem lá) | `Enter` | pendente |
| Sistema / Usuário / Falhos | **`[ Filter: X ]`** | `cycle_filter` | Linha `Filter:` fora do menu, com o valor à direita; cicla com `f` | `f` | migrado |
| Detalhe | Start, Stop, Restart, Enable, Disable, Enable now, Disable now, Logs | `request` (confirmado) / abre logs | Action ×7 + Submenu (já são linhas) | `Enter` | pendente |
| Logs | **`[ Boot: atual/anterior ]`** | Alterna boot | Linha `Filter:` mostra `Boot: …`; `b` alterna | `b` | migrado |
| Logs | **`[ Service: X ]`** | `cycle_log_unit` | Linha `Filter:` mostra `Service: …`; `u` cicla | `u` | migrado |
| Logs | **`[ Priority: X ]`** | `cycle_priority` | Linha `Filter:` mostra `Priority: …`; `p` cicla (corrigido: a partir de "0-7" agora vai para "0") | `p` | migrado |
| Logs | Entrada | Abre detalhe do log | Submenu | `Enter`, `/` | pendente |
| Global | Atualizar | `refresh` | Atalho mantido | `r` | pendente |

### 0.12 `argvus-control-center-power` (página única)

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Energia | Tampa fechada (bateria/AC), botão de energia, tela desligada após (×2), bloquear após (×2), manter acordado | Abre picker → aplica (imediato) | Choice/Value (picker) ×7 (3 em desktop) | `Enter` | pendente |
| Energia | **`[ Suspend now ]`** | `Pending::Suspend` (confirmado) | Botão da barra Ações (`argvus_tui::action_buttons`, primário) | `←/→`, `Enter` | migrado |
| Energia | **`[ Hibernate now ]`** | `Pending::Hibernate` (confirmado) | Botão da barra Ações (perigo) | `←/→`, `Enter` | migrado |
| Energia | **`[ Refresh ]`** | `refresh` | Botão da barra Ações (secundário) | `r`, `←/→`, `Enter` | migrado |

### 0.13 `argvus-control-center-session`

| Tela | Botão/opção atual | Ação (momento) | Nova linha equivalente | Atalho | Status |
| --- | --- | --- | --- | --- | --- |
| Home | Componentes, Autostart, Diagnóstico, Logs | Abre páginas | Submenu ×4 | `Enter` | pendente |
| Componentes | Componente / **`[ Restart ]`** | `Pending::Restart` (confirmado) | Action por item (Enter, como hoje) | `Enter` | pendente |
| Autostart | Entrada / **`[ Enable ]`** / **`[ Disable ]`** | `ToggleAutostart` (confirmado) | Toggle confirmado por item | `Enter`/`Space` | pendente |
| Logs | **`[ Service: X ]`** | `toggle_log_filter` | Choice `Serviço` | `Enter`/`←→` | pendente |
| Logs | Entrada | Abre detalhe | Submenu | `Enter` | pendente |
| Componentes, Autostart, Logs | **`[ Refresh ]`** | `refresh` | Action `Atualizar` | `r` | pendente |

### 0.14 `storage`, `diagnostics`, `about`, `apps`

Sem botões. `storage` e `diagnostics`: Home com submenus, listas que abrem
detalhes, detalhes em `readonly`, `r` atualiza. `about`: abas + rolagem.
`apps`: sem UI própria (backend/CLI usado pelo settings).

---

## 1. Tabela de ícones

### 1.1 Catálogo `argvus-tui::icons`: codepoints que não batem com o nome

Verificado contra os nomes de glyph da Symbols Nerd Font Mono instalada.
Codepoint correto = glyph com o nome do comentário.

| Constante | Atual | Desenha hoje | Nome pretendido | Codepoint correto |
| --- | --- | --- | --- | --- |
| `NETWORK` | `f06f1` | message_cog | nf-md-network | `f06f3` |
| `WIFI` | `f0928` | wifi_strength_4 | nf-md-wifi | `f05a9` (ou manter, D9) |
| `VPN` | `f0583` | walk | nf-md-vpn | `f0582` |
| `DNS` | `f0155` | clock_start | nf-md-dns | `f01d6` |
| `GPU` | `f0fb2` | expansion_card_variant | nf-md-expansion-card | `f08ae` (ou manter) |
| `BOOT` | `f0064` | assistant | nf-md-boot (**não existe**) | proposta: `power_cycle` `f0901` |
| `PACKAGES` | `f03d7` | package_variant_closed | nf-md-package-variant | `f03d6` (ou manter) |
| `INSTALLED` | `f03d9` | palette_advanced | nf-md-package-check (**não existe**) | proposta: `package_variant_closed_plus` `f19d5` |
| `DIAGNOSTICS` | `f0151` | clock_end | nf-md-chart-box | `f154d` |
| `FONTS` | `f019f` | crop_free | nf-md-format-font | `f06d6` |
| `IMAGE` | `f02f9` | image_multiple | nf-md-image | `f02e9` |
| `USERS` | `f000d` | account_minus | nf-md-account-multiple | `f000e` |
| `BELL` | `f009c` | bell_outline | nf-md-bell | `f009a` |
| `BELL_OFF` | `f009e` | bell_ring | nf-md-bell-off | `f009b` |
| `PDF` | `f0e2d` | file_png_box | nf-md-file-pdf-box | `f0226` |

As outras 39 constantes conferem. Além disso, `Appearance > Modo > Float` usa o
emoji literal `"🪟"` (largura 2, fora do catálogo).

### 1.2 Novos glyphs propostos para o catálogo (todos conferidos na fonte)

| Constante proposta | Glyph | Codepoint | Uso |
| --- | --- | --- | --- |
| `TASKBAR` | nf-md-dock_bottom | `f10a9` | Taskbar |
| `WALLPAPER` | nf-md-wallpaper | `f0e09` | Wallpaper |
| `TERMINAL` | nf-md-console | `f018d` | Terminal |
| `LAUNCHER` | nf-md-rocket_launch | `f14de` | Launcher |
| `TELEMETRY` | nf-md-gauge | `f029a` | Widget Telemetry |
| `CONTROL_PANEL` | nf-md-tune_variant | `f1542` | Control Panel |
| `LAYOUT` | nf-md-view_dashboard | `f056e` | Espaços/Bordas/Posição |
| `THEME_MODE` | nf-md-theme_light_dark | `f050e` | Modo de aparência |
| `WINDOW_FLOAT` | nf-md-window_restore | `f05b2` | Modo Float |
| `WINDOW_STICKY` | nf-md-view_split_vertical | `f0bcc` | Modo Sticky |
| `ACCENT` | nf-md-format_color_fill | `f0266` | Cor de destaque |
| `ARROW_UP` / `ARROW_DOWN` / `ARROW_LEFT` / `ARROW_RIGHT` | nf-md-arrow_*_bold | `f0737` / `f072e` / `f0731` / `f0734` | Posição e espaços por lado |
| `GAP` | nf-md-arrow_expand_horizontal | `f084e` | Gaps |
| `BORDER` | nf-md-border_style | `f00d0` | Bordas gerais |
| `ROUNDED` | nf-md-rounded_corner | `f0607` | Arredondado/arredondamento |
| `THICKNESS` | nf-md-format_line_weight | `f05c9` | Espessura |
| `OPACITY` | nf-md-opacity | `f05cc` | Transparência |
| `BLUR` | nf-md-blur | `f00b5` | Blur |
| `ANIMATION` | nf-md-animation_play | `f093a` | Animações |
| `CALENDAR` | nf-md-calendar | `f00ed` | Data |
| `TIMER` | nf-md-timer_outline | `f051b` | Tempo limite |
| `TRANSLATE` | nf-md-translate | `f05ca` | Idioma |
| `EARTH` | nf-md-earth | `f01e7` | Localidade/fuso/região |
| `APPLY` | nf-md-check_bold | `f0e1e` | Linha `Apply` |
| `CANCEL` | nf-md-cancel | `f073a` | Linha `Cancel` |
| `DELETE` | nf-md-delete | `f01b4` | Ações destrutivas de exclusão |
| `RESTORE` | nf-md-restore | `f099b` | Restaurar padrão |
| `IMPORT` / `EXPORT` | nf-md-import / nf-md-export | `f02fa` / `f0207` | Temas |
| `EDIT` | nf-md-pencil | `f03eb` | Editar/renomear |
| `ADD` | nf-md-plus | `f0415` | Novo/criar |
| `PLAY` / `STOP` / `RESTART` | nf-md-play / stop / restart | `f040a` / `f04db` / `f0709` | Serviços, componentes |
| `FILTER` | nf-md-filter_variant | `f0236` | Filtros de lista/log |
| `SLEEP` | nf-md-power_sleep | `f0904` | Suspender |
| `HIBERNATE` | nf-md-snowflake | `f0717` | Hibernar |
| `LINK_ON` / `LINK_OFF` | nf-md-lan_connect / lan_disconnect | `f0318` / `f0319` | Conectar/desconectar |
| `SHIELD` | nf-md-shield_outline | `f0499` | Firewall |
| `CLEAN` | nf-md-broom | `f00e2` | Limpar cache |
| `DOWNGRADE` | nf-md-package_down | `f03d4` | Downgrade |
| `DATABASE` | nf-md-database_refresh | `f05c2` | Atualizar banco |
| `CHIP_CPU` | já existe `CPU` | `f0ee0` | CPU |
| `DEVICES` | nf-md-devices | `f0fb0` | Dispositivos |
| `PROFILE` | nf-md-layers_outline | `f09fe` | Perfis |
| `AUTOSTART` | nf-md-launch | `f0327` | Autostart |
| `USB` | nf-md-usb | `f0553` | Dispositivos de hardware |
| `VOLUME_UP` / `VOLUME_DOWN` / `MUTE` | nf-md-volume_plus / volume_minus / volume_off | `f075d` / `f075e` / `f0581` | Áudio |
| `TRUST` | nf-md-shield_check | `f0565` | Bluetooth confiável |
| `VISIBLE` | nf-md-eye | `f0208` | Bluetooth visível |
| `CONNECTIONS` | nf-md-lan | `f0317` | Rede > Conexões (2026-10-27) |
| `ACTIVATE_CONNECTION` | nf-md-play_network | `f088b` | Rede > Ativar uma conexão |
| `BRIDGE` / `BOND` / `TEAM` | nf-md-bridge / call_merge / source_merge | `f0618` / `f00f8` / `f062d` | Tipos de conexão |
| `VLAN` / `TUNNEL` / `LOOPBACK` | nf-md-tag / tunnel / rotate_left | `f04f9` / `f183d` / `f0465` | Tipos de conexão |
| `DSL` / `HOTSPOT` / `WIREGUARD` | nf-md-phone_classic / access_point / shield_key | `f0602` / `f0003` / `f0bc4` | Tipos de conexão |
| `IP_ADDRESS` / `ROUTES` | nf-md-ip_network / map_marker_path | `f0a60` / `f0d20` | Reservados para listas de endereço e rota |
| `PORT` / `KEY_GENERATE` | nf-md-vector_link / key_plus | `f0fe8` / `f0309` | Portas de bridge/bond/team; gerar chave WireGuard |

### 1.3 Home (`argvus-control-center/src/ui.rs`)

Cabeçalhos de cartão (`home_icon_for_header`, por texto traduzido):

| Cartão | Ícone atual | Ícone proposto |
| --- | --- | --- |
| Idioma e região | `NETWORK` (desenha message_cog) | `EARTH` |
| Aparência | `PALETTE` | `PALETTE` |
| Aplicativos | `APPS` | `APPS` |
| Hardware | `MONITOR` | `HARDWARE` (chip) |
| Energia e sessão | `POWER` | `POWER` |
| Conectividade | `NETWORK` | `NETWORK` (corrigido) |
| Áudio | `AUDIO` | `AUDIO` |
| Sistema | `SETTINGS` | `SERVICES` (server) |
| Preferências | `SETTINGS` | `SETTINGS` |

Itens (`home_icon_for_item(action: usize)`, por índice):

| Item (índice) | Ícone atual | Problema | Ícone proposto |
| --- | --- | --- | --- |
| Aplicativos padrão (0) | `APPS` | — | `APPS` |
| Fontes (1) | `FONTS` | glyph errado (crop_free) | `FONTS` (corrigido) |
| Localidade e região (2) | `NETWORK` | sentido errado | `EARTH` |
| Idioma (3) | `KEYBOARD` | sentido errado | `TRANSLATE` |
| Hardware (4) | `MONITOR` | repete com Displays (irmãos) | `HARDWARE` |
| Rede (5) | `NETWORK` | glyph errado | `NETWORK` (corrigido) |
| Áudio (6) | `AUDIO` | — | `AUDIO` |
| Bluetooth (7) | `LINK` | genérico | `BLUETOOTH` |
| Boot (8) | `BOOT` | glyph errado (assistant) | `BOOT` (power_cycle) |
| Pacotes (9) | `PACKAGES` | — | `PACKAGES` |
| Serviços (10) | `SETTINGS` | repete com Sistema/Configuração | `SERVICES` |
| Sistema (11) | `SETTINGS` | repete com Serviços/Configuração | `SETTINGS` (passa a ser o único com cog) |
| Armazenamento (12) | `STORAGE` | — | `STORAGE` |
| Diagnóstico (13) | `DIAGNOSTICS` | glyph errado (clock_end) | `DIAGNOSTICS` (corrigido) |
| Sobre (14) | `INFO` | — (é informação) | `INFO` |
| Configuração (15) | `SETTINGS` | repete com Sistema | `CONTROL_PANEL` (tune) |
| Energia (16) | `POWER` | — | `POWER` |
| Sessão (17) | `REFRESH` | sentido errado | `USER` (sessão do usuário) |
| Displays (18) | `MONITOR` | repete com Hardware | `MONITOR` |
| Aparência (19) | `PALETTE` | — | `PALETTE` |
| Mouse e touchpad (20) | `MOUSE` | — | `MOUSE` |
| Atalhos de teclado (21) | `KEYBOARD` | repete com Idioma | `KEYBOARD` |

### 1.4 `appearance`

Migrado na Fase 2 (`2f57d07`), com os ícones da proposta abaixo. Os itens
fora desta tabela seguem o D9 registrado em 6.1: listas homogêneas
(famílias, temas personalizados, `.zip`, arquivos de wallpaper, formatos,
blocos, cards, widgets utilitários) ficam sem ícone; Escuro/Claro dos
temas oficiais = `THEME_MODE`; player de áudio = `MUSIC`; launcher =
`LAUNCHER`; Utilitários › = `WIDGET`; Formato › = `CALENDAR` (data) /
`CLOCK` (hora); Segundos = `TIMER`; Ativar = `TELEMETRY` / `CONTROL_PANEL`;
Sessões › = `LAYOUT`; editores de valor = `BLUR` / `OPACITY`; `Apply` =
`APPLY`.

| Tela | Item | Ícone atual | Ícone proposto |
| --- | --- | --- | --- |
| Home | Tema | `PALETTE` | `PALETTE` |
| Home | Cor de destaque | `PALETTE` (repete Tema) | `ACCENT` |
| Home | Wallpaper | `IMAGE` | `WALLPAPER` |
| Home | Espaços/Bordas/Posição | `STORAGE` (HDD) | `LAYOUT` |
| Home | Taskbar | `STORAGE` (HDD) | `TASKBAR` |
| Home | Efeitos | `SUCCESS` (ok) | `EFFECT` |
| Home | Widget Telemetry | `WIDGET` (repete Control Panel) | `TELEMETRY` |
| Home | Control Panel | `WIDGET` | `CONTROL_PANEL` |
| Home | Terminal | `STORAGE` (HDD) | `TERMINAL` |
| Home | Launcher | `STORAGE` (HDD) | `LAUNCHER` |
| Home | Modo de aparência | `STORAGE` (HDD) | `THEME_MODE` |
| Temas | Oficiais ›, Personalizados › | sem ícone | `PALETTE`, `EDIT` |
| Temas | Exportar / Importar | `STORAGE` / `STORAGE` | `EXPORT` / `IMPORT` |
| Modo | Sticky / Float | `FOLDER` / emoji `🪟` | `WINDOW_STICKY` / `WINDOW_FLOAT` |
| Wallpapers | Escolher da home | `FOLDER` | `FOLDER` |
| Wallpapers | Coleções / Modos | `IMAGE` (todos iguais) | `IMAGE` (mesmo sentido: aceito) |
| Destaques | Editar / Restaurar | sem ícone | `EDIT` / `RESTORE` |
| Efeitos | Animações / Blur (toggle) / Blur % | `SUCCESS` / `SUCCESS` / `INFO` | `ANIMATION` / `BLUR` / `BLUR` (Value; mesmo sentido) |
| Terminal, Launcher | Transparência (toggle) / % | sem ícone | `OPACITY` |
| Espaços/Bordas/Posição | Posição da taskbar | `INFO` | `TASKBAR` |
| Espaços/Bordas/Posição | Espaços da taskbar / das janelas | `STORAGE` / `STORAGE` | `GAP` / `WINDOW_STICKY` (→ D9) |
| Espaços/Bordas/Posição | Bordas gerais / Espessura | `INFO` / `INFO` | `BORDER` / `THICKNESS` |
| Posição da taskbar | Topo / Base | `INFO` | `ARROW_UP` / `ARROW_DOWN` |
| Espaços da taskbar | Topo/Esquerda/Direita/Base | `INFO` | setas por lado |
| Espaços das janelas | Gap interno / externos ×4 | `INFO` | `GAP` / setas por lado |
| Bordas gerais | Arredondado / Arredondamento | `INFO` | `ROUNDED` |
| Espessura | Espessura | `INFO` | `THICKNESS` |
| Taskbar | Transparência / Ícones / Data / Hora | sem ícone | `OPACITY` / `APPS` / `CALENDAR` / `CLOCK` |
| Todas as páginas com rascunho | `Apply` (botão) | — | `APPLY` |

### 1.5 Demais crates

| Crate > tela | Item | Ícone atual | Ícone proposto |
| --- | --- | --- | --- |
| audio > Home | Resumo / Saídas / Entradas / Dispositivos | `MONITOR` / `AUDIO` / `MICROPHONE` / `SPEAKER` | `INFO` / `SPEAKER` / `MICROPHONE` / `DEVICES` |
| audio > Resumo | cabeçalhos de seção | `MONITOR`, `AUDIO`, `SPEAKER` | sem ícone (Info) |
| boot > Home | Resumo / Kernels / Bootloader / Initramfs / Plymouth | `MONITOR` / `MEMORY` / `STORAGE` / `PACKAGES` / `PALETTE` | `INFO` / `CPU` / `BOOT` / `PACKAGES` / `IMAGE` (Fase 3) |
| boot > detalhes | cabeçalhos | `MEMORY`, `STORAGE`, `PACKAGES` | sem ícone (títulos de seção) (Fase 3) |
| boot > ações | Definir padrão / Timeout / Linha do kernel / Regenerar GRUB / Regenerar initramfs | — | `STAR` / `TIMER` / `TERMINAL` / `SYNC` / `SYNC` (mesmo sentido) (Fase 3, D29) |
| boot > listas | kernels, entradas, presets, temas Plymouth | — | sem ícone (listas homogêneas, D9) (Fase 3) |
| diagnostics > Home | Resumo, Serviços, Kernel e boot, Gráficos, Rede, Áudio, Bluetooth, Armazenamento, Pacotes, ARGVUS | `MONITOR`, `SETTINGS`, `MEMORY`, `GPU`, `NETWORK`, `AUDIO`, `LINK`, `STORAGE`, `PACKAGES`, `SUCCESS` | `INFO`, `SERVICES`, `BOOT`, `GPU`, `NETWORK`, `AUDIO`, `BLUETOOTH`, `STORAGE`, `PACKAGES`, `PALETTE`/logo (D9) |
| diagnostics | `category_icon` (mapa duplicado do anterior) | idem | unificar com o item |
| displays > Home | Monitor / desconectado / Perfis / Atualizar | `MONITOR` / `ETHERNET` / `APPS` / — | `MONITOR` / `LINK_OFF` / `PROFILE` / `REFRESH` (Fase 3) |
| displays > detalhe | "Monitor desconectado" | `ETHERNET` | sem ícone (Info) (Fase 3) |
| displays > detalhe | Resolução / Taxa / Escala / Posição / Orientação / Ativo / Espelho / Profundidade / VRR / HDR / DPMS / Brilho SDR / Saturação SDR / Workspaces / Primário | — | `ASPECT_RATIO` / `SINE_WAVE` / `ZOOM` / `ARROW_ALL` / `ROTATE` / `POWER` / `MONITOR_MULTIPLE` / `PALETTE` / `SYNC` / `HDR` / `SLEEP` / `BRIGHTNESS` / `CONTRAST` / `GRID` / `STAR` (Fase 3) |
| displays > detalhe | Aplicar / Restaurar padrão / Remover configuração | — | `APPLY` / `RESTORE` / `DELETE` (Fase 3) |
| displays > perfis | Novo perfil / Aplicar perfil / Renomear / Excluir perfil | — | `ADD` / `APPLY` / `EDIT` / `DELETE` (Fase 3) |
| hardware > Home | Resumo / CPU / GPU / Memória / Energia / Dispositivos | `MONITOR` / `MEMORY` / `GPU` / `POWER` / `BATTERY` / `ETHERNET` | `INFO` / `CPU` / `GPU` / `MEMORY` / `BATTERY` / `USB` |
| network > Home | Status / Interfaces / Ethernet / Wi-Fi / VPN / DNS / Proxy / Firewall | `NETWORK` / `ETHERNET` / `NETWORK` / `WIFI` / `LOCK` / `SEARCH` / `LOCK` / `WARNING` | `INFO` / `NETWORK` / `ETHERNET` / `WIFI` / `VPN` / `DNS` / `LINK` / `SHIELD` |
| network > páginas | cabeçalhos de seção | `NETWORK`, `WIFI`, `ETHERNET`, `SEARCH`, `LOCK` | sem ícone (Info) |
| packages > Home | Instalar oficial / AUR / Instalados / Órfãos / Atualizações / Cache / Histórico / Downgrade / Mirrors | `SEARCH` / `SUCCESS` / `PACKAGES` / `ERROR` / `REFRESH` / `STORAGE` / `LOGS` / `UPDATE` / `NETWORK` | `SEARCH` / `ADD` / `INSTALLED` / `CLEAN` / `UPDATE` / `DATABASE` / `HISTORY` / `DOWNGRADE` / `NETWORK` (Fase 3) |
| packages > Home (D55) | Repositórios oficiais / AUR / Cache / Histórico / Downgrade / Mirrors | linha acima | `PACKAGES` / `USERS` / `DATABASE` / `HISTORY` / `DOWNGRADE` / `NETWORK` (2026-10-25) |
| packages > origem (D55) | Instalar / Remover / Atualizar; Órfãos no topo de Remover | — | `ADD` / `DELETE` / `UPDATE`; `CLEAN` |
| packages > detalhes | cabeçalhos | `PACKAGES`, `LINK`, `SETTINGS` | sem ícone (títulos de seção) (Fase 3) |
| packages > ações | Instalar / Reinstalar / Remover / Remover marcados / Atualizar tudo / Atualizar banco / limpezas ×3 / Arquivos em cache / Configurar mirrors | — | `ADD` / `RESTART` / `DELETE` / `DELETE` / `UPDATE` / `SYNC` / `CLEAN` (mesmo sentido) / `FOLDER` / `EDIT` (Fase 3) |
| packages > editor de mirrors | País / Protocolo / Idade máxima / Quantidade / Ordenação / Gerar preview | — | `EARTH` / `LINK` / `CLOCK` / `COUNTER` / `SORT` / `VISIBLE` (Fase 3; `COUNTER` e `SORT` novos) |
| packages > listas | pacotes, atualizações, órfãos, arquivos de cache, histórico, mirrors | — | sem ícone (listas homogêneas, D9) (Fase 3) |
| power | Tampa (bateria) / Tampa (AC) / Botão / Tela desligada ×2 / Bloquear ×2 / Manter acordado | `MONITOR` / `ETHERNET` / `POWER` / `MONITOR` ×2 / `LOCK` ×2 / `MONITOR` | `BATTERY` / `power_plug` `f06a5` (novo) / `POWER` / `MONITOR` / `LOCK` / `SLEEP` (mesmo ícone em bateria e AC: mesmo sentido) |
| services > Home | Sistema / Usuário / Falhos / Logs | `SETTINGS` / `USER` / `WARNING` / `LOGS` | `SERVICES` / `USER` / `WARNING` / `LOGS` |
| session > Home | Componentes / Autostart / Diagnóstico / Logs | `APPS` / `BOOT` / `DIAGNOSTICS` / `LOGS` | `APPS` / `AUTOSTART` / `DIAGNOSTICS` / `LOGS` |
| settings > Localidade | Fuso / Data e hora / Locale regional / Locales do sistema / Teclado | `NETWORK` / `HISTORY` / `NETWORK` / `APPS` / `KEYBOARD` | `CLOCK` / `CALENDAR` / `EARTH` / `TRANSLATE` / `KEYBOARD` |
| settings > Data e hora | Local / Fuso / NTP / RTC | `HISTORY` / `NETWORK` / `SATELLITE` / `BATTERY` | `CALENDAR` / `CLOCK` / `SATELLITE` / sem ícone (Info) |
| settings > Teclado | Layout / Variante / Modelo / Opções / Console | `KEYBOARD` / `FONTS` / `MOUSE` / `SETTINGS` / `MONITOR` | `KEYBOARD` / `KEYBOARD_VARIANT` (Fase 3) / sem ícone / sem ícone / `TERMINAL` |
| settings > Sistema | Hostname / Usuários / Grupos / Não perturbe | `MONITOR` / `USER` / `USERS` / `BELL(_OFF)` | `MONITOR` / `USER` / `GROUP` / `BELL_OFF` fixo (o estado fica no `[x]`) (Fase 3) |
| settings > Apps padrão | Navegador / Launcher / Terminal | `NETWORK` / `BOOT` / `MONITOR` | `EARTH` / `LAUNCHER` / `TERMINAL` |
| settings > Fontes | Taskbar / Sysinfo / Control Panel / Sistema / Apps / Terminal / Navegador | `MONITOR` / `DIAGNOSTICS` / `SETTINGS` / `SERVICES` / `PACKAGES` / `KEYBOARD` / `NETWORK` | `TASKBAR` / `DIAGNOSTICS` / `CONTROL_PANEL` / `SETTINGS` / `APPS` / `TERMINAL` / `EARTH` |
| settings > Fontes | Antialiasing / Hinting / Subpixel / DPI | `SUCCESS` / `SEARCH` / `PALETTE` / `STORAGE` | `EFFECT` / `RULER` / `PALETTE` / `MONITOR` (→ D9) |
| settings > Usuários | Criar / Lista / Contas do sistema | — | `ADD` / `USERS` / `ACCOUNT_COG` (Fase 3) |
| settings > Usuário | Nome completo / Shell / Grupo principal / Grupos suplementares | — (botões sem ícone) | `ID_CARD` / `TERMINAL` / `ACCOUNT_STAR` / `GROUP` (Fase 3) |
| settings > Usuário | Alterar senha / Bloquear / Desbloquear / Exigir troca / Imagem do avatar / Remover avatar / Excluir (manter home) / Excluir e home | — | `KEY` / `LOCK` / `LOCK_OPEN` / `LOCK_RESET` / `AVATAR` / `IMAGE_REMOVE` / `ACCOUNT_REMOVE` / `DELETE_FOREVER` (Fase 3) |
| settings > Firewall | Serviço / Iniciar no boot / Cancelar alterações / Adicionar regras / Aplicar regras salvas | — | `SHIELD` / `AUTOSTART` / `CANCEL` / `SCRIPT` / `SHIELD_REFRESH` (Fase 3) |
| settings > Atalhos | Alterar / Desativar / Restaurar padrão; Restaurar todos | — | `EDIT` / `KEYBOARD_OFF` / `RESTORE`; `RESTORE` (Fase 3) || storage > Home | Resumo / Discos / Partições / Sistemas de arquivos / Pontos de montagem / SMART / Uso | `STORAGE` ×5, `LOCK`, `STORAGE` | `INFO` / `STORAGE` / `LAYOUT` / `FOLDER` / `LINK` / `DIAGNOSTICS` / `DIAGNOSTICS` (SMART e Uso repetem: → D9) |
| storage | `disk_icon`, `partition_icon` (SWAP = `REFRESH`, cifrado = `LOCK`) | — | SWAP = `MEMORY`; manter `LOCK` |

Itens marcados com "→ D9" têm mais de uma proposta razoável; a escolha final
fica para a Fase 1 com revisão visual.

---

## 2. Idiomas por crate

| Crate | Navegação | Botões | Confirmação | Marcadores | Rodapé |
| --- | --- | --- | --- | --- | --- |
| `argvus-control-center` (Home) | Grade de cartões; `Tab` = próximo item; mouse só aqui | — | — | `>` | chave i18n por estado |
| `config_app` | Lista; `←→` ajustam valor | — | — | `>`, `[x]` | 1 chave |
| `about` | Abas (`Tab`), rolagem | — | — | — | 2 chaves |
| `appearance` | `Vec<String>` + `match self.selected`; `Tab` para botão | `Button::new(Apply)` em 9 tipos de página | Páginas-lista Delete/Cancel e Replace/Cancel | `[x]`, ` · atual`, ` · valor`, `›`, ` >`, ` > N%` | 4 chaves + 4 hints em inglês fixo |
| `settings` | Lista própria (`ui/list.rs`), `row_selectable`, botões depois das linhas, `Tab` cicla | `argvus_tui::buttons` + linhas `[ ... ]` | Popup próprio `draw_confirm`; popup de conflito (`r`/`Esc`) | `[x]`, `●`/current, detalhe à direita | chaves por página |
| `boot` | `Selection` + `page::list`; `Tab` | `ActionButton` ×5 | `ConfirmationState` | `>`, ` · ` | chaves por página |
| `packages` | `Selection`; digitação filtra | `ActionButton` ×10 | `ConfirmationState` | `>`, `[x]` (multi) | chaves por página |
| `network` | `Selection`; `/` filtro | `ActionButton` ×7 | `ConfirmationState` (`confirm_forget`) | `>`, `●` conectado | chaves por página |
| `audio` | `Selection` | `ActionButton` ×5 | `ConfirmationState` | `>`, `●` padrão | chaves por página |
| `bluetooth` | `Selection` | `ActionButton` ×7 | `ConfirmationState` (`confirm_remove`) | `>` | chaves por página |
| `hardware` | `Selection` | `ActionButton` ×2 | `ConfirmationState` | `>`, `★ ativo` | chaves com `g`/`e` |
| `services` | índice `usize` próprio; `/` busca | `ActionButton` ×7 (rótulos dinâmicos `X: valor`) | `ConfirmationState` | `>` | chaves por página |
| `displays` | índice próprio; Picker e Prompt | `DisplayButton` ×9 | `ConfirmationState` (perfil) + reversão com prazo (`y`/`n`) | `>`, `●` | chaves por página |
| `power` | índice próprio; picker | `PowerButton` ×3 | `ConfirmationState` | `>` | chaves por página |
| `session` | índice próprio | `SessionButton` ×5 | `ConfirmationState` | `>` | chaves por página |
| `storage` | `Selection` + `readonly` | — | — | `>` | 2 chaves |
| `diagnostics` | `Selection` + `readonly` | — | — | `>`, ok/aviso/erro | 2 chaves |

Inconsistências de teclado encontradas:

- `r` = **atualizar** em quase todo lugar, mas = **restaurar padrões**
  (`reset_current`, com confirmação) no `settings` e = **cancelar** no Picker do
  `displays`.
- `←/→`: voltar/abrir em listas; ajustar valor no `config_app`, editores de
  efeito do `appearance` e Mouse e touchpad; navegar botões quando o foco está
  na barra.
- `Home/End`: navegar, exceto na linha de valor de SurfaceSection (define 0/100).
- `Space`: toggle no settings/appearance; ativar botão nos demais.
- Atalhos de página únicos: `e`/`i`/`d` (temas), `g` (governor), `e`
  (perfil de energia e captura de atalho), `+`/`-`/`=` (fontes e efeitos).

---

## 3. Linhas Info hoje selecionáveis

Linhas que recebem o marcador `>`/realce sem ter ação (ou que o cursor
alcança sem efeito). "Realce sem cursor" = `row_count` 0 mas `page::list`
desenha a linha 0 selecionada.

| Crate > tela | Linhas | Sintoma |
| --- | --- | --- |
| network > DNS | cabeçalho "DNS CONFIGURATION" e valores | realce sem cursor na linha 0 |
| network > Proxy | cabeçalho "PROXY CONFIGURATION" e valores | realce sem cursor |
| boot > Kernel/Bootloader/Initramfs detalhe | todas (cabeçalho + campos) | `selection_len` = 1, linha 0 realçada |
| audio > Resumo | cabeçalho "AUDIO SYSTEM" + campos | `row_count` = 1, linha 0 realçada |
| audio > Dispositivos | lista de dispositivos | navegável, Enter sem efeito |
| bluetooth > Estado | 2 linhas (estado do adaptador) | navegáveis, Enter sem efeito (ações só nos botões) |
| hardware > GPU / Dispositivos | itens | ok (abrem detalhe) |
| services > LogDetail | conteúdo | `selection_len` = 1 |
| session > Diagnóstico | itens de diagnóstico | navegáveis, Enter sem efeito |
| session > LogDetail | conteúdo | `selection_len` = 1 |
| packages > Mirrors | lista (readonly) | ok (readonly); Enter abre editor sem linha de ação visível |
| displays > Home | configs antigas (stale) | navegáveis; abrem detalhe com `Remove config` (ok) |
| diagnostics > Resumo / detalhe | readonly | ok |
| appearance > Bordas gerais | Arredondamento com Arredondado desligado | selecionável e editável sem efeito visual; deve ficar desabilitada |
| appearance > Home | Tema/Destaque/Wallpaper mostram valor | ok (são Submenu com valor) |
| settings > Data e hora | Fuso (Info) e RTC | já não selecionáveis (ok) |
| settings > Teclado | Modelo, Opções | já não selecionáveis (ok) |
| settings > Usuário / Grupo | cabeçalhos "Account", "Identity", "Information", "Group", "Members" | já não selecionáveis (ok) |
| settings > Firewall | "Service", "Active/Stopped", "Start at boot" | selecionáveis (`row_selectable` retorna `true`) |
| settings > Captura de atalho | "Pressione o novo atalho" | já não selecionável (ok) |

---

## 4. Proposta de `RowKind` e API compartilhada (`argvus-tui`)

Local: novo módulo `argvus-tui/src/menu.rs` (lista + linha), `confirm.rs`
(substitui `components::ConfirmationState`), `hints.rs` (rodapé). `buttons.rs`
fica marcado para remoção na Fase 4, quando nenhum crate o usar.

```rust
/// Identidade estável de um item; substitui índices numéricos.
pub trait RowId: Copy + Eq {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKind {
  /// Rótulo + valor, nunca recebe foco.
  Info,
  /// Enter executa.
  Action,
  /// Enter abre outra página (marcador `›`).
  Submenu,
  /// Enter/Space alterna; o momento do efeito é decidido pela página.
  Toggle { on: bool },
  /// Opção exclusiva; `current` desenha `●`.
  Choice { current: bool },
  /// Número ou texto; Enter edita, `←/→` ajusta quando `step` existe.
  Value { text: String, step: Option<Step> },
  /// Ação que sempre passa pelo componente de confirmação.
  Destructive,
  /// Linha separadora visual (não selecionável).
  Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step { pub delta: i32, pub min: i32, pub max: i32 }

#[derive(Debug, Clone)]
pub struct Row<Id> {
  pub id: Id,
  pub kind: RowKind,
  /// Glyph do catálogo (`icons::*`); resolvido por `AppConfig::icon` ao desenhar.
  pub icon: Option<&'static str>,
  pub label: String,
  /// Valor à direita (Info/Submenu/Value).
  pub detail: Option<String>,
  /// Desabilitada: desenhada em `muted` e pulada pelo cursor.
  pub enabled: bool,
  /// Ênfase de tema (Primary para `Apply`, Danger para destrutivas).
  pub emphasis: Emphasis,
}

/// Cursor que só para em linhas selecionáveis.
#[derive(Debug, Clone, Copy, Default)]
pub struct MenuState { selected: Option<usize>, offset: usize }

impl MenuState {
  pub fn selectable(row: &Row<impl Copy>) -> bool; // !Info && !Separator && enabled
  pub fn normalize<Id>(&mut self, rows: &[Row<Id>]);  // primeira selecionável ou None
  pub fn handle<Id>(&mut self, key: KeyCode, rows: &[Row<Id>], page: usize) -> MenuEvent<Id>;
  pub fn selected_id<Id: Copy>(&self, rows: &[Row<Id>]) -> Option<Id>;
}

/// O que a página deve fazer; a página mantém a regra de negócio.
pub enum MenuEvent<Id> {
  None,
  Moved,
  Activate(Id),        // Enter/→ em Action, Submenu, Choice, Value (abrir edição)
  Toggle(Id),          // Space/Enter em Toggle
  Adjust(Id, i32),     // ←/→ em Value com step
  Confirm(Id),         // Enter em Destructive → página abre ConfirmState
  Back,                // Esc/←
}

pub fn draw_menu<Id>(frame: &mut Frame, area: Rect, theme: &Theme, rows: &[Row<Id>], state: &MenuState);

/// Componente único de confirmação.
pub struct ConfirmState { confirm_selected: bool }
pub enum ConfirmOutcome { Pending, Confirmed, Cancelled }
impl ConfirmState {
  pub fn handle(&mut self, key: KeyCode) -> ConfirmOutcome; // ↑↓/Tab/←→ alternam; Enter; Esc; y/n
}
pub struct ConfirmDialog<'a> { pub title: &'a str, pub message: &'a str, pub confirm: &'a str, pub cancel: &'a str, pub danger: bool, pub deadline: Option<Duration> }
pub fn draw_confirm(frame: &mut Frame, area: Rect, theme: &Theme, dialog: ConfirmDialog<'_>, state: &ConfirmState);

/// Rodapé derivado do tipo da linha selecionada + extras da página.
pub struct HintContext<'a> { pub row: Option<&'a RowKind>, pub can_go_back: bool, pub search: bool, pub refresh: bool, pub extra: &'a [(&'a str, &'a str)] }
pub fn hints(lang: Lang, ctx: HintContext<'_>) -> String;
```

Notas de projeto:

- **Ícone pertence ao item**: cada crate expõe `fn icon(id) -> &'static str`
  (ou preenche `Row::icon`) a partir de um enum de itens; `home_icon_for_item`
  e `home_icon_for_header` passam a usar `HomeItem`/`HomeCategory`.
- **Marcadores**: coluna própria antes do ícone: `●` (Choice atual),
  `[x]`/`[ ]` (Toggle; ver D1), `›` à direita (Submenu). Remove ` · atual`,
  `★ ativo`, ` >`, ` > N%`.
- **Apply**: helper `draft_actions(dirty: bool, lang) -> Vec<Row<Id>>` gera
  `Separator` + `Apply` (desabilitada se `!dirty`) e, quando aprovado, `Cancel`.
- **Estado no `settings`**: `row_selectable` e `page_buttons` são substituídos
  pelo `RowKind`; `Row { label, detail, current }` do settings migra para
  `argvus_tui::menu::Row`.
- **Testes da Fase 1** (no `argvus-tui`): pular Info/Separator/desabilitadas em
  `↑↓ jk Home End PgUp PgDn`; lista vazia; página só com Info (sem cursor,
  rolagem); seleção inicial; `Adjust` só em Value com step; `y`/`n` na
  confirmação; rodapé por tipo de linha.

---

## 5. Progresso

| Fase | Item | Status |
| --- | --- | --- |
| 0 | Auditoria (`docs/ux-audit.md`) | concluída (2026-10-04); decisões em 6.1 |
| 1 | Catálogo: corrigir codepoints (1.1) e adicionar glyphs (1.2) | concluída (`argvus-tui` `29808a3`) |
| 1 | `RowKind`, `Row`, `MenuState`, `draw_menu` em `argvus-tui` | concluída (`argvus-tui` `e2fd513`) |
| 1 | Componente único de confirmação (`y`/`n`) | concluída (`argvus-tui` `1d3459d`) |
| 1 | Rodapé contextual (`hints`) | concluída (`argvus-tui` `bee7e32`, `argvus-i18n` `0c2d823`) |
| 1 | Testes de cursor e confirmação | concluída (junto com cada componente) |
| 1 | Campo de texto ativo captura `q`/`?` (D10) | concluída (`argvus-control-center` `5b346ca`) |
| 2 | `appearance` | concluída (`2f57d07`, `75d3dc1`, `4737a4c`; `argvus-i18n` `a62346d`) |
| 3 | `settings` | concluída (`2f694e3`, `c423f33`, `14e51c1`; `argvus-tui` `0ee7a52`, `908e747`; `argvus-i18n` `c49a792`) |
| 3 | `displays` | concluída (`545bcfe`, `c394639`; `argvus-tui` `106013b`; `argvus-i18n` `bb784d3`) |
| 3 | `boot` | concluída (`93bc44f`, `1d04196`, `a82ada7`; `argvus-i18n` `2e6be4e`) |
| 3 | `packages` | concluída (`33a033f`, `63689aa`; `argvus-tui` `6beda24`; `argvus-i18n` `ef69e70`) |
| 3 | `network` | concluída (`83a0461`, `d5df88d`; `argvus-tui` `541d788`; `argvus-i18n` `b46d99b`) |
| 3 | `network` > Conexões (editor de perfis no estilo nmtui, D68–D72; ver 5.12) | concluída, sem commit. 84 testes da crate; validação real com `nmcli` em perfis descartáveis (criados e apagados, nunca ativados). Teste interativo em terminal real pendente |
| 3 | `services` | parcial, sem commit: listas Sistema/Usuário/Falhos e Logs no layout Filter / Search / bloco. Linha `Filter:` fora do menu com o valor à direita (`f` cicla; não é mais linha selecionável); caixa `Search:` com borda, destacada enquanto recebe texto (`/` busca nas unidades e nos logs); unidades no bloco `Services:` com bolinhas Active/Enabled; o cabeçalho `Name:` saiu. Logs: a linha `Filter:` resume Boot/Service/Priority, e `b`, `u` e `p` alteram cada um; a barra de botões saiu. `p` corrigido: a partir de "0-7" não ficava preso. Detalhe da unidade como menu: bloco "Service" com informações somente leitura e bloco "Actions" com Start/Stop/Restart/Enable/Disable/Enable now/Disable now/Logs; filtro volta a All ao sair da lista. Pendente: decisão "ações no topo da lista" (sem alvo com um cursor só); rótulo "View Logs" com a chave `control_center.view_logs` em argvus-i18n |
| 3 | `audio` | migrada, sem commit (barra de botões removida; teclas no rodapé; `argvus-i18n` com a chave `navigate_volume_enter_set_default_r_refresh_esc_back_help`; `[Value]` em `v`, a confirmar) |
| 3 | `bluetooth` | pendente |
| 3 | `power` | pendente |
| 3 | `storage` | pendente |
| 3 | `hardware` | pendente |
| 3 | `diagnostics` | pendente |
| 3 | `session` | pendente |
| 3 | `about` | pendente |
| 3 | `projects` (página em Applications, padronizada com o layout de Window rules: seção Add no topo, um bloco por pasta configurada com Remove próprio e separador; `argvus projects` passou a recusar adicionar além do limite de 9 projetos, já que `SUPER + ALT + 1..9` só endereça essa faixa) | concluída, sem commit; verificação visual em 80x24 pendente |
| 3 | `ssh-keys` (página nova em Applications, sem tela antiga para migrar; já no modelo novo: seções Gerar, Chaves e Agente, sem barra de botões, Atualizar e Descarregar todas como linhas de ação, Apagar na Zona de perigo com confirmação e foco em Cancelar; backend `argvus keys`, sem passphrase em argv). Carregar abre um terminal ARGVUS para a senha. Sem atalho novo, então o cheatsheet não muda. Teste de renderização em 80x24 pendente | concluída, sem commit; renderização real pendente |
| 3 | `gpg-keys` (página em Applications, sem tela antiga para migrar). Gerar chave e Importar são seções colapsáveis (expandidas no início, `Enter` alterna); Chaves › abre o chaveiro com a busca no topo e, por chave, Editar ›, exportar pública e privada, copiar chave e fingerprint, excluir. Editar › tem expiração, senha (pinentry), assinar, revogar, confiança e servidor de chaves. Importação pública e privada usa o gerenciador de arquivos (`pick_file` no core), com campo de caminho quando o gerenciador é gráfico. Pinentry fixo em pinentry-gtk, só informativo. A senha do formulário vai ao launcher pela entrada padrão. Pendente: buscar chave no servidor por nome ou e-mail; renderização em 80x24 pendente. | concluída parcialmente, sem commit |
| 3 | Home: catálogo em `home/catalog.rs` (`HomeDestination` com IDs estáveis, ícone e cartão por destino), taxonomia do esboço `TUI_home.png` (seis cartões, ver 5.9) e breadcrumbs/busca alinhados aos cartões. Testes de catálogo e 763 testes do workspace passando (com `ARGVUS_I18N_DIR` apontando para o catálogo do repositório) | concluída, sem commit; renderização real em 80x24 pendente |
| 4 | Chaves i18n órfãs (en-US e pt-BR) | pendente |
| 4 | Código morto (páginas `Transparency`/`Blur*`, `argvus_tui::buttons`) | pendente |
| 4 | README "Keyboard controls" e entrada no CHANGELOG | pendente |

### 5.1 Notas da Fase 1 (2026-10-04)

API entregue em `argvus-tui` (nada no Control Center usa ainda; as telas
migram nas Fases 2 e 3):

- `menu::{RowKind, Row, Emphasis, draft_actions, MenuState, MenuEvent,
  MenuStyle, draw_menu}`. `Row::value(id, label, text, step)` guarda o valor
  em `detail`; `MenuState::select(rows, &id)` restaura a seleção por id.
- `confirm::{ConfirmState, ConfirmOutcome, ConfirmDialog, draw_confirm}`. O
  `components::ConfirmationState` antigo continua até a migração dos crates.
- `hints::{HintContext, hint_keys, hints, confirm_hint_keys, confirm_hints}`,
  com chaves `control_center.hint.*` e `control_center.confirm` (en-US e
  pt-BR) no `argvus-i18n`.
- Catálogo `icons`: das 15 constantes da seção 1.1, 12 tiveram o codepoint
  corrigido; `WIFI`, `GPU` e `PACKAGES` mantiveram o glyph (variantes de mesmo
  sentido) e o comentário passou a nomear o glyph real. 56 constantes novas
  (seção 1.2 mais `POWER_PLUG`), todas conferidas na fonte instalada.

Telas que mudam de comportamento com o D10 (`q`/`?` passam a ser texto):

| Crate | Tela / estado | Antes |
| --- | --- | --- |
| `packages` | Instalar (oficial), AUR, Instalados, Órfãos, Atualizações: listas que filtram ao digitar | `q` saía, `?` abria a ajuda |
| `packages` | Prompt de entrada (`input`) | idem |
| `settings` | Busca ativa (`/`) em qualquer página | idem |
| `settings` | Edição de hostname | idem |
| `settings` | Captura de atalho de teclado | idem (`Super+Q` também saía) |
| `network` | Senha Wi-Fi, DNS manual, filtro de lista (`/`) | idem |
| `network` | Firewall embutido: editor de administração | idem (só a rota Settings era protegida) |
| `services` | Busca de unidades e de logs | idem |
| `audio` | Campo de valor do volume | idem |
| `boot` | Campo de timeout / linha de comando do kernel | idem |
| `displays` | Prompts (nome de perfil, valores) | idem |
| `appearance` | Prompts (nome de tema, caminho, espaços, bordas) e editor HEX do destaque | idem |

Sem mudança: Home (a busca global já era protegida), editor de
administração na rota Settings (já protegido), e todas as listas que não
filtram ao digitar. Os prompts de texto/número foram incluídos como "campo de
texto ativo" segundo o registro do D10 em 6.1; confirmar se o ajuste "só
filtro ou busca" pretendia excluí-los.

Achados fora do escopo, sem alteração:

- `Ctrl+C` não sai do app enquanto a busca global da Home está ativa (o ramo
  da busca retorna antes); comportamento anterior à Fase 1.
- `boot`: a edição de "Kernel command line" (GRUB) reutiliza `timeout_input`,
  cujo tratamento de teclas só aceita dígitos até 2 caracteres; com o valor
  atual carregado, só é possível apagar, não digitar. Tratar na migração do
  `boot` (Fase 3). **Corrigido em `93bc44f` (D30).**
- O cspell não está instalado nesta máquina; `docs/ux-audit.md` já está no
  `ignorePaths`.

### 5.2 Notas da Fase 2 (2026-10-04)

Dependências: a Fase 2 compila contra `argvus-tui` e `argvus-i18n` na
branch `ux_ui` (path dependencies). Nada disso está em `main`: o
`packaging/arch/remote/PKGBUILD` baixa `main` dos dois repositórios e só
compila depois da mesclagem; em execução, as chaves `control_center.hint.*`,
`confirm`, `discard*`, `draft_changed`, `export` e `import` exigem o
`argvus-i18n` instalado a partir de `ux_ui` (sem elas o rodapé mostra a
chave).

Mudanças de comportamento, além da apresentação:

- `Esc` que descartaria um rascunho pede confirmação; `r` e o recarregamento
  automático não descartam mais um rascunho com alterações (D15).
- `←` em linhas Value com passo ajusta em vez de voltar (D3); `Home/End`
  navegam e o 0/100 direto passou para o prompt do Enter (D14).
- Correções encontradas na migração: Control Panel > Sessões alternava o
  card errado quando havia card indisponível (o cursor indexava
  `ControlPanelCard::ALL`, a lista mostrava só os disponíveis); o editor de
  Blur global não limpava o valor não aplicado ao voltar, e o valor
  reaparecia na entrada seguinte; o editor de efeito aceitava valor
  negativo com `-`/`h` repetidos (`saturating_sub` sem limite inferior).

Achados para a Fase 4 (sem alteração):

- Páginas mortas `Transparency`, `TransparencySurface` (exceto Launchers) e
  `BlurSurface`, além da seção `SurfaceSection::Blur` (nenhuma linha leva a
  ela). `Transparency` passou a mostrar só linhas Info, porque nunca teve
  ação.
- Campo `control_panel_draft`: é escrito e nunca lido.
- Chaves i18n que o `appearance` deixou de usar:
  `theme_profile_themes_help`, `theme_profile_import_help`,
  `theme_profile_delete_help`, `theme_profile_duplicate_help`,
  `jk_navigate_enter_apply_r_refresh_esc_back`,
  `jk_navigate_enter_open_space_toggle_r_refresh_esc_back`,
  `navigate_tab_actions_adjust_enter_activate_esc_back_help`,
  `navigate_tab_actions_move_enter_activate_esc_back_help`,
  `navigate_tab_actions_move_enter_activate_r_refresh_esc_back_help`.
  Conferir uso nos outros crates antes de remover.

### 5.3 Notas da Fase 3: `settings` (2026-10-04)

Dependências: como na Fase 2, compila contra `argvus-tui` e `argvus-i18n`
na branch `ux_ui`. Em execução, os títulos de seção (`section_*`,
`danger_zone`), `shell`, `home_directory`, `uid_gid`, `gid`, `groups_label`,
`font_size`, `discard_config_changes`, `replace`, `set_default`,
`apply_saved_rules_description` e `restore_shortcut_*` exigem o
`argvus-i18n` instalado a partir de `ux_ui`; sem elas a tela mostra a chave.

Componentes novos no `argvus-tui`: `Row::section(título)` (separador com
título na cor de destaque, sem `--`, com valor opcional à direita) e 16
glyphs no catálogo (`KEY`, `LOCK_OPEN`, `LOCK_RESET`, `AVATAR`,
`IMAGE_REMOVE`, `ACCOUNT_REMOVE`, `DELETE_FOREVER`, `ACCOUNT_COG`,
`ACCOUNT_STAR`, `GROUP`, `ID_CARD`, `KEYBOARD_VARIANT`, `KEYBOARD_OFF`,
`FONT_SIZE`, `SCRIPT`, `SHIELD_REFRESH`), todos conferidos na fonte.

Mudanças de comportamento, além da apresentação:

- `Tab`/`BackTab` não fazem mais nada no `settings` (não há abas); `←/→`
  não navegam mais entre botões.
- Locales do sistema: Enter numa linha alterna o locale (antes aplicava de
  qualquer linha); `Aplicar` fica na barra Ações embaixo da lista, apagada sem
  mudança e confirmada (D20, revisado).
- Fontes: o Tamanho é a primeira linha do seletor (Value), com prompt 8–32
  no Enter (D21).
- Sistema > Hostname abre o popup direto (D21).
- Idioma: sem bandeiras emoji nem "· em uso"; o `●` marca o atual.
- Shell e Grupo principal usam `●`; Grupos suplementares, Membros e
  Layouts usam a coluna `[x]`.
- `Salvar`/`Criar`/`Aplicar` ficam apagados sem mudança; `Esc` com
  rascunho pergunta antes de descartar e o descarte volta ao estado
  carregado; o snapshot não troca um rascunho alterado (D15).
- `r` na lista e na edição de atalhos restaura o atalho (só o
  selecionado/editado) depois da confirmação (D19); `r` também recarrega
  as listas de usuários e grupos.
- Confirmações usam o componente único (`y`/`n`, `↑↓/jk/Tab`); exclusões
  de usuário e grupo e as restaurações de padrões usam o estilo de perigo.
- O conflito de atalho virou o componente de confirmação; `Esc` na rota
  principal agora cancela o conflito em vez de voltar de página por trás
  do popup (`argvus-control-center/src/event.rs`).

Correções encontradas na migração:

- `Space` na lista de atalhos alternava o último atalho **editado**
  (`keybinding_edit_id` nunca era limpo) em vez do selecionado.
- A busca da lista de contas do sistema sempre mantinha o primeiro usuário
  (o filtro preservava o índice 0, que só era o `Reload` na lista de grupos).
- Itens de listas filtradas (fusos, fontes, apps, layouts, usuários,
  grupos) eram resolvidos recontando as linhas visíveis; agora cada linha
  carrega o índice da lista original.

Removido: `ui/list.rs` (lista própria), `ui/buttons.rs`, o popup
`draw_keybinding_editor` e as linhas `keybinding_picker_rows`/
`select_keybinding_picker`, todos sem chamador.

Verificação visual: a 80x24 (tmux), Fontes e Sistema renderizam com
seções, ícones, `›` e Zona de perigo no fim. Neste ambiente o tmux não
entrega teclas ao app (nem à Home, que não mudou), então a navegação foi
validada só pelos testes.

Achados para a Fase 4 (sem alteração):

- Chaves i18n que o `settings` deixou de usar:
  `navigate_enter_open_help_q_quit`,
  `navigate_enter_open_tab_actions_esc_back_help`,
  `navigate_tab_actions_move_enter_activate_search_esc_back_help`,
  `navigate_enter_apply_size_search_tab_actions_esc_back_help`,
  `navigate_space_toggle_search_enter_apply_esc_back`, `keybindings_help`,
  `keybindings_editor_actions`, `keybindings_capture_help`,
  `keybindings_editor_navigation`, `keybindings_choose_replace`,
  `navigate_space_enable_enter_default_search_esc_back_help`,
  `navigate_enter_apply_search_esc_back_help`, `enter_edit_esc_back_help`,
  `navigate_enter_apply_esc_back_help`,
  `navigate_tab_actions_move_enter_activate_esc_back_help`,
  `navigate_search_enter_open_esc_back_help`,
  `fields_tab_switch_actions_enter_activate_esc_back_help`,
  `navigate_enter_open_esc_back_help`, `enter_confirm_esc_cancel`,
  `cancel_f8378f`, `identity`, `information`, `in_use`. Nenhuma é usada
  por outro crate (conferido com `git grep`), exceto a asserção de teste de
  `cancel_f8378f`.
- `App::keybinding_editor_state` ficou sem chamador.
- Mouse e touchpad: as linhas seguem sem ícone, como antes (D9 provisório).

### 5.4 Notas da Fase 3: `displays` (2026-10-04)

Dependências: como nas fases anteriores, `argvus-tui` e `argvus-i18n` na
branch `ux_ui`; em execução, as chaves `keep`, `revert`,
`keep_configuration_*`, `section_actions`, `remove_config_description` e
`profile_monitors` exigem o `argvus-i18n` instalado a partir de `ux_ui`.

Reversão (conferida no código antigo e coberta por testes): o prazo continua
15 s (`REVERT_SECONDS`) contados de quando a mudança é armada; sem resposta,
`poll` aplica o `previous_config` armado com `apply_all` (salva
`monitors.lua` e roda `hyprctl reload`) e mostra "Revertido <nome>", como
antes. A detecção de hotplug continua suspensa durante a contagem. As
decisões foram extraídas em `arm_revert`, `take_expired_revert` e
`answer_revert`, testadas sem disco nem `hyprctl`.

Mudanças de comportamento, além da apresentação:

- Reversão pelo componente único (D24): foco em Reverter; `Enter` logo
  após a mudança reverte (antes mantinha), `Space` não mantém mais; `y`
  mantém; `n`/`Esc` revertem; o rodapé começa por `y Manter`.
- `Remover configuração` pede confirmação (antes era imediata).
- A linha Perfis aparece sempre na Home.
- Enter num perfil abre a página do perfil (D4).
- O seletor marca o valor em vigor com `●` (sem sufixos "atual"/"primário");
  workspaces são Toggle, ainda imediatos e voltando ao detalhe.
- Enquanto um snapshot ou ação roda, o cursor pode se mover (antes ficava
  parado); ativar continua bloqueado.

Correção feita na migração: abrir uma configuração antiga (monitor ausente)
abria sempre a primeira delas (`Detail(stale_start())`); agora abre a
escolhida. O texto fixo em português "escala" da Home passou a ser
traduzido.

Bugs encontrados, **sem alteração** (tarefa separada):

- Orientação: as opções usam `PersistChange::None` e o `args` é ignorado;
  escolher uma orientação não salva nem aplica nada.
- Presets de brilho e saturação SDR: também `PersistChange::None`, sem
  efeito; só o valor personalizado funciona.
- `Monitor::info_rows` (`model.rs`) usa rótulos fixos em português
  ("Fabricante", "Modelo"...), sem i18n.

Achados para a Fase 4 (sem alteração): chaves que o `displays` deixou de
usar, sem uso em outro crate: `navigate_enter_apply_r_cancel_help`,
`type_value_enter_confirm_esc_cancel`,
`navigate_tab_actions_enter_new_r_reload_esc_back_help`,
`navigate_tab_actions_enter_open_r_refresh_esc_back_help`,
`keep_this_configuration_enter_keep`,
`reverting_automatically_if_no_key_is_pressed`, `current_4cdf18`,
`primary`, `new`.

### 5.5 Notas da Fase 3: `boot` (2026-10-05)

Dependências: como nas fases anteriores, `argvus-tui` e `argvus-i18n` na
branch `ux_ui`. Em execução, os títulos de seção (`section_base_system`,
`section_components`, `section_entries`, `section_entry`,
`section_files`, `section_presets`), os rótulos do Resumo e do initramfs
(`firmware`, `boot_manager`, `default_entry`, `esp_path`,
`initramfs_modules`/`_binaries`/`_files`/`_hooks`), `regenerate_grub` e
os motivos `unavailable_no_systemd_boot_entry`/
`unavailable_unknown_bootloader` exigem o `argvus-i18n` instalado a
partir de `ux_ui`; sem elas a tela mostra a chave (visto na verificação
em tmux). Os rótulos que já existiam com dois-pontos (`package`,
`version_20bc85`, `status`, `image`...) são reaproveitados sem o `:`.

Conferido no código antigo e preservado: as sete ações (padrão systemd-boot,
timeout systemd-boot/GRUB, linha do kernel, regenerar GRUB, regenerar
initramfs, tema Plymouth) continuam passando pela confirmação, com
efeito imediato depois dela, elevadas por `pkexec` e com o painel de
saída ao vivo (rolagem, follow, `Esc` fecha) sem mudança. As mensagens
de confirmação (`action_message`) são as mesmas.

Mudanças de comportamento, além da apresentação:

- Sem barra de botões; `Tab`/`BackTab` não fazem nada no `boot` e `←/→`
  não movem mais foco entre botões.
- `Definir padrão` saiu da lista de Kernels e do Bootloader: está na
  página do kernel e na página da entrada.
- Timeout, Linha do kernel e Regenerar GRUB ficam só na página
  Bootloader (antes também apareciam nos botões do detalhe da entrada,
  onde o timeout é o mesmo e a linha do kernel e o GRUB nunca existiam
  juntos com entradas) (D28).
- Linhas inválidas no momento ficam desabilitadas, com o motivo à
  direita, em vez de falhar depois de ativadas (D28).
- `Esc`/`←` voltam com o cursor no item que abriu a página (antes, na
  primeira linha).
- Enquanto um recarregamento ou uma ação roda, as páginas continuam
  abrindo, mas nenhuma mudança de boot começa. Antes, `Enter` na lista do
  initramfs ou do Plymouth podia abrir uma segunda confirmação e trocar
  o job em andamento.
- Confirmação pelo componente único: foco em Cancelar (como antes),
  `y`/`n`; as duas regenerações em estilo de perigo.
- O Resumo é só Info, sem cursor (antes a página era `readonly`); o
  `★` dos kernels e do Plymouth virou valor "Atual" e `●`.

Correção feita: o campo "Linha do kernel" aceitava só 2 dígitos (D30).

Achados, **sem alteração**:

- A lista de Kernels mostra o pacote duas vezes ("6.18.49-2-lts
  6.18.49-2-lts"): `detect_kernels` preenche `version` com o mesmo nome
  do diretório de módulos. Formato anterior à Fase 3.
- O backend recusa a linha do kernel vazia ("invalid GRUB kernel command
  line"); a UI deixa enviar e mostra esse erro, como antes.

Achados para a Fase 4 (sem alteração): chaves que o `boot` deixou de
usar, sem uso em outro crate: `d_set_as_default_for_the_next_boot`,
`d_set_default_t_change_timeout`, `g_regenerate_all_initramfs_images`,
`kernel_620593`, `bootloader_entry`, `initramfs_fc455d`, `base_system`,
`bootloader`, `regenerate`, `apply_theme`, `modules`, `binaries`,
`files`, `hooks`. `navigate_tab_actions_move_enter_activate_r_refresh_esc_back_help`,
`r_refresh_esc_back_help` e `navigate_enter_open_esc_back_r_refresh_help`
ainda são usadas por outros crates.

### 5.6 Notas da Fase 3: `packages` (2026-10-05)

Dependências: como nas fases anteriores, `argvus-tui` e `argvus-i18n` na
branch `ux_ui`. Em execução, os títulos de seção (`section_package`,
`section_source`, `section_dependencies`, `section_packages`,
`section_summary`, `section_servers`), `remove_marked`, `mark`,
`cache_files`, `cache_file_count`, `configure_mirrors`,
`mirror_enabled`/`mirror_disabled`, `unavailable_no_reflector`, as três
limpezas (`clean_cache_keep_three`/`_keep_one`/`_uninstalled`) e
`value_must_be_between` exigem o `argvus-i18n` instalado a partir de
`ux_ui`; sem elas a tela mostra a chave. Os testes comparam com `tr(...)`
para não depender do catálogo instalado. A página do pacote reaproveita os
rótulos com dois-pontos (`name`, `version_20bc85`, `repository`...) sem
o `:`.

Conferido no código antigo e preservado: o efeito, o privilégio (`pkexec`,
ou o helper do AUR como usuário normal), o cálculo do plano e a
confirmação de cada operação (Install, Reinstall, Remove, Update,
Upgrade all, Refresh database, Downgrade, as três limpezas e Aplicar
mirrors); as contagens do plano na confirmação; o painel de saída ao vivo
(rolagem, follow, `Esc` fecha); as listas que filtram ao digitar (D8) e o
D10 (`q`/`?` são texto); a busca remota com Enter sem resultado e pelo
`/`; as opções e limites do reflector (idade 1–8760 h, quantidade 1–100),
o país padrão e a prévia confirmada.

Mudanças de comportamento, além da apresentação:

- Sem barra de botões; `Tab`/`BackTab` não fazem nada no `packages` e
  `h`/`l` não movem mais foco entre botões.
- Install, Reinstall e Remove saíram das listas e ficam na página do
  pacote (D4). Remover um órfão sem marcá-lo passa pela página dele.
- Órfãos: Space marca em vez de digitar espaço no filtro; as marcas
  aparecem na coluna `[x]` (D31).
- Cache: os arquivos ficam numa subpágina só de rolagem (D32).
- `/` só abre a busca nas cinco listas com filtro (antes abria em qualquer
  página, sem efeito visível fora delas).
- Editor de mirrors é uma página: `←` volta uma opção em País, Protocolo e
  Ordenação (antes avançava); Enter avança nessas três e abre campo
  numérico em Idade e Quantidade (D34).
- `Configurar mirrors` fica desabilitado com o motivo sem reflector (antes
  Enter mostrava o erro).
- Confirmação pelo componente único (`y`/`n`); estilo de perigo em Remove,
  limpezas e Downgrade; download em MiB/GiB; AUR sem plano vazio (D35).
- `Esc`/`←` voltam com o cursor no item que abriu a página (antes, na
  primeira linha). O rodapé deixou de anunciar `r` nas listas com filtro,
  onde `r` sempre foi texto.
- A página do pacote decide Instalar/Reinstalar e habilita Remover pelo
  estado da entrada da lista enquanto os metadados não chegam (antes,
  nesse intervalo, tratava todo pacote como não instalado).

Correções feitas na migração:

- Órfãos: as marcas guardavam o índice da lista **filtrada** e eram lidas
  da lista **sem filtro** (`selected_names`); com filtro ativo, removia o
  pacote errado. Agora as marcas seguem o nome, e um recarregamento
  descarta as que não estão mais na lista (antes nunca eram limpas).
- AUR: a lista era filtrada na tela, mas o cursor indexava os resultados
  sem filtro; Enter abria outro pacote.

Achados, **sem alteração**:

- Reinstalar um pacote do AUR já instalado roda `pacman -S <nome>`
  (`Action::Reinstall`), que falha para pacotes estrangeiros. Formato
  anterior à Fase 3.
- A mensagem de status é desenhada sobre o rodapé (`f.area()`) e, a
  80 colunas, cobre o começo das dicas até sumir; o `boot` desenha em
  `body`. Comportamento anterior à Fase 3.
- O `query` persiste entre Órfãos, Atualizações e AUR ao voltar à Home
  (só Busca e Instalados o limpam ao abrir), como antes.

Achados para a Fase 4 (sem alteração): `PackagesPage::HistoryDetails`
continua sem caminho até ela. Chaves que o `packages` deixou de usar, sem
uso em outro crate: `dependencies`, `downgrade_c6e26f`, `download_bytes`,
`generate_mirrors_with_reflector`, `keep_one`, `package_b3ef4b`, `select`,
`source_70835f`, `tab_actions_r_refresh`, `uninstalled`, `update`.
`navigate_enter_open_esc_back_r_refresh_help`,
`navigate_tab_actions_move_enter_activate_r_refresh_esc_back_help` e
`r_refresh_esc_back_help` ainda são usadas por outros crates.

---

### 5.6.1 Pacotes agrupados por origem e tarefa (2026-10-25)

A Home de Pacotes passou a ser `Repositórios oficiais`, `AUR (Arch User
Repository)` (com helper), `Cache`, `Histórico`, `Downgrade` e `Mirrors`
(D55). Cada origem abre `Instalar`, `Remover` e `Atualizar`, todas na tela
de lista com filtro de antes (`draw_filtered_list`), com a barra de ações só
em `Atualizar`. `PackagesPage` trocou `Search`, `Installed`, `Updates` e
`Aur` por `Source(PackageSource)` e `Task(PackageSource, PackageTask)`, e
`Details(usize)` por `Details`, com a linha de origem guardada no app.

Atalhos: Enter executa a tarefa e `→` abre o pacote (D56, atalho novo);
`Tab` leva à barra de Atualizar (D58). Órfãos fica em `Remover` (D57). O
backend ganhou `foreign()` (`pacman -Qm`), `aur_updates()` (`<helper> -Qua`)
e o contador `foreign_count` no painel; as ações novas `AurUpgradePackage` e
`AurUpgrade` rodam o helper como usuário (D58).

i18n: novas chaves `official_repositories`, `aur_repository`,
`update_aur_package`, `upgrade_all_aur` e `search_*` (rótulos da busca
global); saíram `install_official`, `install_aur`, `installed_official`,
`installed_orphans`, `updates_official` e os cinco `breadcrumb_*` de
Pacotes, sem outro uso no workspace. Rotas CLI novas: `official`,
`aur-remove` e `aur-updates`.

---

### 5.7 Notas da Fase 3: `network` (2026-10-05)

Conferido no código antigo e preservado: Wi-Fi ligado/desligado (imediato, sem
confirmação, via `set_wifi_enabled`); conexão com senha (campo mascarado com
`•`, captura de texto, D10); DNS manual e automático (imediato, pelo helper
elevado); busca `/` nas listas; varredura e `r` (Wi-Fi com varredura, demais
páginas sem); Esquecer com confirmação; a rota Firewall (sem mudança).

Mudanças de comportamento, além da apresentação:

- Sem barra de botões: `Tab`/`BackTab` não movem mais o foco.
- Conectar e Desconectar saem das listas de interfaces, Wi-Fi e VPN e vão
  para o detalhe do item (D39). Enter na interface continua abrindo o detalhe;
  Enter na rede Wi-Fi e na VPN continua conectando.
- Atalho novo `i` (Detalhes) em Wi-Fi e VPN, com `i Details` no rodapé nessas
  linhas (D36). Não substitui nenhuma tecla existente.
- Conectar e Desconectar do detalhe de interface passam a ser alcançáveis; antes
  só existiam como botões da lista, e o detalhe tinha só Refresh.
- Toggle Wi-Fi desabilitado quando não há estado de rádio, com o motivo à
  direita (D37). O botão antigo aparecia e agia mesmo assim.
- Desconectar da lista de interfaces tinha estilo de perigo e agora é Action
  comum (D40).
- Esquecer passa pelo `ConfirmDialog`, com estilo de perigo e foco em Cancelar.
  Texto e efeito iguais.
- `Esc`/`←` voltam com o cursor na linha que abriu a página.

Achados, **sem alteração**:

- `wifi_enabled = None` cobre "sem placa" e também "consulta falhou". O texto
  `Indisponível · sem placa Wi-Fi` é uma aproximação para os dois casos.
- `run_wifi` ainda grava o texto `Conectado` sem traduzir (`ui.rs`, sucesso da
  conexão com senha). Fora do escopo da migração.

Testes: 26 testes do crate passam. Nenhum executa `nmcli`, `pkexec` ou
`systemctl` de fato: a dev-dependency `testing` do `core` faz o processo falhar
sem executar. Os testes de navegação não disparam job quando dá para evitar.

Não verificado nesta fase: execução real com NetworkManager e tela em 80x24.
As chaves novas (`unavailable_no_wifi_card`, `signal`, `security`,
`frequency`) só aparecem traduzidas com `argvus-i18n` instalado a partir de
`b46d99b`.

### 5.2 Correções fora da migração de `RowKind` (2026-10-07)

Três bugs relatados pelo usuário, sem relação com a migração de botões →
`RowKind` das Fases 1-4; não alteram o status "pendente" da linha "Home" na
tabela de Progresso (que é só sobre migrar o grid da Home para o menu
compartilhado).

**Breadcrumb/título duplicando "ARGVUS" e caminho errado no Hyprland.**
`draw_header` (`argvus-tui`) já desenha "ARGVUS" em negrito; a chave i18n
`control_center.argvus_control_center` e o `shell()` do `argvus-tui` também
incluíam "ARGVUS Control Center" no texto, duplicando a marca em toda tela
(`"ARGVUS ARGVUS Control Center"`). A chave passou a valer só `"Control
Center"` (en-US e pt-BR). Além disso, Keyboard Shortcuts, Window Rules e as
páginas de efeitos reaproveitadas do `appearance` (Window Spaces, Animations,
Blur, Borders) só existem atrás do roteador "Hyprland" do Home, mas o
breadcrumb delas ainda mostrava a raiz genérica da Settings/Appearance, e
Window Rules ficava aninhado por engano sob Keyboard Shortcuts. O breadcrumb
dessas páginas agora é `Hyprland > Settings > …`, a própria tela "Hyprland"
ganhou o segmento "Settings" (`Hyprland > Settings`), e o separador `›` usado
só em `hyprland.rs`/`appearance` foi trocado por `>` para bater com o resto do
app. O título hardcoded (sem i18n) da tela de Usuários também foi trocado
para usar `app.breadcrumb()`.

Isso também torna a documentação em `docs/en/index.md` e `docs/pt-br/index.md`
que descrevia "Window rules" como estando em "Applications" desatualizada:
desde a reorganização da Home que introduziu a categoria "Hyprland" (commit
`0400503`, anterior a esta correção), Window rules e Keyboard Shortcuts vivem
em "Hyprland", não em "Applications". Os dois `index.md` foram corrigidos
junto desta correção de breadcrumb, já que os dois descrevem o mesmo caminho.

**Busca da Home imprecisa.** `SearchRegistry::match_rank`
(`argvus-control-center-core`) só considerava `starts_with` por palavra ou
`contains` no título inteiro; agora também casa por `contains` em qualquer
palavra individual do título/categoria, então digitar um trecho no meio de
uma palavra (ex.: "resolu") encontra a tela mesmo sem depender só da lista de
`keywords`. O rótulo exibido para a entrada de resolução de Displays também
estava errado (mostrava "Res" em vez de "Resolution"); corrigido o mapeamento
em `search_title` (`argvus-control-center/src/ui.rs`).

**Scroll lateral ausente na Home.** `draw_home_cards` não tinha nenhum
indicador visual de que havia mais conteúdo abaixo quando o grid não cabia na
tela (só truncava o desenho). Passou a desenhar uma `Scrollbar` do ratatui na
borda direita, no mesmo padrão já usado em `argvus-about`, visível só quando
o total de linhas do grid excede a altura visível.

Validação: `cargo fmt --all -- --check`, `cargo check --workspace
--all-features`, `cargo clippy --workspace --all-features -- -D warnings` e
`cargo test` (por crate) em `argvus-control-center`,
`argvus-control-center-settings`, `argvus-control-center-appearance`,
`argvus-control-center-core` e `argvus-tui`, todos sem falhas. Não
documentado como "Fase" porque não migra nenhum botão para `RowKind`.

### 5.11 Raiz traduzida, fonte única de cartões e breadcrumbs em pt-BR (2026-10-15)

- **Raiz traduzida**: a chave `control_center.breadcrumb.root` (en-US
  `Control Center`, pt-BR `Central de Controle`) é a única fonte do prefixo do
  cabeçalho. `argvus_tui::page::shell` recebe `lang` e monta `<raiz> > <caminho>`;
  antes o prefixo era o texto fixo `Control Center >`. A chave
  `argvus_control_center` (título da Home e do About) passou a `Central de Controle`
  em pt-BR, para o título e o caminho dizerem a mesma coisa.
- **Fonte única dos cartões**: `argvus-control-center-core/src/navigation.rs`
  define `HomeCardId` (Appearance, DesktopInput, Preferences, Hardware, System,
  Tools), `root_label`, `join` e o separador `" > "`. A Home (`home/catalog.rs`),
  os breadcrumbs de cada crate, o settings e a busca (`search_category`) leem os
  nomes dos cartões dali. Não há mais `control_center.card_*` repetido dentro de
  breadcrumbs. As chaves são as mesmas que já existiam; nenhuma chave de cartão foi criada.
- **Bluetooth sob Hardware**: o Bluetooth passa a mostrar o cartão em que está
  listado. Antes: `Control Center > Bluetooth > State`; agora:
  `Control Center > Hardware & Connectivity > Bluetooth > State`
  (pt-BR: `Central de Controle > Hardware e conectividade > Bluetooth > …`).
- **Testes**: breadcrumbs em en-US e pt-BR no settings (`breadcrumbs_are_translated_in_pt_br`),
  cabeçalho renderizado em pt-BR no diagnostics
  (`diagnostics_header_is_translated_in_pt_br`), paridade da busca com o cartão
  (`search_and_breadcrumbs_share_card_names`), `navigation.rs` (raiz e cartões
  traduzidos em ambos os idiomas) e a raiz do `argvus-tui` lida do catálogo.

Glossário (en → pt-BR), para revisão humana:

| en-US | pt-BR | Chave |
| --- | --- | --- |
| Control Center | Central de Controle | `control_center.breadcrumb.root` |
| Appearance | Aparência | `appearance` |
| Desktop & Input | Desktop e entrada | `card_desktop_input` |
| Preferences | Preferências | `preferences` |
| Hardware & Connectivity | Hardware e conectividade | `card_hardware` |
| System Administration | Administração do sistema | `card_system` |
| Tools & Diagnostics | Ferramentas e diagnóstico | `card_tools` |
| Settings | Configurações | `settings` |
| Language & Region | Idioma e Região | `language_region` |
| Displays | Monitores | `displays` |
| Audio | Áudio | `audio` |

Títulos da busca:

- Cada entrada do registro (`app.rs`) guarda a chave do título, e o título é
  resolvido no idioma atual ao registrar. A função `search_title` e o mapa por
  segmento de rota foram removidos. Assim a busca mostra o mesmo texto da página,
  e a pesquisa também casa termos em pt-BR. As palavras-chave em inglês e português continuam.
- Três títulos mudaram de texto para bater com a página: `Dev Services` virou
  `Developer Services`, `Highlight color` virou `Accent Color`, e `Kernel and Boot`
  virou `Kernel & Boot`. `Widget Telemetry` passou a `Widget telemetry`, a
  forma da chave existente. Só quatro chaves são novas: `themes`,
  `taskbar_icons`, `taskbar_date` e `taskbar_time`.

Revisão de termos pt-BR:

- `Central de Controle` foi escolhido pelo mantenedor.
- `Apps Padrão` virou `Aplicativos padrão`, sem mistura de idiomas.
- `Kernel & Boot` virou `Kernel e boot`.
- `Hostname` continua como termo técnico, consistente com a seção do sistema.
- Ainda em inglês no catálogo pt-BR, para decisão futura: `Mirrors`, `Snippets`,
  `Autostart` e `Dev Dashboard`.

Verificação visual:

- Em um terminal real (tmux, 120x40), a Home renderiza em pt-BR com
  `Central de Controle` no título, e a página Hardware mostra
  `Central de Controle > Hardware e conectividade > Visão geral do hardware > Resumo`.
- A digitação de busca não chegou ao app no tmux: `/` e as letras não alteraram
  o campo, e o `j` não moveu a seleção de forma verificável. Por isso a busca em
  pt-BR foi verificada por testes (títulos resolvidos e chaves presentes nos dois
  catálogos), não pela tela.

### 5.10 Settings do Control Center, System Administration e breadcrumbs (2026-10-15)

- **Settings** é um item fora dos cards, no fim da Home (`HomeRow::Standalone`).
  Abre `Route::AppSettings` (`settings_hub.rs`), com duas linhas: **Language**
  (`Page::Language`, Esc volta ao hub) e **Style ›** (`Route::Config`, Esc volta
  ao hub). `s` abre o hub; a CLI `config` continua abrindo Style direto.
- **Blur** saiu da tela Style. Blur do Control Center é controlado pelo Hyprland
  (Hyprland > Blur). O valor persistido `blur_control-center_enabled` é lido ao
  abrir e reenviado sem alteração, porque o script `surface-apply` exige o
  argumento e outros componentes leem a chave.
- **System Administration** passou a ser: Packages, Services, Boot, Hostname,
  Users & Groups (hub com Users › e Groups ›), Firewall e Notifications (Do Not
  Disturb). **General** foi removido; a rota CLI `system` abre Hostname.
- **Firewall** carrega os dados ao abrir por qualquer caminho (Home, busca ou
  CLI), via `Administration::ensure_loaded`. Antes só carregava no construtor,
  então aparecia só a linha Reload.
- **Breadcrumbs**: cada página mostra o card onde está listada. Os crates de
  domínio prefixam o card (`Desktop & Input > Displays`, `Preferences > Power`,
  `Hardware & Connectivity > Network & Wi-Fi`, `System Administration > Packages`,
  `Tools & Diagnostics > Diagnostics`). Date & Time fica em `Preferences > Date
  & Time` mesmo aberto pelo hub de Language & Region.

Desvio do plano aprovado: o card de cada destino é fixo, então o prefixo fica no
`breadcrumb()` de cada crate (sem campo `parent`). Isso cobre Home, busca e CLI
pelo mesmo caminho e evita estado extra.

### 5.9 Taxonomia do esboço `TUI_home.png` (2026-10-15)

Substitui a organização da 5.8. A Home agora tem seis cartões, na ordem do
esboço: **Appearance** (Fonts, Desktop Style, Application Style, Taskbar,
Control Center, Widget Telemetry), **Desktop & Input** (Hyprland, Displays,
Keyboard & Shortcuts, Mouse & Touchpad, Window Rules), **Preferences**
(Language & Region, Date & Time, Default Applications, Power, Session,
Interface Language, Configuration), **Hardware & Connectivity** (Hardware
Overview, Network & Wi-Fi, Bluetooth, Audio, Storage), **System
Administration** (Packages, Services, Boot, Users & Groups, Firewall,
General) e **Tools & Diagnostics** (Projects, Snippets, SSH keys, GPG keys,
Dev Services, Diagnostics, About ARGVUS). Cada cartão tem título, descrição
e ícone no próprio catálogo; um cartão sem destino disponível não aparece.

Mudanças de código: o catálogo está em `home/catalog.rs` (`HomeDestination`
com IDs estáveis, `glyph()` e `HomeCategory`), a Home não usa mais números de
ação nem comparação de texto traduzido para ícone e descrição, e o cartão
"wide" de System foi removido (todos os cartões têm o mesmo tamanho). A linha
de descrição do cartão tem divisória, e a Home ganhou um subtítulo.

Busca e breadcrumbs seguem o cartão onde o destino aparece: Default Apps em
Preferences; Projects, Snippets, SSH keys e GPG keys em Tools & Diagnostics;
Firewall em System Administration; Keyboard Shortcuts e Window Rules em
Desktop & Input; Taskbar, Control Center e Widget Telemetry em Appearance.
O ícone `WRENCH` (`nf-md-wrench`, `U+F05B7`, conferido na Nerd Font instalada)
entrou no catálogo de `argvus-tui`.

Desvios do esboço estão em 6.3 (D44 a D49).

### 5.8 Reorganização das categorias da Home (2026-10-15, substituída pela 5.9)

Mudança de arquitetura de informação, fora das fases de `RowKind`. Não altera
ações nem o momento em que cada mudança é efetivada.

- **Home**: os cards Hyprland, Taskbar, Control Panel e Widget Telemetry viram
  um único card **Desktop Shell** (quatro itens com seus próprios destinos).
  **Language & Region** abre diretamente Language (antiga Regional Locale),
  Time Zone, Date & Time, System Locales (só no Arch) e Keyboard Layout; o hub
  `Page::LocaleRegion` continua acessível pela CLI `locale`. **Appearance**
  abre Fonts, Desktop (Theme, Appearance Mode, Accent Color, Wallpaper) e
  Application Style (Terminal, Launcher). O item "Themes & Visuals" saiu da
  Home; o hub `AppearancePage::Home` fica só como raiz da CLI `appearance`.
  **Security** (novo) reúne SSH Keys, GPG Keys e Firewall. Applications
  mantém Default Apps, Projects e Snippets.
- **Breadcrumbs** (`argvus-control-center-settings` e
  `argvus-control-center-appearance`) passaram a seguir o cartão da Home, por
  exemplo `Control Center > Security > SSH keys`,
  `Control Center > Language & Region > Language`,
  `Control Center > Applications > Default Apps` e
  `Control Center > Desktop Shell > Hyprland > Keyboard Shortcuts`.
  `Control Center > General` substitui o antigo `System`, e Hostname passou a
  usar a chave i18n em vez de texto fixo. Layout do teclado também deixou de
  ser texto fixo.
- **Firewall** saiu da aba de Network: `NetworkPage::Firewall` foi removido. A
  página é a mesma de `Page::Firewall`, aberta a partir de Security. A rota
  CLI `network firewall` continua funcionando e abre essa página.
- **Busca**: SSH Keys, GPG Keys e Firewall usam a categoria Security. O
  registro ganhou a entrada Firewall.
- **i18n**: chave `control_center.security_category` (Security / Segurança)
  para o cabeçalho do card. Removidas as chaves órfãs
  `hyprland_description`, `taskbar_description`,
  `control_panel_description`, `widget_telemetry_description`,
  `themes_and_visuals` e `themes`, em en-US e pt-BR.

### 5.12 Notas: Conexões de rede (2026-10-27)

Funcionalidade nova, não migração: Rede e Wi-Fi ganhou **Conexões**, **Ativar uma conexão** e **Adicionar conexão**, com um editor de perfis do NetworkManager no modelo de lista única.

- Domínio puro em `argvus-control-center-network/src/profile/`. `schema.rs` tem uma tabela de propriedades por seção (widget, padrão do NetworkManager, condições de exibição e de habilitação), da qual saem as linhas, a validação e os argumentos do `nmcli`. `draft.rs` guarda o rascunho, `validate.rs` faz as checagens e `nmcli.rs` traduz de e para o `nmcli`.
- Backend em `backend/connections.rs`: listar, carregar, salvar (só o que mudou), ativar, excluir, importar, plugins de VPN e chaves WireGuard.
- UI em `ui/connections.rs` e `ui/editor.rs`.
- Peças compartilhadas novas:
  - `ProcessRequest::stdin` em `argvus-control-center-core`, com `Debug` que só mostra o tamanho;
  - `TextEditor::masked` em `argvus-tui`;
  - os glyphs da seção 1.2;
  - `Capabilities::has_teamd` e `has_pppd`.
- A rede passou a receber o `KeyEvent` inteiro e o paste (`event.rs`).
- Rotas: busca `network.connections`, `network.activate` e `network.add_connection`; CLI `network connections|activate|add`.
- Breadcrumbs: `Hardware & Connectivity > Network & Wi-Fi > Connections > <perfil>`, `… > Connections > Add connection > <tipo>` e as subpáginas (lista, Portas, Peers) depois do perfil.

## 6. Pontos de decisão

### 6.1 Decisões registradas (2026-10-04)

| Ponto | Decisão | Status |
| --- | --- | --- |
| D1 | Toggle como `[x]`/`[ ]` em todo o app. | decidido |
| D2 | Aceito. Sem linha `Cancel` nas páginas com rascunho do `appearance`; `Esc` com rascunho pendente pede confirmação (componente único). | decidido |
| D3 | Em linhas Value com step, `←/→` ajustam; `Home/End` continuam navegando; `Esc` volta; o rodapé mostra a dica de ajuste nessas linhas. | decidido |
| D4 | Enter mantém o que faz hoje na ação principal do item. Ações secundárias (ex.: Forget, Remove, Trust) vão para a página de detalhes do item, mantendo os atalhos de uma tecla existentes. | decidido |
| D5 | Aceito, desde que cada botão sem linha própria fique marcado no inventário como "Equivalente: Enter na linha do item" (feito nas seções 0.4, 0.5, 0.6, 0.8 e 0.10). | decidido |
| D6 | Manter `r` = restaurar padrões no `settings`, passando pela confirmação única, e criar também a linha `Restore defaults` / `Restaurar padrões`. | decidido |
| D7 | Recomendação aceita: manter `Tab`/`BackTab` na grade da Home. | decidido (provisório, ver nota) |
| D8 | Recomendação aceita: listas com filtro por digitação mantêm o comportamento atual (letras viram texto da busca). | decidido (provisório, ver nota) |
| D9 | Recomendação aceita: ícones com mais de uma opção são decididos com captura de tela, na migração de cada crate. No `appearance`, aprovada a proposta da Fase 2 (seção 1.4). | decidido para o `appearance`; provisório para os demais crates |
| D10 | Incluir. Campo de texto ativo captura as teclas; `q` e `?` globais não fecham o app nem abrem a ajuda durante a digitação. | decidido (Fase 1) |
| D11 | Fora do escopo. Mouse nas páginas de domínio fica para outra tarefa; o critério "com e sem mouse" vale para a Home (que já trata clique) e não é exigido das páginas de domínio neste refactor. | decidido |
| D12 | Recomendação aceita: hints em inglês fixo resolvidos pelo rodapé contextual; chave i18n nova para `Hostname`. | decidido (confirmado na Fase 2; `Hostname` resolvido na Fase 3 com `control_center.hostname`) |
| D13 | `docs/ux-audit.md` entra no `ignorePaths` do `cspell.json`. Verificado: o site só publica `docs/en/` e `docs/pt-br/`; arquivos na raiz de `docs/` são ignorados (`web/argvus-website/src/lib/documentation/loader.ts`, filtro de locale `en`/`pt-br`). | decidido |
| D14 | `Home/End` navegam (D3); para preservar o 0/100 direto, Enter numa linha Value com passo abre o prompt numérico 0–100, que grava só no rascunho (`Apply` continua necessário). | decidido (Fase 2) |
| D15 | Recarregar (`r` ou o automático ao entrar numa página) preserva um rascunho com alterações e só reconstrói um rascunho limpo. | decidido (Fase 2) |
| D16 | `Space` não é removido no `appearance`: continua ativando Action, Submenu e Choice (e abrindo o editor de Value sem passo), tratado na página sem mudar o padrão do componente para os outros crates. | decidido (Fase 2) |
| D17 | `Bloquear senha`/`Desbloquear senha` continuam duas linhas: o snapshot de `argvus accounts` não traz o estado de bloqueio (`crates/argvus-accounts-core/src/admin.rs`, objeto `users`). | decidido (Fase 3) |
| D18 | `Remover avatar`: Action com confirmação, sem estilo de perigo, na seção Avatar. `Aplicar regras salvas`: Action confirmada na seção Regras, fora da Zona de perigo; a confirmação avisa que o firewall será reiniciado. | decidido (Fase 3) |
| D19 | `r` em Atalhos mantém o efeito de antes (restaura **só o atalho selecionado**; na edição, o atalho editado e volta) e passa pela confirmação única, como o D6 exige. `Restaurar todos os atalhos` é a linha Destructive da Zona de perigo. A linha `Restaurar padrão` da edição continua imediata, como antes. | decidido (Fase 3) |
| D20 | Locales do sistema: Enter numa linha alterna o locale (rascunho); `Aplicar` fica na barra **Ações** embaixo da lista, com `/` para buscar no topo, `Tab`/`←/→` para o foco, como Fuso horário; apagada sem mudança e confirmada antes de gravar (revisado em 2026-10-24). | decidido (Fase 3, revisado) |
| D21 | Fontes: Tamanho vira Value no topo do seletor (prompt 8–32 no Enter). Sistema > Hostname abre o popup direto; a página Hostname segue para a rota CLI. | decidido (Fase 3) |
| D22 | Layouts: Enter define o padrão e Space liga/desliga o layout na lista; o rodapé mostra as duas teclas (`Enter Definir padrão`, `Space Alternar`). | decidido (Fase 3) |
| D23 | Firewall: Iniciar/Parar viram uma linha de estado `Serviço · Ativo/Parado` cuja Enter alterna (o backend informa `active`), confirmada como antes. | decidido (Fase 3) |
| D24 | Reversão de monitor pelo componente único com prazo: foco em Reverter, `Enter` imediato reverte, `Space` sem efeito, `y` mantém, `n`/`Esc` revertem; o rodapé destaca `y Manter`. Motivo: `Enter` por reflexo com a tela ruim não pode manter a configuração. Prazo (15 s), destino (`previous_config` com `apply_all`) e mensagem idênticos. | decidido (Fase 3) |
| D25 | `displays` não tem rascunho: `Aplicar` é Action comum que reaplica a configuração salva com a contagem, sem `draft_actions`; o inventário 0.4 foi corrigido. | decidido (Fase 3) |
| D26 | `Restaurar padrão` mantém o comportamento atual: imediato e sem contagem de reversão (`reset_button` não arma `RevertState`). | decidido (Fase 3) |
| D27 | (B1 do `boot`) `[ Default ]` do Bootloader com GRUB não ganha linha equivalente: com GRUB a lista de entradas vem vazia (`parse_grub_defaults` não preenche `entries`) e o botão só mostrava o erro "não foi possível mapear". Registrado no inventário 0.5 como "sem efeito no código antigo". | decidido (Fase 3) |
| D28 | (B2 do `boot`) Timeout, Linha do kernel e Regenerar GRUB ficam só na página Bootloader. Linhas inválidas no momento ficam desabilitadas e mostram o motivo à direita, traduzido: `Indisponível · sem entrada systemd-boot` (Definir padrão sem entrada mapeável) e `Indisponível · bootloader desconhecido` (Definir padrão e Timeout com bootloader desconhecido). | decidido (Fase 3) |
| D29 | (B3 do `boot`) Ícones: `STAR` (Definir padrão), `TIMER` (Timeout), `TERMINAL` (Linha do kernel), `SYNC` (Regenerar GRUB e Regenerar initramfs, mesmo sentido); Home do boot conforme 1.5. | decidido (Fase 3) |
| D30 | O campo "Linha do kernel" aceita qualquer texto até 2048 bytes (limite do backend); aspas, `\` e caracteres de controle continuam recusados ao aplicar. O popup alarga até 100 colunas e mostra o fim do texto. Commit separado (`93bc44f`). | decidido (Fase 3) |
| D31 | (`packages`) Órfãos: Space marca/desmarca o pacote, com a marca visível na coluna `[x]`; Enter continua abrindo os detalhes; `Remover marcados (N)` na Zona de perigo age sobre o conjunto marcado. | decidido (Fase 3) |
| D32 | (`packages`) Os arquivos do cache ficam numa subpágina só de rolagem (`Arquivos em cache`), para a lista longa não ficar inalcançável entre o Resumo e a Zona de perigo. | decidido (Fase 3) |
| D33 | (`packages`) Histórico: Enter nunca abria uma entrada (`HistoryDetails` inalcançável); registrado no inventário 0.6 como "sem efeito no código antigo", sem página de detalhes. | decidido (Fase 3) |
| D34 | (`packages`) O editor de mirrors vira página com Value por opção e `Gerar preview` no fim, sem `draft_actions` (nada é salvo antes da confirmação). Glyphs novos `SORT` e `COUNTER` no `argvus-tui`. | decidido (Fase 3) |
| D35 | (`packages`) A confirmação mantém as contagens do plano (sem página de plano), com o download em MiB/GiB; a instalação pelo AUR não mostra 0/0/0, só o aviso do PKGBUILD. | decidido (Fase 3) |
| D36 | (`network`) Enter em Wi-Fi e VPN continua conectando (D4). `i` (Detalhes) é atalho novo: abre a página de detalhes da rede com Conectar, Desconectar e Esquecer. O rodapé mostra `i Detalhes` nas linhas de rede Wi-Fi e VPN, também fora da página de detalhes. Em Interfaces, Enter já abre o detalhe e não muda. | decidido (Fase 3) |
| D37 | (`network`) Toggle Wi-Fi desabilitado quando `wifi_enabled` é `None`, com o motivo à direita (`Indisponível · sem placa Wi-Fi`). Mudança intencional: o botão antigo agia mesmo sem placa. | decidido (Fase 3) |
| D38 | (`network`) Esquecer usa `ConfirmDialog` em estilo de perigo, com foco em Cancelar. Texto e efeito iguais. | decidido (Fase 3) |
| D39 | (`network`) Conectar e Desconectar de interfaces, Wi-Fi e VPN saem das listas e vão para a página de detalhes do item. Enter na interface continua abrindo o detalhe. | decidido (Fase 3) |
| D40 | (`network`) Desconectar é Action comum, sem confirmação (não havia), e não estilo de perigo. | decidido (Fase 3) |
| D41 | (`network`) Glyphs novos no `argvus-tui`: `PROXY` (`nf-md-routes`, `f046a`) e `AUTORENEW` (`nf-md-autorenew`, `f006a`), conferidos na Symbols Nerd Font Mono. | decidido (Fase 3) |
| D42 | (`network`) As listas terminam com a seção Ações (Atualizar), e `r` continua igual. Em Status, DNS, Proxy e detalhes, as Ações ficam antes do resumo. | decidido (Fase 3) |

Nota sobre D7, D8, D9 e D12: a resposta veio como o modelo
`[aceito as recomendações / minhas respostas]`, sem escolha explícita. Foram
registradas como recomendação aceita; confirmar antes da fase que depende de
cada uma (D7: Home; D8: `packages`; D9: migração de cada crate; D12:
`appearance` e `settings`).

Ajustes aprovados junto com o plano da Fase 1:

- **Foco inicial da confirmação.** Verificado: todos os usos de
  `ConfirmationState` (`audio`, `bluetooth`, `boot`, `displays`, `hardware`,
  `network`, `packages`, `power`, `services`, `session`) criam o estado com
  `::default()`, ou seja, foco em **Cancel**. O componente único mantém Cancel
  como foco inicial; com `danger` ligado o foco também é Cancel, e
  confirmações comuns seguem o mesmo padrão já existente.
- **Cursor nas pontas.** Verificado: nenhuma lista dá a volta hoje
  (`argvus_tui::page::Selection`, `settings::App::move_selection`,
  `App::move_home` e as listas com índice próprio usam
  `saturating_sub`/`clamp`). Só as barras de botões davam a volta
  (`rem_euclid`), e elas serão removidas. A lista única para nas pontas.
- **D10, escopo.** `q` deixa de sair do app só onde a lista já trata
  caracteres digitáveis como filtro ou onde um campo de busca/texto está
  ativo. As telas afetadas estão listadas na seção 5 (Progresso, D10).

### 6.2 Propostas originais

- **D1. Visual do Toggle.** Proposta: manter `[x]`/`[ ]` (ASCII, já usado em
  todo o app e funciona sem Nerd Font). Alternativa: glyph de switch
  (`toggle_switch` `f0521`/`toggle_switch_off_outline` `f0a19`).
- **D2. Linha `Cancel` nas páginas com rascunho do `appearance`.** Hoje não
  existe botão Cancel nelas (só `Apply`; `Esc` descarta em silêncio). Proposta:
  não criar linha `Cancel` (Esc já cumpre o papel) e adicionar a confirmação no
  `Esc` com rascunho pendente. No Firewall do settings, o `Cancel` existente
  vira linha `Cancelar alterações` (recarrega e permanece na página, como hoje).
- **D3. `←/→` em Value.** A regra manda `←/→` ajustar valores, mas hoje `←` é
  "voltar" na linha de valor de SurfaceSection, e `Home/End` definem 0/100 ali.
  Proposta: em linhas Value, `←/→` ajustam e `Home/End` continuam navegando;
  0/100 passam a `Shift+←/→`? Ou manter `Home/End` só durante a edição inline.
- **D4. Ações dependentes do item** (Wi-Fi, Bluetooth, pacotes, perfis de
  display, kernels). Proposta: Enter mantém o efeito atual do item e uma
  página de detalhe do item (já existe em pacotes, kernels, serviços) lista as
  ações como linhas. Para Wi-Fi e Bluetooth isto cria uma página de detalhe
  nova; a alternativa é um submenu de ações aberto com `Enter` e o efeito
  atual de Enter movido para uma linha. Preciso da sua escolha, porque muda o
  que Enter faz em Wi-Fi/Bluetooth.
- **D5. Botões duplicados de ações já disponíveis por Enter** (`[ Apply theme ]`
  no Plymouth, `[ Default ]` no áudio, `[ Apply Governor ]`, `[ Apply Profile ]`,
  `[ Update ]` em Atualizações, `[ Profiles ]` na Home do displays,
  `[ Regenerate ]` no detalhe do initramfs). Proposta: não criar linha extra,
  porque a mesma ação continua na linha do item. Confirmar se aceita.
- **D6. Atalho `r`.** No `settings`, `r` restaura padrões (com confirmação); no
  Picker do `displays`, cancela. Proposta: manter `r` = restaurar no settings
  (preservação de atalho), mas com a confirmação única; no Picker, manter `r` e
  documentar. Alternativa: unificar `r` = atualizar e mover "restaurar" para
  outra tecla (quebra atalho existente).
- **D7. `Tab` na Home.** Hoje `Tab` avança item na grade. A regra diz `Tab`
  só para abas/painéis. Proposta: manter (a grade de cartões é o caso de
  "painéis"), ou remover e deixar só `↑↓←→`.
- **D8. Digitação vs atalhos em listas com filtro** (`packages`): letras viram
  texto da busca, então `r`, `j`, `k` não funcionam nessas páginas. Proposta:
  manter como está nesta refatoração (comportamento existente).
- **D9. Ícones com mais de uma opção** (marcados "→ D9" na seção 1). Proponho
  decidir na Fase 1 com captura de tela e ícones ligados/desligados.
- **D10. `q`/`?` globais engolindo digitação** (achado 6). É um bug funcional
  fora do escopo estrito de apresentação. Corrigir junto (guardar teclas
  quando um campo de texto estiver ativo) ou em tarefa separada?
- **D11. Mouse nas páginas de domínio.** Hoje só a Home trata clique. O
  critério de aceite pede "com e sem mouse". Proposta: a lista única
  (`draw_menu`) devolve a área de cada linha e o roteador passa cliques para a
  página ativa (clique = Enter na linha; Info ignorada). Isto adiciona suporte
  a mouse onde não existia; confirmar se entra no escopo.
- **D12. Hints com texto fixo em inglês** no `appearance` (4 ocorrências) e o
  rótulo literal `"Hostname"` no settings. Proposta: resolver com o rodapé
  contextual (Fase 1/2) e chave i18n nova para Hostname.
- **D13. Local deste arquivo.** `docs/` publica no site (`en/`, `pt-br/` com
  front matter `slug`). Este arquivo não tem front matter e fica fora de
  `en/`/`pt-br/`, mas confirme se a sincronização do site ignora arquivos na
  raiz de `docs/`. O `cspell.json` usa idioma `en`; se o cspell rodar em `docs/`,
  este arquivo em português vai acusar palavras.

### 6.3 Pontos abertos da reorganização (2026-10-15)

| Ponto | Decisão | Status |
| --- | --- | --- |
| D43 | `HardwarePage::Power` (Hardware > Power) saiu da Home, mas continua acessível pela CLI `hardware power` e pela busca. É o único seletor de perfis de energia do ARGVUS (tecla `e`); o cartão Power & Session não tem esse recurso, então remover a página apagaria a função. Opções: (a) manter como está; (b) mover o seletor de perfis para Power & Session e remover `Hardware > Power`; (c) remover só a rota CLI e a entrada de busca. | aberto |
| D44 | Taxonomia da Home segue o esboço `TUI_home.png` e o prompt: Appearance, Desktop & Input, Preferences, Hardware & Connectivity, System Administration, Tools & Diagnostics. Substitui o cartão Desktop Shell e o cartão Security da reorganização anterior. | decidido (2026-10-15) |
| D45 | Itens do esboço sem página equivalente: "Icons & Themes" não existe como card (temas ficam em Desktop Style; o tema de ícones é a página Appearance > Desktop > Icons, com linhas de escolha aplicadas na hora); "Storage & Displays" vira só Storage (Displays já está em Desktop & Input). | decidido (2026-10-15); página de ícones implementada, confirmar |
| D46 | Rótulos combinados do esboço viram duas linhas, cada uma com seu destino real: Projects e Snippets; SSH keys e GPG keys; Keyboard & Shortcuts (página própria) separado de Keyboard layout (dentro de Language & Region). Power e Session continuam separados, pois são páginas distintas. Nenhum hub novo foi criado. | decidido (2026-10-15), confirmar |
| D47 | Language & Region abre o hub `Page::LocaleRegion`, que já reúne Time Zone, Date & Time, Regional locale, System locales e Keyboard. Date & Time continua também como item próprio em Preferences, como no esboço. | decidido (2026-10-15) |
| D48 | Itens fora do esboço mantidos para não perder funções: Application Style e Widget Telemetry (no prompt), Interface Language, Configuration, General (Hostname e configurações do sistema) e Window Rules (no prompt). | decidido (2026-10-15) |
| D49 | Language (idioma da interface) passa a ser "Interface Language" em Preferences. A entrada de Language & Region deixa de ter o mesmo rótulo. | decidido (2026-10-15) |
| D50 | Settings é um item fora dos cards, no fim da Home. Abre um hub com Language e Style, e `s` abre o hub. | decidido (2026-10-15) |
| D51 | Blur sai da tela Style. O valor `blur_control-center_enabled` é preservado, sem UI, porque o script `surface-apply` e outros componentes dependem dele. | decidido (2026-10-15) |
| D52 | General sai da Home. Hostname vira item próprio, e a rota CLI `system` abre Hostname. Page::System foi removida. | decidido (2026-10-15), confirmar |
| D53 | Notifications é uma página própria com Do Not Disturb, reaproveitando o toggle e o job existentes. | decidido (2026-10-15) |
| D54 | Date & Time aparece com o caminho raso (`Preferences > Date & Time`) mesmo quando aberto pelo hub de Language & Region. | decidido (2026-10-15) |
| D55 | (`packages`) A Home de Pacotes agrupa por origem: `Repositórios oficiais` e `AUR (Arch User Repository)` (só com `paru`/`yay`) abrem `Instalar`, `Remover` e `Atualizar`, que usam a mesma tela de lista com filtro; `Cache`, `Histórico`, `Downgrade` e `Mirrors` seguem na Home. As rotas antigas (`search`, `installed`, `updates`, `aur`, `orphans`) continuam abrindo as mesmas listas. | decidido (2026-10-25) |
| D56 | (`packages`) Nas listas de tarefa, Enter executa a tarefa da tela no pacote (instalar, remover, atualizar, com o plano e a confirmação de antes) e `→` abre a página do pacote. A tecla dedicada não pode ser uma letra porque as letras vão para o filtro (D8). Órfãos mantém Enter abrindo o pacote (D31). | decidido (2026-10-25) |
| D57 | (`packages`) Órfãos sai da Home e vira a primeira linha de `Repositórios oficiais > Remover`; voltar de Órfãos recarrega a lista de instalados com o cursor em Órfãos. | decidido (2026-10-25) |
| D58 | (`packages`) Backend mínimo do AUR: Remover lista `pacman -Qm` e remove com o mesmo `pacman -Rns` privilegiado; Atualizar lista `<helper> -Qua` e roda `<helper> -S <pacote>`/`<helper> -Sua` como usuário normal, sem plano, com o aviso do PKGBUILD. Com o menu AUR na Home, os pacotes do AUR saem da lista oficial de Remover. Instalar um pacote do AUR já instalado passa a usar o helper (`-S`), porque o `pacman -S` antigo não encontra pacotes fora dos repositórios. Na barra de Atualizar, `Tab` entra e sai da barra e `←/→` só andam entre os botões com a barra focada (antes `←/→` também entravam nela), já que `→` abre o pacote e `←` volta. | decidido (2026-10-25) |
| D59 | (`packages`) Remover um pacote exigido por outros instalados falhava no plano (`pacman -R --print`: "could not satisfy dependencies"). O plano agora usa `-Rs` (como o `-Rns` que executa) com `LC_ALL=C`; quando o pacman lista os dependentes (as linhas `:: removing … required by …` saem no stdout, junto com o `error:` do stderr), a confirmação de perigo (foco em Cancel) os nomeia e oferece a remoção em cascata, planejada com `-Rcs --print` e executada pela operação privilegiada `remove-cascade` (`pacman -Rcns`). O `--print-format` passou a usar tab real: o `\t` escrito não era expandido pelo pacman, e nome, versão e tamanho do plano nunca eram separados (o download aparecia como 0 B). | decidido (2026-10-25) |
| D60 | (`packages`) Complementa o D59 a pedido do usuário: com dependentes, a confirmação vira uma escolha entre `Remover só <pacote>` (`pacman -Rdd`, operação privilegiada `remove-only`) e `Remover com os dependentes` (`pacman -Rcns`, `remove-cascade`), acima de `Cancelar`. Vale para Remover oficial e do AUR e para a página do pacote. A cascata mantém `-Rcns` em vez do `-Rdc` sugerido: o `-d` não muda nada quando os dependentes saem juntos, e `-ns` removem também as dependências que ficam sem uso e os `.pacsave`, como o Remover normal. Novo componente compartilhado `argvus_tui::confirm::draw_choice` (`ChoiceState`, foco inicial em Cancelar, sem `y`) e `hints::choice_hints`. | decidido (2026-10-25), confirmar flags |
| D61 | (`packages`) `Repositórios oficiais > Repositórios > Adicionar / Listar`. Adicionar é página de rascunho (Origem URL/Texto/Formulário, Chave Nenhuma/URL/Texto, `Revisar e adicionar` apagado enquanto incompleto); sair com rascunho pede confirmação. Campos de texto usam o novo componente compartilhado `argvus_tui::text_editor` (`Ctrl+S`, colar), por isso a rota Packages passou a receber o `KeyEvent` completo e o `Paste`. | decidido (2026-10-25) |
| D62 | (`packages`) Segurança do Adicionar: downloads só por `https://` (`curl --proto =https --proto-redir =https`, até 256 KiB) e como usuário; a confirmação mostra a impressão digital em grupos de quatro antes do `pacman-key --lsign-key`; o lado root relê a impressão do mesmo arquivo com `gpg --show-keys` no chaveiro do pacman e recusa se não bater. A chave viaja em hexadecimal, porque o `.gpg` publicado é binário. | decidido (2026-10-25) |
| D63 | (`packages`) Gravação: `/etc/pacman.d/<nome>.conf` (nome da primeira seção) com `# argvus-key: <FPR>` no topo, e `Include` no fim do `pacman.conf` (ou a linha comentada do mesmo arquivo reativada). Depois, oferece `pacman -Syu` (`UpgradeAfterRepoAdd`), nunca `-Sy` sozinho. | decidido (2026-10-25) |
| D64 | (`packages`) Listar gerencia includes de `/etc/pacman.d/*.conf` e seções inline. Desativar: include vira `# Include = …`; seção inline recebe `#argvus-off ` em cada linha (para não confundir com `#[custom]` e `#[core-testing]` do arquivo padrão). Remover apaga linha/seção e o arquivo incluído (com cópia), e oferece `pacman-key --delete` quando a chave foi registrada. Seções oficiais nunca aparecem. Repositórios do mesmo arquivo (ex.: `argvus` e `argvus-extras`) mudam juntos. Parsing e edição puros em `argvus_control_center_core::pacman_conf`, usados pela UI e pelo root. | decidido (2026-10-25) |
| D65 | (`about`) O cabeçalho do About mostrava só a aba (`Control Center > System`), sem o cartão da Home. Passa a `Control Center > Tools & Diagnostics > About ARGVUS > <aba>` (chaves `card_tools` e `about_argvus` do `catalog.rs`), em todas as abas. A aba System vira `System & Hardware` (`about_system_hardware`), agrupada em Boot, System e Hardware, com rótulos alinhados numa coluna. A busca `about` passa ao cartão `card_tools`. Os dados são coletados no primeiro desenho da página (`App::ensure_loaded`), não mais na abertura do Control Center. | decidido (2026-10-25) |
| D66 | (`about`) System & Hardware ganha Bootloader (EFI `LoaderInfo`; GRUB por `/boot/grub`), Tema, Monitores (`hyprctl monitors -j`), Áudio/Shell/Login/Init (PipeWire, Quickshell, greetd, systemd na mesma consulta `pacman -Q`, versões sem época e pkgrel), Pacotes (total, explícitos, externos), Tempo ligado, Locale, todas as baterias (por `type`, sem periféricos `scope=Device`, com saúde e ciclos), frequência máxima da CPU (maior `cpuinfo_max_freq`, correta em híbridas), Memória usada/total, Swap e Armazenamento de `/` (atravessa partição, LUKS e LVM; Btrfs resolvido pela fonte do mountinfo; discos virtuais sem tipo). Toda detecção é genérica: o que não se aplica é omitido e o que não se lê vira N/A. A coleta (~200 ms, dominada por `pacman -Qqm`) passou para um job em segundo plano (`JobManager`, `App::poll` no loop principal) com a linha "Carregando informações do sistema...". `c` copia a aba como texto (`wl-copy`); o helper de clipboard saiu de `argvus-control-center-settings` para `argvus_control_center_core::clipboard`, sem mudar o comportamento das páginas SSH/GPG. | decidido (2026-10-25) |
| D67 | (`about`, Home, Settings) A barra de rolagem do About parava antes do fim: o `ScrollbarState` só chega ao fim na posição `content_length - 1`, e o último deslocamento é `linhas - viewport`. O mapeamento que a Settings já fazia virou `argvus_tui::scroll::thumb_position`, usado por About, Settings e Home (a Home tinha o mesmo defeito; como as linhas de cartões têm alturas diferentes, o último deslocamento é calculado por `last_home_scroll`). No About, o documento passa a ser quebrado e rolado pela área que o mostra (ao lado ou abaixo do logo na aba System & Hardware), e não pela largura e altura do terminal inteiro. | decidido (2026-10-25) |
| D68 | (`network`) Em **Conexões**, Enter abre o editor do perfil (a ação mais comum ao gerenciar perfis) e a tecla nova `a` conecta ou desconecta o perfil sob o cursor, mostrada no rodapé só nessa linha. **Ativar uma conexão** (como no `nmtui`) é a lista em que Enter ativa ou desativa. As páginas antigas (Interfaces, Ethernet, Wi-Fi, VPN) continuam e ganham **Editar conexão**. | decidido (2026-10-27) |
| D69 | (`network`) Tipos oferecidos: Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, túnel IP, Loopback, WireGuard e VPN por plugin (formulário OpenVPN; demais plugins como `chave=valor`), mais importação de arquivo. Ficam de fora WEP (obsoleto e inseguro), InfiniBand, MACsec e OVS; perfis desses tipos aparecem em "Outros", com nome, IPv4 e IPv6 editáveis. | decidido (2026-10-27) |
| D70 | (`network`) Escrita como o usuário da sessão, sem `pkexec`: o polkit do NetworkManager autoriza `wheel` local e pede senha aos demais pelo agente da sessão. Segredos (senha Wi-Fi/802.1X/PPPoE, chave privada, peers WireGuard com PSK, `vpn.secrets`) vão só pela stdin de `nmcli connection edit` e nunca são lidos de volta (exceto os peers, lidos com `-s` para não perder a PSK ao regravar). Como o `edit` sai com 0 mesmo recusando um valor, qualquer linha `Error` na stderr é falha. Salvar um perfil ativo pede confirmação e o reativa (`connection up`). | decidido (2026-10-27) |
| D71 | (`network`) Excluir uma bridge, bond ou team exclui também os perfis de porta que apontam para ela (por UUID ou nome de interface), como o `nmtui`; a confirmação lista as portas. As portas são escolhidas no rascunho (um toggle por dispositivo Ethernet) e criadas ou apagadas no Salvar. | decidido (2026-10-27) |
| D72 | (`network`) Perfis WireGuard passam a aparecer na página VPN (antes só tipos com "vpn" no nome). | decidido (2026-10-27) |
