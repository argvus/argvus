---
title: Introdução
description: O que é o ARGVUS e como ele é montado.
slug: pt/0.1.0/docs/intro
---

O ARGVUS é um ambiente de desktop completo e pronto para uso, focado em [Hyprland](https://hypr.land), empacotado para [Arch Linux](https://archlinux.org).

Ele não é apenas uma configuração pessoal: o projeto reúne um ambiente Wayland rico e integrado com uma barra [Waybar](https://github.com/Alexays/Waybar) no topo, lançadores [Rofi](https://github.com/davatorium/rofi) e [Wofi](https://hg.sr.ht/~scoopta/wofi), uma sidebar de informações do sistema construída com [Quickshell](https://quickshell.outfoxxed.de)/QML, um gerenciador de armazenamento removível em Rust para o Waybar e um popup de calendário nativo. Oito famílias de tema — dark, dark silver, light e slate (além das variantes float) — unificam GTK, terminal, rofi, dunst, waybar e o próprio Hyprland, todas guiadas por um sistema compartilhado de cores de destaque.

O projeto está em desenvolvimento. Os pacotes são publicados no repositório público [packages](https://github.com/argvus/packages) pelo workflow **Build and publish Arch packages**. O desktop é dividido em pacotes de componentes focados: `argvus` (configuração), `argvus-session` (launchers de sessão), `argvus-appearance` (wallpapers e fontes), `argvus-storage`, `argvus-calendar` e `argvus-greeter`.

## Próximos passos

* [Instalação](../install/) — adicione o repositório, instale, atualize e remova o desktop.
* [Temas](../themes/) — famílias de tema e cores de destaque.
* [Sessões](../sessions/gdm/) — inicie o ARGVUS a partir do GDM ou de um TTY.
* [Calendário](../calendar/) — o popup de calendário nativo da Waybar.
* [Armazenamento Removível](../storage/) — o módulo de armazenamento da Waybar.
* [Packaging](../packaging/) — layout do pacote e publicação.
