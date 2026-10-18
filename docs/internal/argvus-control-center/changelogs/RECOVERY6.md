# Recovery 6 — Storage, SMART, Diagnostics e Readonly UX

Esta recuperação adiciona os domínios read-only de Armazenamento e
Diagnóstico. A descoberta de block devices usa `lsblk --json`; mounts usam
`findmnt --json`; uso de filesystem é obtido por `df -P -k` com argumentos
separados; swap é lido de `/proc/swaps`. Pseudo-filesystems não entram na
lista principal. A coleta ocorre em `JobManager`, nunca durante renderização.

SMART é somente leitura e só aceita dispositivos `/dev` identificados pelo
backend. `smartctl` é opcional; ausência ou falta de suporte é apresentada
como informação indisponível, sem iniciar testes destrutivos.

Diagnóstico possui modelo de checks com severidade, evidência e rota. Ele
consome o snapshot de Storage e mantém Packages parcial: updates e lock podem
ser informativos, mas provider selection, AUR review, mirrors e transactions
não são tratados como concluídos nem como defeitos do sistema.

O contrato readonly separa dados informativos de rows acionáveis. Campos
informativos não recebem seleção nem anunciam Enter; listas que abrem detalhes
continuam selecionáveis. Ações indisponíveis são distintas de informação
readonly.

Limitações: a primeira integração diagnóstica ainda não expõe todos os
snapshots específicos de Network, Audio, Bluetooth, Boot e ARGVUS como
providers dedicados; esses domínios continuam sendo a fonte de verdade para
suas próprias telas. Nenhuma ação de storage, SMART, reparação ou auto-fix é
executada.
