# ARGVUS Accounts (notas de desenvolvimento)

O código do gerenciador de contas fica em `crates/argvus-accounts` (CLI) e
`crates/argvus-accounts-core` (biblioteca). O pacote `argvus` compila o binário
`argvus-accounts` e o instala como `/usr/lib/argvus/commands/accounts`, o caminho que
o despachante (`argvus accounts`), o Control Center (`administration.rs`) e o
`UserCard.qml` do painel de controle usam. Não mude esse caminho sem atualizar esses
três pontos e a regra polkit.

## Fronteira de privilégio

Não há serviço, setuid, capability nem D-Bus próprio. O modelo é:

1. A CLI roda como o usuário. Operações de leitura e a edição do próprio avatar
   (`$HOME/.face`) não precisam de root.
2. Qualquer mutação que altere outra conta ou dados do sistema passa por
   `elevate::ensure(Requirement::Always)` (ou `OtherUsersOnly`). Essa função
   reexecuta o próprio binário com `pkexec`, com ambiente limpo e apenas
   `DBUS_SESSION_BUS_ADDRESS` e `XDG_RUNTIME_DIR`.
3. `polkit/50-argvus-accounts.rules` (instalado em
   `/usr/share/polkit-1/rules.d/`) autoriza `org.freedesktop.policykit.exec` somente
   quando `program` é `/usr/lib/argvus/commands/accounts`. Quem está no grupo `sudo`
   autentica com a própria senha; os demais, com a do root.
4. O processo root é o mesmo binário. Toda validação de entrada é feita nele, em
   `admin.rs`, `validation.rs`, `metadata.rs` e `password.rs`. O subcomando `manage`
   lê JSON da entrada padrão com limite de 64 KiB e recusa execução que não seja root.

A elevação não concede nada além do que o polkit autorizou: `ARGVUS_ACCOUNTS_ELEVATED`
só evita uma segunda tentativa de elevação.

## Regras que não podem regredir

- Binários de sistema são chamados por caminho absoluto (`/usr/bin/...`). Nunca use
  o nome nu: o PATH do chamador não pode escolher o que roda como root. Há testes em
  `groups.rs` e `metadata.rs`. `password.rs` (`chpasswd`) ainda não tem teste dedicado.
- Arquivos de avatar são abertos com `O_NONBLOCK` e validados pelo descritor (`fstat`),
  não pelo caminho. Um FIFO ou dispositivo é recusado sem bloquear. A leitura é limitada
  a `MAX_SOURCE_FILE_BYTES + 1`. Coberto por `avatar_rejects_bad_inputs` em
  `tests/integration.rs`.
- `Edit` chama `protect_system_account`: contas com UID abaixo de 1000 e `nobody` não
  são editadas. A guarda de auto-remoção de administrador usa o grupo `sudo`.
- Na revisão, não foi encontrado nenhum log que grave senhas ou hashes. Mantenha assim ao
  adicionar diagnósticos (`--verbose`).

## Senhas no fluxo de elevação

`passwd USER` lê três linhas da entrada padrão: senha atual, nova e confirmação. As
checagens baratas (confirmação e política) e a prova da senha própria rodam antes da
elevação. Depois, `elevate::ensure_with_input` faz `exec` do `pkexec` com um pipe anônimo
como entrada, preenchido antes com os mesmos bytes. Os segredos nunca vão para argv nem
para o ambiente. O processo elevado relê a entrada, e `change_password` impõe a prova.

## Avatar: diretório pessoal aberto uma vez

`deploy_png` e `remove_avatar` abrem o diretório pessoal com `O_DIRECTORY|O_NOFOLLOW`
e fazem todos os passos relativos a esse descritor (`openat`, `renameat`, `unlinkat`).
Um caminho trocado por symlink depois da checagem de dono não redireciona a escrita.
Um symlink no próprio diretório pessoal é recusado. `avatar_never_writes_through_a_symlinked_home`
cobre esse caso; a corrida em si não é reproduzida de forma determinística.

## Riscos residuais conhecidos

- A regra polkit autoriza o binário inteiro, não cada operação. `org.freedesktop.policykit.exec`
  não recebe os argumentos como identidade de ação. Uma autorização de administrador vale
  para qualquer subcomando, inclusive `manage`, que aceita qualquer operação em JSON. Corrigir
  isso exige ações polkit por operação, o que implica um helper root separado com
  `CheckAuthorization`, ou uma regra por `command_line`. A segunda opção não foi validada
  contra um polkitd real.
- A regra polkit usa a ação genérica. O README descreve ações próprias
  `com.argvus.accounts.*` como futuras; não existem `.policy` instalados.

## Testes

`cargo test -p argvus-accounts -p argvus-accounts-core --locked` cobre a biblioteca, a
CLI e a integração de autorização. Os testes de administração registram as chamadas de sistema
em vez de executá-las.
