---
title: Sensores de temperatura
description: Painel ao vivo de temperaturas usando o JSON do lm_sensors.
---

`argvus sensors` abre o painel Ratatui em processo usado pelo popup de temperatura da CPU na taskbar. Ele executa `sensors -j`, agrupa as leituras pelos nomes de chips e rótulos informados pelo hardware e atualiza a cada dois segundos. O monitoramento de GPU pertence a `argvus system-monitor --gpu`.

As leituras usam Celsius. O painel mostra `max`, `crit` e alarmes quando fornecidos pelo lm_sensors. O medidor aparece somente quando existe um limite positivo. Ausência do `lm_sensors`, falhas do comando, JSON inválido e leituras vazias permanecem visíveis como estados recuperáveis; a coleta tenta novamente no próximo ciclo.

Pressione `q` ou `Esc` para fechar. A taskbar abre o painel no Kitty por meio de `argvus tui-terminal`; o título localizado da aba inferior mostra a dica para fechar. Este comando exibe detalhes de temperatura da CPU; o ícone de temperatura da GPU abre o monitor GPU-only do btop. O pacote da taskbar já depende de `lm_sensors`, portanto não é necessário pacote ou dependência separado.
