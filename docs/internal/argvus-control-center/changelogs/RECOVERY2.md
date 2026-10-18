# Recovery 2 — Rede, Áudio e Bluetooth

Esta recuperação conecta as ações já existentes dos domínios de rede, áudio e
Bluetooth à TUI, mantendo o `ProcessRunner`, `JobManager`, o chrome compartilhado,
o tema e o sistema de idiomas existentes.

## Rede

O backend NetworkManager usa `nmcli` com argumentos separados e campos terse
explícitos. A TUI oferece refresh e scan em background, lista filtrável de
interfaces e redes Wi-Fi, conexão com senha mascarada, desconexão, esquecimento
com confirmação, VPN, alternância do rádio Wi-Fi e DNS automático/manual por
conexão. IPv4 e IPv6 são validados antes da alteração. `resolv.conf` não é
editado diretamente.

## Áudio

PipeWire/WirePlumber é acessado através de `wpctl`. Saídas, entradas e nodes
podem ser selecionados; volume é limitado a 0–100 e há ações de mute, unmute e
definição de dispositivo padrão. A ausência de `wpctl` aparece como estado
indisponível, sem controles funcionais falsos.

## Bluetooth

O backend usa comandos one-shot do `bluetoothctl`, sempre com argumentos
estruturados. A TUI oferece estado do adaptador, power, discoverable, scan,
pair, trust/untrust, connect/disconnect e remoção com confirmação. Informações
de paired/trusted/connected/bateria são obtidas por `bluetoothctl info`.

Pairing que exige PIN, passkey ou confirmação numérica ainda depende de um
Agent BlueZ D-Bus, que não faz parte desta recuperação. A interface não finge
que esse fluxo está implementado: o erro real do backend é exibido. A próxima
extensão deve registrar e remover o Agent de forma explícita, sem recorrer a
uma sessão interativa frágil de `bluetoothctl`.

## Segurança e execução

Todas as operações lentas são jobs de background. Não há `sudo`, `sh -c` ou
`bash -c`. Senhas Wi-Fi têm vida útil limitada ao job de conexão, aparecem
mascaradas e não são incluídas em logs, debug ou mensagens de erro. Textos
recebidos dos backends passam pela sanitização central antes de renderização.
