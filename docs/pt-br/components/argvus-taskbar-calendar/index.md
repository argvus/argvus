---
title: Calendário
description: Use o calendário da taskbar ARGVUS.
---

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

`argvus-taskbar-calendar` é o popup GTK4 com eventos locais, lembretes e importação/exportação ICS. O serviço é `argvus-taskbar-calendar.service`.

O calendário acompanha o tema ativo do ARGVUS, incluindo as variantes Sticky e Float. A mudança é aplicada pelo cache gerado do usuário; use `argvus taskbar-calendar reload` se um popup já aberto precisar de atualização explícita.

Quando aberto pelo módulo de data da taskbar, o popup fica preso à borda da barra: abre abaixo da taskbar no topo e acima da taskbar na parte inferior, alinhado à borda direita do módulo de data. Execuções diretas sem geometria da taskbar usam o ponteiro ou o fallback do monitor.

## Comandos

```bash
argvus taskbar-calendar                 abre o calendário
argvus taskbar-calendar toggle          mostra ou oculta (o módulo de data da Waybar usa isto)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          abre a configuração do sistema para edição (sudo)
argvus taskbar-calendar config-path    mostra o caminho da configuração
argvus taskbar-calendar reload
argvus taskbar-calendar import arquivo.ics
argvus taskbar-calendar export --output calendario.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```

## Caminhos

- Configuração: `/etc/argvus/taskbar/calendar/config.toml` (válida para o sistema; um padrão é instalado pelo pacote).
- Temas: `style.css`, `theme.css` e `themes/` no mesmo diretório.
- Dados: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (ou `~/.local/share/argvus-taskbar-calendar/`).
- Estado: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (ou `~/.local/state/...`); a chave de eventos fica ali.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (ou `~/.cache/argvus-taskbar-calendar/`).

A cor de destaque é lida de `~/.config/argvus/data/.accent-color` e aplicada em tempo de execução,
sem substituir o tema selecionado.
