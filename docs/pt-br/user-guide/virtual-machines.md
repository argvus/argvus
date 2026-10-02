---
title: Máquinas virtuais
description: Execute ARGVUS com hardware gráfico virtual.
slug: pt/0.4.0/docs/user-guide/virtual-machines
---

`argvus-start` detecta virtualização e exporta ajustes de compatibilidade para sessões afetadas. Se a renderização falhar, inspecione o ambiente e os logs antes de adicionar overrides; veja [solução de problemas gráficos](./troubleshooting/graphics-and-virtual-machines/).

Quando a virtualização é detectada, `argvus-start` exporta variáveis de compatibilidade para a sessão, incluindo fallbacks de renderização por software, cursor por software e modificadores. Esse é o primeiro caminho a testar; não adicione um override permanente apenas porque o desktop está rodando em uma VM.

Se a detecção não for suficiente, use um override de usuário do Hyprland como fallback de diagnóstico:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

Adicione isto somente quando a pilha gráfica da VM exigir:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Faça logout e login novamente depois de alterar o ambiente da sessão. Confirme o valor ativo com `echo $LIBGL_ALWAYS_SOFTWARE`; a renderização por software pode tornar o desktop utilizável, mas pode reduzir o desempenho.
