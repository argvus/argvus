---
title: Superfícies do shell
description: Ownership de taskbar, painel e telemetria.
---

`argvus-panels` fornece `Quickshell Taskbar`; `argvus-panels` possui as ações e a UX da taskbar, mas o JSONC e o CSS que ele entrega são projeções de `config.json`, e o `argvus-config` escreve a camada ativa em `data/generated/taskbar/` mais uma cópia espelhada em `data/taskbar/`; `argvus-panels` possui cards Quickshell; `argvus-panels` possui a barra opcional e a semântica dos blocos, e seu JSONC/CSS são escritos em `data/taskbar/` a partir dos defaults empacotados em uma mudança de aparência, depois do que o `argvus-config` reescreve apenas os blocos delimitados `ARGVUS_TELEMETRY_*` e de fonte. `argvus-session` possui os serviços. Não existe `argvus-shell.service` neste modelo.
