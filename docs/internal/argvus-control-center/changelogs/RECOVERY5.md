# Recovery 5 — Pacotes funcionais

O domínio de pacotes usa os parsers e o `ProcessRunner` existentes para
consultas assíncronas e a mesma TUI compartilhada para listas, detalhes,
busca, confirmações, status e resultados.

## Consultas

São suportados search, detalhes, instalados, foreign packages, updates,
órfãos, cache, histórico do `pacman.log`, AUR através de `paru`/`yay` e
mirrorlist configurada. A ausência de pacman, helper AUR, reflector, paccache
ou log não derruba a aplicação.

## Transações

Antes de confirmar install, remove, reinstall ou upgrade, o backend usa o modo
read-only `pacman --print` com `--print-format` para obter os targets e o total
de bytes reportado pelo próprio pacman. A confirmação exibe esse resumo e a
execução somente começa depois dela.

Instalação, reinstalação, remoção, atualização completa, downgrade de arquivo
presente no cache e limpeza paccache passam por `system-settings package`.
Pacotes são validados no frontend e novamente no helper. O helper nunca recebe
um comando arbitrário, nunca remove `db.lck`, nunca usa `--nodeps` ou
`--overwrite`, e instalação/upgrade mantêm a política Arch de full upgrade;
ARGVUS não executa partial upgrades.

As ferramentas oficiais continuam responsáveis pelo solver, verificação de
assinaturas, conflitos e hooks. A UI mostra falhas reais e não promete rollback
quando uma transação já iniciou alterações.

## AUR e mirrors

Builds AUR são executados como usuário normal pelo helper detectado. A UI mostra
aviso de confiança antes da ação e não trata foreign package como sinônimo de
AUR. Reflector gera conteúdo antes de qualquer alteração; conteúdo vazio ou
sem `Server =` é recusado. A aplicação usa backup não sobrescrito, arquivo
temporário, `sync_all` e rename atômico para substituir a mirrorlist.

## Limitações reais

- O solver e prompts interativos complexos do pacman/paru/yay não são
  reimplementados pela aplicação.
- A fila de decisões para provider, replacement, conflito e assinatura ainda
  não está exposta como modal; sem `--noconfirm`, o helper deixa o pacman
  recusar a operação quando essa interação é necessária.
- A preview é baseada nos dados estruturados disponíveis e permanece
  conservadora quando a ferramenta não fornece um plano completo sem iniciar a
  transação.
- Downgrade usa somente arquivos locais do cache; não busca versões antigas.
- A seleção completa de países/protocolos do reflector ainda é a política
  segura padrão do backend, não um editor de argumentos livre.
- A suíte não instala, remove ou atualiza pacotes reais; mutações são testadas
  pela validação da boundary e por backends fake/temp.
