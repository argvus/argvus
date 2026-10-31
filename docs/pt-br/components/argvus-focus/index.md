---
title: ARGVUS Focus
description: Pomodoro e sessões de foco com pausas, notificações e um histórico local das sessões.
---

O ARGVUS Focus é um aplicativo GTK 4 para trabalhar em sessões de foco separadas por pausas. Ele abre como popup junto à taskbar. Abra-o pelo lançador, clicando no indicador da taskbar, com **SUPER + Shift + F** ou com:

```sh
argvus focus              # abre o popup perto do ponteiro
argvus focus --toggle     # oculta o popup se estiver visível, senão o mostra
argvus focus --toggle --x 1600 --y 30  # abre junto à posição do clique na taskbar
argvus focus --start      # inicia ou retoma a fase atual sem abrir a janela
argvus focus --pause      # pausa a fase em andamento
argvus focus status --waybar           # imprime um status JSON Waybar e encerra
argvus focus status --waybar --follow  # transmite as mudanças de status para Waybar
```

A taskbar passa as coordenadas raiz do clique para que o popup use o monitor clicado e se alinhe à barra; ao iniciar sem coordenadas, usa a posição do ponteiro. Se o ponteiro permanecer fora do popup por 5 segundos, ele fecha automaticamente; mover o ponteiro para dentro cancela a contagem. **Esc** na página principal ainda o oculta imediatamente, e **Esc** numa subpágina volta à página anterior. O indicador da taskbar mostra só o ícone quando ocioso e o tempo restante `MM:SS` durante uma fase, com a cor da fase; uma fase pausada mantém o tempo restante e sinaliza a pausa. O comando `status` roda dentro do `argvus`, sem iniciar GTK. `argvus --focus` é um alias. Só uma instância do Focus roda por sessão: uma segunda execução repassa as opções para a que já está aberta.

ARGVUS Focus, ARGVUS Clock e o Control Panel compartilham a mesma lateral da tela. Ao abrir o Focus, o Clock e o Control Panel são fechados primeiro; ao abrir qualquer uma dessas outras superfícies, o Focus é fechado.

## Sessões

- O modo **Pomodoro** alterna foco e pausas curtas e fecha cada ciclo com uma pausa longa. As durações padrão são 25, 5 e 15 minutos, com pausa longa após 4 sessões de foco.
- O modo **Sessão única** roda uma sessão de foco com duração própria (45 minutos por padrão), sem pausas.
- A janela mostra a fase (foco, pausa curta, pausa longa, cada uma com sua cor), o tempo restante dentro de um anel de progresso, a posição no ciclo e o total do dia.
- **Iniciar foco / Iniciar pausa**, **Pausar**, **Retomar**, **Cancelar sessão** (abandona a fase sem registrar nada) e **Concluir agora** (encerra a sessão de foco antes e registra o tempo realmente usado). Durante uma pausa, **Pular pausa** volta ao foco.
- Quando uma fase termina, a próxima espera por você, a menos que **Iniciar pausas automaticamente** ou **Iniciar o foco automaticamente após a pausa** esteja ligado.

Teclado: **Espaço** inicia ou pausa, **Ctrl+,** abre as configurações, **Ctrl+H** o histórico, **Esc** volta, **Ctrl+W** oculta a janela e **Ctrl+Q** sai.

## Configurações

As predefinições (Clássico 25/5/15, Estendido 50/10/30, Trabalho profundo 90/20/30) preenchem as durações, que continuam editáveis. As configurações também têm as sessões antes da pausa longa, os inícios automáticos, a notificação ao fim de cada fase e os **sons da sessão**: um sino curto com volume próprio e o botão **Testar som**. Esses sons são independentes dos sons de alarme do [ARGVUS Clock](/pt/docs/components/argvus-clock/).

## Histórico

Cada sessão de foco concluída é registrada com início, fim e tempo usado; pausas e sessões canceladas não são registradas. A página de histórico mostra as sessões e os minutos de hoje, os totais e as sessões recentes. Registros com mais de 365 dias são descartados. **Limpar histórico** pede confirmação.

## Janela, segundo plano e suspensão

- Fechar a janela durante um ciclo apenas a oculta: a sessão continua e avisa quando a fase termina. Sem nada em andamento, fechar encerra o aplicativo.
- Se o sistema for suspenso durante uma fase, ela é pausada no momento da suspensão e uma mensagem informa por quanto tempo o sistema ficou suspenso. O tempo suspenso não conta como foco.
- A sessão é salva a cada mudança e a cada 30 segundos. Se o ARGVUS Focus for fechado durante uma fase (logout, falha, **Ctrl+Q**), ela volta pausada nesse ponto na próxima abertura; o aplicativo nunca finge que o timer continuou enquanto não podia avisar.
- O tempo é medido no relógio de boot do sistema, então mudar a hora ou o fuso não encurta nem alonga uma fase.

## Caminhos

| Caminho | Conteúdo |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Configurações |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Sessões concluídas |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Checkpoint da sessão atual |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Sino da sessão |

Os arquivos são gravados de forma atômica. Um arquivo que não pode ser lido é renomeado para `<nome>.corrupt-<hora>` e os padrões são usados; uma mensagem diz para onde ele foi.

As cores seguem o tema e a cor de destaque ativos do ARGVUS (consumidor de tema `time`). Os textos vêm dos domínios de tradução `focus` e `time`.

O calendário da taskbar, ARGVUS Focus, ARGVUS Clock e Control Panel são exclusivos entre si: abrir um fecha os demais.
