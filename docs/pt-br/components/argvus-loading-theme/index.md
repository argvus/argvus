---
title: Overlay de carregamento
description: O overlay exibido enquanto um tema é aplicado ou a sessão inicia.
---

O overlay de carregamento é uma superfície GTK 4 de layer-shell exibida enquanto o ARGVUS aplica um
tema ou inicia a sessão do desktop. É um auxiliar do pacote `argvus`, iniciado pela sessão em
`/usr/lib/argvus/loading-theme/splash`, e não fica em `/usr/bin`.

No início da sessão o overlay mostra apenas o indicador de carregamento, sem interação, e encerra
com `SIGTERM`. Ele lê o tema selecionado do diretório de dados do ARGVUS e usa a paleta Dark padrão
do pacote se não houver seleção. Ele só informa que está pronto depois do primeiro quadro de cada
superfície que exibe, e não quando o processo inicia.
