---
title: Overlay de carregamento
description: O overlay exibido enquanto um tema é aplicado ou a sessão inicia.
---

O overlay de carregamento é uma superfície GTK 4 de layer-shell exibida enquanto o ARGVUS aplica um
tema ou recarrega a sessão diretamente. É um auxiliar do pacote `argvus` em
`/usr/lib/argvus/loading-theme/splash`, e não fica em `/usr/bin`.

O login não o usa: o splash do greeter e o indicador do início da sessão são o modo Wayland do helper
do handoff, que desenha o mesmo indicador sem GTK e mostra o primeiro quadro em milissegundos (veja
[Greeter, bloqueio e splash](/pt/docs/developer-guide/subsystems/greeter-lock-and-splash/)). Com
`--spinner-only` o overlay mostra apenas esse indicador e encerra com `SIGTERM`. Ele lê o tema selecionado do diretório de dados do ARGVUS e usa a paleta Dark padrão
do pacote se não houver seleção. Ele só informa que está pronto depois do primeiro quadro de cada
superfície que exibe, e não quando o processo inicia.
