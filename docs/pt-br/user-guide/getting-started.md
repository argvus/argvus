---
title: Primeira configuração
description: Um roteiro prático para configurar o ARGVUS após a instalação.
slug: pt/0.4.0/docs/user-guide/getting-started
---

Após a primeira sessão gráfica, você não precisa configurar todas as partes do ARGVUS. Comece pelos ajustes que mudam a sensação do desktop e configure hardware e serviços conforme necessário.

## Uma ordem útil

1. Leia a [visão geral do desktop](/pt/docs/user-guide/desktop/) para identificar janelas, áreas de trabalho, taskbar, Control Panel e lançador.
2. Abra o [Control Center](/pt/docs/argvus-control-center/) e escolha tema, modo, acento e wallpaper em **Aparência**.
3. Ajuste [janelas e layout](/pt/docs/argvus-hyprland/windows-and-layout/) se os gaps, bordas ou espaços da taskbar não forem adequados à sua tela.
4. Configure a [taskbar](/pt/docs/argvus-taskbar/taskbar/) e escolha quais cards do [Control Panel](/pt/docs/argvus-control-panel/) devem ficar visíveis.
5. Revise os [atalhos de teclado](/pt/docs/argvus-hyprland/keyboard-shortcuts/) e altere apenas os atalhos que realmente usa.
6. Configure [mouse e touchpad](/pt/docs/argvus-hyprland/input/), displays, rede e energia nas páginas de configuração correspondentes.

Os quatro primeiros passos são personalização opcional. Os demais normalmente só são necessários quando o hardware, idioma ou fluxo de trabalho exigir.

## Dois lugares para alterar configurações

Use o **Control Center** para configurações que descrevem como o ARGVUS deve ser configurado: aparência, layout, fontes, entrada, atalhos, idioma, região e aplicativos padrão. Use o **Control Panel** para status e ações frequentes da sessão, como volume, brilho, estado da rede, notificações, energia e sessão.

Veja [Control Center e Control Panel](/pt/docs/argvus-control-center/) para a distinção completa e [Onde configurar as coisas](/pt/docs/user-guide/where-to-configure/) para um índice rápido.

## Alterações, persistência e recuperação

A maioria das alterações do Control Center é aplicada enquanto a sessão está ativa e salva como estado de usuário do ARGVUS. As mudanças de aparência são confirmadas na configuração canônica, e o `argvus-config` projeta a partir dela os arquivos consumidos pelo Hyprland e pelo shell. Um toggle do Control Panel pode ser uma ação direta ou uma alteração de estado da sessão, e não uma configuração permanente.

Quando uma página oferecer **Restaurar padrões**, use essa ação em vez de apagar arquivos gerados. Aparência possui ações separadas para restaurar acento, wallpaper e valores de layout; a página de atalhos possui sua própria restauração. Não existe um único botão de restauração universal para toda a configuração do desktop.

## Próximos passos

* [Aparência](/pt/docs/user-guide/appearance/) — temas, wallpapers, efeitos e layout.
* [Layout do desktop](/pt/docs/argvus-hyprland/windows-and-layout/) — entenda gaps, bordas e espaço dos painéis.
* [Dispositivos de entrada](/pt/docs/argvus-hyprland/input/) — controles de mouse e touchpad.
* [Solução de problemas de tema e wallpaper](/pt/docs/user-guide/troubleshooting/theme-and-wallpaper/) — recuperação quando uma alteração visual não ficou como esperado.
