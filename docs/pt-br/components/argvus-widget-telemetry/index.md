---
title: Telemetria do widget
description: Os blocos de telemetria da Waybar e a sua alternância, pelo argvus widget-telemetry.
---

O widget de telemetria é a informação do sistema mostrada na taskbar (cabeçalho, teclas, sistema,
CPU e GPU, memória, armazenamento, processos, rede e o painel do desenvolvedor). Ele é um comando
em processo do pacote `argvus`: não há coletores de shell.

## Comandos

```text
argvus widget-telemetry block BLOCO ...   mostra o texto de um bloco (usado pela barra)
argvus widget-telemetry toggle ...        liga e desliga os blocos, e define a ordem deles
```

`block` é o caminho quente da barra: é chamado a cada atualização. `toggle` muda quais blocos a
barra mostra e move a janela de rolagem.

Os blocos são `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` e
`dev_dashboard`. A taskbar também usa as sondas `cpu-temp` e `gpu`.
