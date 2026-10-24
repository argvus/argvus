---
title: Rede e Bluetooth
description: Gerencie NetworkManager e Bluetooth.
---

O subcomando `argvus network` consulta e controla o NetworkManager, e `argvus bluetooth` consulta e controla o estado e a disponibilidade do adaptador Bluetooth. Os dois rodam em processo dentro do `argvus`; `networkctl` e `bluetoothctl` continuam como aliases. O Control Panel e a Waybar usam esses comandos. Consulte `--help` para os subcomandos instalados.

O Control Center separa o status de rede das páginas de configuração: **Network → Status** resume o estado atual, enquanto interfaces, Wi-Fi, Ethernet, VPN, DNS, proxy e firewall são rotas separadas quando os providers estão disponíveis. **Bluetooth → State**, **Devices** e **Pair** cobrem o estado do adaptador e as ações sobre dispositivos. Essas páginas executam ações pelos providers; a taskbar e o Control Panel oferecem a superfície de status/ações rápidas.

Alterações de rede e Bluetooth podem exigir autorização e podem ser aplicadas imediatamente pelo NetworkManager ou serviço Bluetooth. Elas não são o mesmo que alterar o indicador da taskbar.

## Gerenciamento de rede pela linha de comando

Use `argvus networkctl` para consultar e controlar a rede sem a interface gráfica:

```sh
# Mostrar status de rede
argvus networkctl status

# Ativar/desativar rede
argvus networkctl enable
argvus networkctl disable

# Controle de Wi-Fi
argvus networkctl wifi on
argvus networkctl wifi off

# Abrir o gerenciador de rede
argvus networkctl manager
```

## Gerenciamento de Bluetooth pela linha de comando

Use `argvus bluetoothctl` para o adaptador Bluetooth:

```sh
# Mostrar o estado do adaptador (linhas key=value)
argvus bluetoothctl status

# Sair com 0 quando há um adaptador disponível, 1 caso contrário
argvus bluetoothctl available

# Ligar, desligar ou alternar o adaptador
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Abrir o gerenciador de Bluetooth (Blueman)
argvus bluetoothctl manager
```

Quando o kernel não lista nenhum controlador em `/sys/class/bluetooth`, `status` responde `status=no-controller` na hora, sem consultar o `bluetoothctl` (que, com o serviço parado, ficaria esperando o BlueZ). O módulo da taskbar e os cards do Control Panel leem esse mesmo status.

Use `--help` em qualquer comando para ver opções e subcomandos adicionais específicos da sua versão instalada.
