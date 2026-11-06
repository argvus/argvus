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

O Control Center envia senhas Wi-Fi ao nmcli pela entrada padrão com `--ask`; as senhas ficam fora dos argumentos do processo e são ocultadas nos erros retornados pelo comando.

Enquanto o diálogo de senha Wi-Fi está aberto, atualizações da busca preservam o SSID e o tipo de segurança selecionados. Se a rede desaparecer ou a segurança mudar, a confirmação é interrompida com um erro e exige uma nova seleção.

DNS manual envia servidores IPv4 e IPv6 às respectivas configurações do NetworkManager antes de atualizar o arquivo do resolvedor. Symlinks de resolvedores gerenciados por serviços são preservados pela recusa de substituição do arquivo. O retorno bem-sucedido ao DNS automático restaura e remove o backup, permitindo que o próximo ciclo manual salve o estado atual do resolvedor.

O fechamento do editor de perfis de rede aguarda a conclusão do trabalho atual, para que salvar ou excluir um perfil não seja interrompido pelo descarte do editor. Atualizações de consulta em segundo plano continuam canceláveis.

Pedidos de conexão Wi-Fi compartilham a proteção das ações de rede: uma ação ou tarefa do editor de perfis em andamento termina antes que outra alteração possa começar. Mensagens de progresso e conclusão seguem o idioma selecionado.

A confirmação de remoção Bluetooth mantém o endereço original do dispositivo mesmo quando a lista é atualizada. Porcentagens de bateria do bluetoothctl são reconhecidas, e consultas de detalhes usam concorrência limitada e timeouts individuais.

Os campos de senha Wi-Fi, DNS manual e busca de rede aceitam colagem pelo terminal. Controles e quebras de linha colados são omitidos. Atalhos de caracteres com apenas Ctrl ou Alt que não são suportados não inserem letras. Os editores aceitam caracteres imprimíveis de AltGr codificados como Ctrl+Alt; Ctrl+S salva apenas sem Alt. O editor compartilhado mantém a coluna visual na navegação vertical e alinha a rolagem horizontal de caracteres largos.
