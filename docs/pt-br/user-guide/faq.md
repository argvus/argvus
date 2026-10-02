---
title: Perguntas frequentes
description: Dúvidas comuns sobre o ARGVUS, sua arquitetura e seu uso diário.
slug: pt/0.4.0/docs/user-guide/faq
---

Esta página responde às dúvidas mais comuns sobre o que é o ARGVUS, como ele é distribuído e onde configurá-lo. Para procedimentos detalhados, siga os links de cada resposta.

## O que é o ARGVUS?

O ARGVUS é um desktop environment modular para Wayland e Arch Linux, construído ao redor do Hyprland. Ele coordena a sessão gráfica, as superfícies do shell, os aplicativos de configuração, a aparência, os serviços do desktop e as ferramentas integradas por meio de uma coleção de pacotes.

Ele é mais do que uma configuração do Hyprland ou uma coleção de dotfiles pessoais: fornece um entrypoint de sessão suportado, estado compartilhado de configuração e aparência e responsabilidades definidas para componentes como taskbar, Control Center, notificações, rede e displays. Consulte a [introdução](/pt/docs/introduction/) para entender como essas partes trabalham juntas.

## O ARGVUS é gratuito?

Sim. O ARGVUS pode ser baixado e usado gratuitamente, e seus principais pacotes do projeto são distribuídos como software livre sob a [GNU General Public License versão 3](https://www.gnu.org/licenses/gpl-3.0.html). Isso garante as liberdades de executar, estudar, modificar e redistribuir o software coberto, respeitando os termos da licença.

O ARGVUS é distribuído como pacotes Arch Linux assinados. O software não tem preço de compra, mas o download dos pacotes pode envolver custos normais de internet ou hospedagem. Componentes e aplicativos de terceiros podem ter suas próprias licenças.

## O ARGVUS é uma distribuição Linux?

Não. O ARGVUS é a camada de ambiente desktop instalada sobre o Arch Linux. O Arch Linux fornece o sistema operacional, o kernel, o gerenciador de pacotes e os pacotes-base; o Hyprland fornece a composição Wayland; e o ARGVUS coordena a sessão do desktop e seus componentes integrados.

Veja a [instalação](/pt/docs/getting-started/installation/) para o modelo suportado e [O que é — e o que não é — o ARGVUS](/pt/docs/introduction/#o-que-é--e-o-que-não-é--o-argvus) para a distinção arquitetural.

## Por que um desktop environment em vez de uma distribuição?

Manter o ARGVUS como um desktop environment dá ao usuário mais controle sobre o restante do sistema. Uma distribuição normalmente escolhe e mantém uma base completa, incluindo kernel, seleção de pacotes, repositórios, padrões e políticas do sistema. O ARGVUS concentra-se na experiência do desktop e deixa essas decisões mais amplas para o usuário e para a instalação subjacente do Arch Linux.

Esse modelo também mantém o projeto modular: gerenciamento da sessão, configuração do Hyprland, aparência, taskbar, configurações e provedores podem ser desenvolvidos e empacotados separadamente, trabalhando juntos como um único desktop.

## Quais sistemas são compatíveis?

A base atualmente suportada é o Arch Linux. O ARGVUS é empacotado para Arch Linux e depende dos componentes de Wayland, Hyprland e do sistema descritos nos [requisitos de instalação](/pt/docs/getting-started/installation/requirements/).

Outras distribuições podem oferecer alguns dos mesmos componentes upstream, mas isso não torna o conjunto completo de pacotes ou a integração de sessão do ARGVUS oficialmente suportados nelas.

## Como faço para instalar o ARGVUS?

Siga o [guia de instalação](/pt/docs/getting-started/installation/). Ele explica os requisitos, como configurar o repositório assinado de pacotes do ARGVUS, como instalar o conjunto de pacotes e como iniciar a primeira sessão.

O ARGVUS é entregue como um conjunto coordenado de pacotes Arch. O pacote `argvus` é o principal alvo de instalação e traz os componentes do desktop que ele coordena.

## Como faço para alterar as configurações no ARGVUS?

Use o **Control Center** para a configuração persistente do desktop, incluindo temas, wallpapers, efeitos, layout, fontes, teclado, entrada, idioma, região e aplicativos padrão. As formas mais eficientes de abri-lo são o atalho `SUPER + Alt + C` ou o lançador com `SUPER + D`: digite **ARGVUS Control Center**. Você também pode digitar `argvus` para listar os aplicativos do ARGVUS.

Como alternativa complementar, você pode abri-lo pelo terminal com:

```sh
argvus --control-center
```

Também é possível abrir uma área diretamente com `argvus-control-center`, por exemplo `argvus-control-center appearance themes`. Execute `argvus-control-center --help` para ver as rotas disponíveis na versão instalada.

Use o **Control Panel** para consultar o estado atual e executar ações frequentes, como volume, brilho, rede, notificações, energia e controles da sessão. Ele não é uma segunda base de configurações. Veja [Onde configurar as coisas](/pt/docs/user-guide/where-to-configure/) e [Control Center](/pt/docs/argvus-control-center/) para entender a diferença.

## Onde ficam armazenadas as configurações do ARGVUS?

A configuração pertencente ao usuário fica abaixo de `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). A configuração lógica canônica é mantida ali, enquanto os arquivos de consumo derivados para Hyprland, Waybar e o widget de telemetria, GTK, Qt6ct, Quickshell, Rofi, Dunst, Hyprlock, Yazi, Superfile, terminais, fontes, efeitos, calendários, entrada, teclado, energia e dispositivos removíveis ficam em `~/.config/argvus/data/generated/`. Essa árvore é escrita apenas pelo `argvus-config` e reconstruída a partir da configuração canônica; o `argvus-appearance` confirma as mudanças de aparência e depois reconcilia somente os consumidores externos que não conseguem ler a configuração canônica sozinhos.

Não edite arquivos gerados como método permanente de configuração. Use o Control Center ou o caminho de override nativo documentado para o componente que você deseja personalizar. Consulte [arquivos de configuração](/pt/docs/reference/configuration-files/) e [configuração e estado](/pt/docs/developer-guide/architecture/configuration-and-state/).

## O que fazer se uma alteração não for aplicada?

Primeiro confirme se a alteração foi salva ou aplicada na página responsável. Depois verifique se o componente afetado precisa de reload da sessão ou do serviço. Uma alteração confirmada é projetada nos arquivos de consumo pelo `argvus-config`, mas projetá-los não garante que todo consumidor em execução os tenha recarregado. O `argvus-sessionctl reload` aplica uma geração já confirmada a partir do manifesto de projeção em vez de projetar de novo, e pula o reload gráfico quando o estado canônico não mudou.

Use o [guia de solução de problemas](/pt/docs/user-guide/troubleshooting/) correspondente para instalação, inicialização da sessão, gráficos, displays, Hyprland, temas e wallpapers. Ao relatar um problema, inclua o componente, a configuração alterada, a página ou comando utilizado e a saída do status ou diagnóstico relevante.

## Onde posso aprender mais?

* [Primeira configuração](/pt/docs/user-guide/getting-started/) para o fluxo recomendado após a primeira sessão.
* [Guia do usuário](/pt/docs/user-guide/) para instruções específicas de cada recurso.
* [Referência](/pt/docs/reference/) para comandos, caminhos, serviços e contratos de configuração.
* [Guia do desenvolvedor](/pt/docs/developer-guide/) para arquitetura, responsabilidade dos pacotes e contribuição.
