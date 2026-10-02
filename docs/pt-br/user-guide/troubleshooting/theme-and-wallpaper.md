---
title: Problemas de tema e wallpaper
description: Diagnostique propagação de aparência.
slug: pt/0.4.0/docs/user-guide/troubleshooting/theme-and-wallpaper
---

Confira a seção canônica `appearance` em `$XDG_CONFIG_HOME/argvus/config.json` e os arquivos gerados em `$XDG_CONFIG_HOME/argvus/data/generated`. Os dot-files antigos são apenas entradas de migração.

O `argvus-config` é o único escritor abaixo de `data/generated/`. Para recuperar um
tema ou wallpaper quebrado, refaça a geração inteira a partir do documento
canônico e aplique o resultado, em vez de editar consumidores gerados:

```sh
argvus-config validate
argvus-config apply
argvus-sessionctl reload
```

`theme-switch.sh` e `accent-switch.sh` são orquestradores: validam, travam, mostram
o splash, confirmam a mudança pelo `argvus-config` e só então reconciliam os
consumidores externos que não conseguem ler o `config.json`. Eles nunca escrevem em
`data/generated/`, e `theme-switch.sh` não grava mais os arquivos de GTK, storage
ou `mode.css`. Use `accent-switch.sh --apply` para reexecutar apenas a reconciliação
de acento.

Se apenas o greeter pré-login estiver branco ou usando o tema de fallback, compare a fonte privada com a projeção pública:

```sh
uid=$(id -u)
argvus-config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```

Repare a projeção sem alterar a configuração do greetd:

```sh
sudo argvus-greeter-setup --sync-themes
```

Esse arquivo é publicado para o pré-login pelo orquestrador de aparência e fica fora
de `data/generated/`, portanto um simples `argvus-config apply` não o atualiza.

Se o greeter informar `Invalid color name: 'null'` em um cache do Kitty, atualize
o pacote `argvus-terminal` e gere o cache novamente abrindo o greeter. Um acento
canônico ausente representa o valor padrão, não uma cor Kitty; o launcher atual
rejeita `null` do JSON e valores legados inválidos antes de gravar
`active_tab_foreground`.

Se o terminal padrão parar de abrir depois de várias trocas de tema, inspecione o cache derivado do Kitty em vez de editá-lo manualmente:

```sh
argvus-config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus-terminal --apply
argvus-terminal --print-config
```

O comando acima lê a preferência canônica. `.active-theme` é apenas uma
projeção de compatibilidade. O cache é regenerado pelo `argvus-terminal` e
sincronizado com a aplicação do tema. Um
`/usr/lib/argvus/theme-splash/splash` ausente ou antigo indica uma atualização
incompleta dos pacotes; reconstrua e instale os pacotes coordenados antes de
avaliar o resultado na sessão real.

Se as bordas das janelas mantiverem a cor do tema anterior, não edite o Lua do
Hyprland manualmente. As mudanças de tema e acento são projetadas a partir do
`config.json`; depois `argvus-sessionctl reload` recarrega o Hyprland e reaplica
as cores das bordas ativa, inativa e de grupos pelo lifecycle da sessão:

```text
config.json → plano de projeção → reload do Hyprland → reaplicação via hyprctl das bordas ativas/inativas/grupo
```

As *preferências* de borda são alteradas por `argvus-config borders` (ou seu
delegate `borders-switch.sh`) no Control Center e no Control Panel; o reload
reaplica as cores já confirmadas pelo ciclo de vida da sessão e não as
rederiva.

Inspecione o plano e os valores efetivos do compositor:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus-sessionctl reload
```

Se `hyprctl` não conseguir acessar o socket atual do Hyprland, a projeção das
bordas será reportada como falha e os componentes dependentes não serão tratados
como recarregados com sucesso. Consulte o journal da sessão do usuário antes de
tentar novamente.
