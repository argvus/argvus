---
title: Rede e Bluetooth
description: Gerencie NetworkManager e Bluetooth.
slug: pt/0.4.0/docs/user-guide/hardware/networking
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

Use `argvus bluetoothctl` para controlar dispositivos e adaptador Bluetooth:

```sh
# Mostrar estado do adaptador Bluetooth
argvus bluetoothctl status

# Ativar/desativar Bluetooth
argvus bluetoothctl enable
argvus bluetoothctl disable

# Listar dispositivos pareados
argvus bluetoothctl list

# Conectar/desconectar dispositivos
argvus bluetoothctl connect <device-name-or-address>
argvus bluetoothctl disconnect <device-name-or-address>

# Confiar ou não confiar em um dispositivo
argvus bluetoothctl trust <device-address>
argvus bluetoothctl untrust <device-address>
```

Use `--help` em qualquer comando para ver opções e subcomandos adicionais específicos da sua versão instalada.
