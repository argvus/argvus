---
title: Sessões GDM
description: Inicie o ARGVUS a partir do GDM.
---

Depois de instalar, selecione a sessão **ARGVUS** no GDM/display manager.

Os launchers de sessão são fornecidos pelo pacote `argvus-session`. O `argvus-session` prepara o ambiente e executa o `argvus-start`, que inicia o Hyprland com `start-hyprland` e carrega `$XDG_CONFIG_HOME/hypr/hyprland.lua` apenas quando um override completo do usuário existe. Caso contrário, ele inicia a partir do empacotado `/usr/share/argvus/hypr/hyprland.lua`. Preferências do usuário como tema, accent e espaçamento são lidas de `$XDG_CONFIG_HOME/argvus`, e a configuração de runtime gerada é lida de `$XDG_STATE_HOME/argvus/config`.

Se o GDM voltar para a tela de login, verifique os logs:

```sh
sed -n '1,320p' ~/.local/state/argvus/session.log
journalctl --user -b -u 'wayland-wm@*' --no-pager
```

No VirtualBox, desligue a VM e selecione **VMSVGA**, pelo menos **128 MB** de memória de vídeo e **Enable 3D Acceleration** em `Settings > Display`. Mantenha drivers e Mesa atualizados no Arch guest:

```sh
sudo pacman -Syu --needed virtualbox-guest-utils mesa
sudo systemctl enable --now vboxservice.service
```

O launcher detecta máquinas virtuais, limpa overrides de GPU físicas, habilita os fallbacks de renderização aceitos pelo Hyprland e usa a configuração Lua empacotada atual, a menos que o usuário tenha fornecido explicitamente um override completo do Hyprland. O tema escolhido pelo usuário continua carregando. O suporte ao Hyprland em VMs ainda depende da GPU virtual fornecida pelo hypervisor.

O pacote não escreve em `$HOME` durante a instalação. Os defaults ficam em `/usr/share/argvus/`.
