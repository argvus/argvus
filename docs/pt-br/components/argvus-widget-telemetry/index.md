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
`dev_dashboard`. A taskbar também usa as sondas `cpu-temp` e `gpu`. A sonda de GPU aceita `--present` (uma GPU suportada está visível em DRM/sysfs), `--available` (há uma GPU e o comando btop para abrir o painel), `--available-temp` (uma temperatura atribuível pode ser lida) e `--temp` (a maior temperatura válida ou `Indisponível`).

As temperaturas de GPU são associadas ao dispositivo PCI correspondente. Leituras NVIDIA proprietárias usam os IDs de barramento de `nvidia-smi`/NVML; leituras NVIDIA nouveau, AMD e Intel usam hwmon somente quando o vínculo do dispositivo em sysfs corresponde exclusivamente àquela GPU. A temperatura da taskbar é o maior valor válido entre as GPUs detectadas. Leituras ausentes, inválidas ou ambíguas permanecem indisponíveis. Temperaturas NVIDIA proprietárias dependem do provider do driver; temperaturas AMD e Intel dependem de o kernel expor um sensor hwmon vinculado. A GPU Intel não fornece temperatura pelo painel do btop 1.4.7.

## Início com a sessão

A unit de usuário `argvus-widget-telemetry.service` só inicia com a sessão quando o widget está
ativado. O interruptor mestre é `effects.widget_telemetry_enabled` na configuração, que vem
ligado por padrão, então uma conta nova inicia o widget no login.

Depois que a alternância grava `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, esse
arquivo passa a decidir. Só uma primeira linha `enabled` inicia o widget. Os arquivos antigos
`desktop-telemetry-state` e `sysinfo-state` continuam sendo lidos quando o arquivo atual não
existe.
