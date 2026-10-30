---
title: Luz noturna
description: Ajuste a temperatura de cor e a programação por horários locais no Hyprland.
---

Abra **Central de Controle > Área de trabalho e entrada > Monitores > Luz noturna**, busque **Luz noturna** ou execute `argvus control-center displays night-light`.

A luz noturna e a programação começam desligadas. Escolha de **1000 a 20000 Kelvin** (padrão **4500 K**). Valores menores produzem cores mais quentes. A página mostra a ativação manual salva, o estado efetivo e a próxima transição. Enter edita um valor; Esc cancela a edição ou volta a Monitores. Valores inválidos são recusados antes da gravação. Mudanças são salvas e aplicadas imediatamente. Se a aplicação falhar após a gravação, a preferência salva continua visível junto ao estado efetivo e ao erro.

## Programação e ações rápidas

A programação fixa usa o fuso horário local atual. O início está incluído e o fim excluído; **20:00–07:00** atravessa a meia-noite. Os horários devem ser diferentes e usar `HH:MM`. Não há localização nem cálculo solar.

Use o cartão **Display** do Control Panel ou **SUPER + Shift + B** para alternar rapidamente. O seletor preserva todas as ações de brilho e acrescenta a luz noturna quando o backend está disponível, inclusive sem um backend de brilho. Não há módulo permanente na taskbar.

Sem programação, a ativação manual é persistente. Com programação, a ação rápida cria uma sobreposição volátil até a próxima transição. Alterar temperatura ou programação remove a sobreposição. Logout, nova instância do compositor ou mudança da próxima transição (inclusive por troca de fuso) também a invalidam.

## Comandos e estado

Todas as interfaces usam o mesmo comando público:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```

`status` retorna JSON versão 1: `available`, `unavailable_reason`, `settings` (enabled, temperature, schedule_enabled, start, end), `active`, `effective_temperature`, `mode` (`manual` ou `scheduled`), `manual_override`, `next_transition` (ISO 8601 com offset), `minimum_temperature` e `maximum_temperature`. Os campos efetivos são nulos na indisponibilidade; a temperatura efetiva é nula quando há identidade. Falhas de disponibilidade aparecem no status; falhas nas ações retornam código não zero e diagnóstico.

A preferência canônica é `/displays/night_light`, gravada atomicamente por `argvus config` em `$XDG_CONFIG_HOME/argvus/config/displays.json`. Sobreposições e o lock das ações ficam em `$XDG_RUNTIME_DIR/argvus/night-light/`. Os arquivos gerados de monitores são independentes. Não edite arquivos gerados para configurar este recurso.

## Runtime

Requer Hyprland e **hyprsunset >= 0.4.0**. A sessão inicia uma única instância pelo serviço upstream `hyprsunset.service`, aplica o estado salvo por `argvus-night-light-init.service` e avalia a programação a cada minuto por `argvus-night-light.timer` e `argvus-night-light.service`. Mudanças do relógio e fuso também disparam a avaliação. Sem programação, a operação periódica preserva o estado manual. Aplicações repetidas consultam identidade/temperatura e pulam gravações sem mudança. Não há reload do compositor.

O ARGVUS fornece um drop-in systemd para o serviço upstream: inicia com `--identity` e isola a descoberta de configuração em `/usr/share/argvus/session/config/night-light`. Isso evita perfis nativos concorrentes e preserva arquivos pessoais `hyprsunset.conf`. O drop-in se aplica ao serviço enquanto o pacote de sessão estiver instalado, inclusive fora do ARGVUS. Overrides systemd do usuário podem ter precedência; confira `systemctl --user cat hyprsunset.service` nesses casos. O Hyprsunset 0.4.0 rejeita o argumento de caminho separado de `--config`, por isso o isolamento usa variáveis XDG locais ao serviço.

O backend usa a IPC oficial do Hyprsunset (`temperature`, `identity`, `identity get`), com timeout e leitura após aplicação. Não usa shader, backend alternativo, gravação de configuração do sistema nem privilégios elevados. Controles indisponíveis ficam desativados na Central de Controle e ocultos no cartão Display.
