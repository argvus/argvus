---
title: ARGVUS Clock
description: Relógio, relógios mundiais, cronômetro, temporizador e alarmes com som.
---

O ARGVUS Clock é um aplicativo GTK 4 com a hora atual e as ferramentas de tempo: relógios mundiais, cronômetro, temporizador e alarmes. Abra-o pelo lançador, clicando na hora da taskbar, com **SUPER + Shift + C** ou com:

```sh
argvus clock                     # abre o popup perto do ponteiro
argvus clock --toggle            # oculta o popup se estiver visível, senão o mostra
argvus clock --toggle --x 1600 --y 30  # abre junto à posição do clique na taskbar
argvus clock --page alarms       # abre numa página: clock, world, stopwatch, timer, alarms
argvus clock --background        # roda sem janela, para manter os alarmes ativos
```

O ARGVUS Clock abre como popup alinhado à taskbar, com um espaço de 4 pixels. A taskbar passa as coordenadas raiz do clique para que o popup use o monitor clicado; ao iniciar sem coordenadas, usa a posição do ponteiro. Se o ponteiro permanecer fora do popup por 5 segundos, ele fecha automaticamente; mover o ponteiro para dentro cancela a contagem. **Esc** numa aba principal ainda o oculta imediatamente, e **Esc** numa página interna volta à aba anterior. `argvus --clock` é um alias. Clicar na data da taskbar continua abrindo o calendário; o ARGVUS Clock não mostra eventos.

## Páginas

- **Relógio**: a hora em dígitos grandes (24 horas por padrão, 12 horas nas configurações, segundos opcionais), a data localizada, o fuso local e seu deslocamento de UTC, e o próximo alarme.
- **Mundo**: cidades adicionadas pela busca de cidade ou fuso IANA (acentos e maiúsculas são ignorados). Cada uma mostra a hora, a diferença para o horário local e se lá é ontem, hoje ou amanhã. O horário de verão vem do banco de fusos IANA; nada é calculado com deslocamentos fixos. As cidades podem ser reordenadas e removidas.
- **Cronômetro**: iniciar, pausar, retomar, zerar e voltas (tempo da volta e total).
- **Temporizador**: horas, minutos e segundos, ou as predefinições de 5, 10, 15, 25, 30 e 60 minutos. Pausar, retomar, reiniciar e cancelar. Ao terminar, toca como um alarme. É independente das sessões do ARGVUS Focus.
- **Alarmes**: criar, editar, ligar, desligar e excluir alarmes. Cada um tem hora, nome opcional, os dias da semana em que repete (nenhum significa que toca uma vez), um som e a duração da soneca.

Teclado: **Ctrl+1** a **Ctrl+5** trocam de página, **Ctrl+N** cria um alarme, **Ctrl+,** abre as configurações, **Esc** volta, **Ctrl+W** fecha a janela e **Ctrl+Q** sai.

## Som do alarme

Os alarmes tocam um som de verdade até você responder:

- Sons: **Carrilhão** (padrão) e **Sinal**, instalados em `/usr/share/argvus/sounds/`, um arquivo de áudio seu, ou **Sem som**, que precisa ser escolhido explicitamente. Se o seu arquivo sumir, o Carrilhão toca e uma mensagem avisa.
- **Volume do alarme** (10 a 100, padrão 80) e **Ouvir o som do alarme** ficam nas configurações; o editor tem um botão de teste para o som escolhido.
- Um alarme tocando mostra um cartão no topo da tela, acima das outras janelas, e uma notificação, ambos com **Soneca** e **Dispensar**. Com o cartão em foco, **Enter** dispensa e **S** ativa a soneca.
- A soneca toca de novo após a duração de soneca do alarme (1 a 60 minutos, padrão 10).
- Um alarme sem resposta para após **Parar de tocar após** (5, 10, 15 ou 30 minutos, padrão 10) e deixa uma notificação.
- Dois alarmes tocando juntos aparecem ambos; o segundo nunca silencia o primeiro.
- Se o som não puder ser tocado, o cartão e a janela informam o erro, que também vai para o log. O ARGVUS Clock nunca informa um som que não tocou.

A reprodução usa o suporte de mídia do GTK (GStreamer), então funciona com PipeWire, PulseAudio ou ALSA. O pacote depende de `gst-plugins-base` (Ogg Vorbis) e `gst-plugins-good` (saída de áudio).

## Quando os alarmes tocam

Os alarmes só tocam enquanto o ARGVUS Clock está em execução na sua sessão:

- Fechar a janela mantém o ARGVUS Clock rodando enquanto alarmes, um temporizador ou um cronômetro em andamento precisarem dele (**Continuar em execução com a janela fechada**, ligado por padrão). Sem nada pendente, ele encerra.
- **Iniciar com a sessão** ativa a unit de usuário `argvus-clock.service`, que roda `argvus clock --background` após o login. Vem desligado.
- Os alarmes não tocam com a sessão encerrada nem com o computador desligado ou suspenso.
- Um alarme percebido com até 5 minutos de atraso ainda toca. Com mais atraso (o relógio não estava rodando, ou o sistema estava suspenso ou desligado) ele não toca: é informado como **perdido** numa notificação e na página Alarmes. Um alarme recorrente segue para o próximo dia; um alarme único é desligado.
- Cada ocorrência é salva como tratada antes de tocar, então reabrir o ARGVUS Clock nunca toca o mesmo alarme duas vezes, e só uma instância roda por sessão.
- Alarmes recorrentes são calculados na hora local e nos dias escolhidos, então mantêm o horário nas mudanças de horário de verão. Um horário pulado na mudança toca no primeiro minuto após o intervalo; um horário que acontece duas vezes toca na primeira.
- O temporizador continua contando durante a suspensão. Se terminou enquanto o sistema dormia, toca ao acordar e informa o atraso. Depois que o ARGVUS Clock foi fechado, um temporizador ou cronômetro em andamento volta pausado.

## Caminhos

| Caminho | Conteúdo |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Configurações e relógios mundiais |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarmes |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Checkpoint do temporizador e do cronômetro |
| `/usr/lib/systemd/user/argvus-clock.service` | Início com a sessão |
| `/usr/share/argvus/sounds/` | Sons do pacote |

Os arquivos são gravados de forma atômica; um arquivo ilegível é renomeado para `<nome>.corrupt-<hora>` e os padrões são usados. As cores seguem o tema e a cor de destaque ativos do ARGVUS (consumidor de tema `time`). Os textos vêm dos domínios de tradução `clock` e `time`.
