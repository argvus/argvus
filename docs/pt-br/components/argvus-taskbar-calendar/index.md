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

## Eventos e lembretes

Os eventos começam desativados. Ative-os no popup em **Configurações > Eventos**; a
escolha fica no diretório de estado e também ativa e inicia o
`argvus-taskbar-calendar.service`, que mostra os lembretes. Ao desativar, o serviço para.

- **+ EVENTO** (ou `n`) abre o editor no dia selecionado. Eventos podem ser criados e
  alterados a partir de hoje; um evento de um dia passado ainda pode ser aberto e excluído.
- Um evento novo começa às 09:00 e dura o tempo padrão definido nas Configurações. Mudar a
  hora inicial mantém a duração; uma hora final menor que a inicial significa que o evento
  termina no dia seguinte.
- Todo evento mostra uma notificação com som no início, independentemente das caixas
  **Lembrete** e **Repetir**. Um evento salvo depois de começar (um evento de dia inteiro
  criado durante o dia, por exemplo) avisa em até 30 segundos.
- **Lembrete** acrescenta um aviso antecipado, o tempo escolhido antes do início (no início,
  se esse horário já passou).
- Num evento de dia inteiro o campo de hora vira **AVISAR ÀS**: o evento ocupa o dia todo e
  avisa nesse horário (00:00 avisa no começo do dia). O horário é mantido ao reabrir o evento.
- Em eventos de dia inteiro, **Repetir** avisa de novo no intervalo escolhido a partir desse
  horário.
- Editar um evento sem mudar o início não repete lembretes já mostrados.
- As notificações seguem o idioma do calendário (Configurações > Idioma).
- Um dia mostra um ponto (`•`) quando há evento nele, incluindo todos os dias de um evento
  que atravessa a noite ou dura vários dias. Os dias seguem o fuso horário local.
- Eventos locais são removidos automaticamente 10 minutos depois de terminarem.

Arquivos `.ics` importados mantêm datas, fusos horários (`TZID`) e alarmes (`VALARM`).
Eventos importados também avisam no início, além dos próprios alarmes. Eventos recorrentes (`RRULE`) são
guardados e exportados, mas só a primeira ocorrência aparece e gera lembrete. Depois de uma atualização do pacote, o serviço de lembretes sai e o systemd inicia a versão
nova em até 30 segundos. `sync` apenas
informa os calendários remotos configurados; a sincronização CalDAV ainda não existe.

## Caminhos

- Configuração: `/etc/argvus/taskbar/calendar/config.toml` (válida para o sistema; um padrão é instalado pelo pacote).
- Estilos: `style.css` e `theme.css` no mesmo diretório; as cores do tema ativo do ARGVUS vêm do build do tema, pelo cache abaixo.
- Dados: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (ou `~/.local/share/argvus-taskbar-calendar/`).
- Estado: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (ou `~/.local/state/...`); a chave de eventos fica ali.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (ou `~/.cache/argvus-taskbar-calendar/`).

A cor de destaque é lida de `~/.config/argvus/data/.accent-color` e aplicada em tempo de execução,
por cima das cores do tema.
