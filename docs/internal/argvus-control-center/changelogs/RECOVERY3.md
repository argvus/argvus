# Recovery 3 — Hardware, Serviços e Journal

Esta recuperação conecta as implementações existentes de Hardware, systemd e
journal à TUI compartilhada do Control Center. Ela não adiciona novas
categorias nem altera os domínios de Rede, Áudio e Bluetooth.

## Hardware

O backend coleta informações em jobs de background a partir de `/proc`, `/sys`,
DMI e ferramentas opcionais detectadas por `Capabilities`. Resumo, CPU, GPU,
Memória, Energia e Dispositivos usam o chrome e as listas compartilhadas da
aplicação. GPU e dispositivos possuem páginas de detalhes; CPU e Energia
possuem seleção de governor/perfil quando o mecanismo detectado oferece essa
operação.

Governors são validados no helper privilegiado contra os governors disponíveis
em sysfs. Perfis do `power-profiles-daemon` são descobertos dinamicamente e
`powerprofilesctl` é usado somente com argumentos separados. A alteração de
governor e perfil atualiza o snapshot após o job terminar. A detecção de GPU,
vmwgfx, render nodes, bateria e firmware permanece tolerante à ausência de
hardware ou ferramentas opcionais.

## Serviços systemd

Sistema, usuário e unidades falhas são carregados com `systemctl` em formato
JSON, sem parsing de tabelas humanas. A lista possui filtros por estado e busca
por nome/descrição. Detalhes exibem estado, PID principal e unit fragment
quando fornecidos pelo systemd.

Ações de unidades de sistema passam por `SystemSettingsOperation`, com ações
enumeradas e validação estrita de nomes `.service`. Ações de usuário executam
`systemctl --user` como o usuário atual, sem elevação. Start, stop, restart,
enable, disable e suas variantes `--now` são atualizados por refresh após a
conclusão do job. Confirmações permanecem aplicadas às ações mutáveis.

## Journal

Logs são carregados com `journalctl --output=json`, com limite de 200 entradas
por refresh, filtro de boot atual/anterior, prioridade e unidade. A busca por
texto ocorre somente sobre as entradas carregadas. A tela de detalhes exibe
timestamp, unidade, prioridade, PID, executável, boot ID e mensagem.

Campos externos passam pela sanitização central antes de renderização. O modo
Follow contínuo não foi forçado nesta recuperação: refresh manual e paginação
limitada são preferidos enquanto não houver uma API de streaming estável no
JobManager.

## Limites conhecidos

- Informações de OpenGL/Vulkan continuam opcionais e dependem de `glxinfo`,
  `eglinfo` ou `vulkaninfo` instalados.
- A alteração de governor exige cpufreq e autorização; mecanismos como TLP ou
  power-profiles-daemon são identificados para evitar oferecer controle
  incompatível.
- O viewer de dispositivos é informativo e não oferece unbind/rebind.
- A ausência de journal, systemd, bateria ou ferramentas opcionais resulta em
  status de indisponibilidade, não em crash.
