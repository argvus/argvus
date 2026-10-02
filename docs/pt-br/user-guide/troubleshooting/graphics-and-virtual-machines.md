---
title: Gráficos e máquinas virtuais
description: Diagnostique falhas de renderização.
slug: pt/0.4.0/docs/user-guide/troubleshooting/graphics-and-virtual-machines
---

Inspecione o ambiente da sessão e os logs do compositor. Em máquinas virtuais, verifique as variáveis exportadas por `argvus-start`. Não force renderização por software sem confirmar a necessidade.

Verificações úteis:

```sh
echo "$ARGVUS_VIRTUALIZATION"
echo "$LIBGL_ALWAYS_SOFTWARE"
echo "$WLR_RENDERER_ALLOW_SOFTWARE"
argvus-sessionctl logs
```

Para uma VM que exige um fallback manual, adicione `hl.env("LIBGL_ALWAYS_SOFTWARE", "1")` a `~/.config/argvus/data/hypr/user.lua` e faça logout/login novamente. Isso altera o ambiente da sessão e pode reduzir o desempenho; remova o override quando o driver gráfico virtual não precisar mais dele.
