---
title: Localização
description: Use o ARGVUS em inglês ou português.
---

`argvus-i18n` fornece os catálogos compartilhados, o `argvus i18n validate` comando e o QML módulo `org.argvus.i18n`. ARGVUS navios 18 idiomas de interface: árabe, bengali, alemão, inglês, espanhol, francês, hindi, indonésio, italiano, japonês, coreano, português brasileiro, português europeu, russo, turco, urdu, vietnamita e chinês simplificado.

Nativo ARGVUS aplicações e componentes de shell consomem os catálogos compartilhados. Uma string traduzida pode, portanto, ser usada por mais de um repositório. O Árabe e o Urdu usam a disposição da direita para a esquerda no suporte GTK, Quickshell e interfaces terminais.

Use **Control Center → Preferências → Idioma & Região → Idioma** para selecionar o ARGVUS linguagem de interface. Isto escreve `ARGVUS_LANGUAGE` para `$XDG_CONFIG_HOME/locale.conf` (normalmente `~/.config/locale.conf`). Por exemplo, `ARGVUS_LANGUAGE=pt-PT` selecciona o Português Europeu. Português Brasileiro (`pt-BR`) é uma opção separada. O catálogo de idiomas é instalado com ARGVUS, por isso não depende de `locale-gen`.

O Painel de Controle reinicia automaticamente após as alterações da linguagem da interface. Inicie uma nova sessão para aplicá-la aos componentes da sessão. A configuração separada ** Regional Locale** escreve `LANG` e controla formatos regionais e aplicações de terceiros. Requer um locale do sistema gerado; habilitando `/etc/locale.gen` executa `locale-gen` com autorização do administrador. O layout do teclado, a variante e o mapa do console são configurações separadas.

# # Quando as variáveis de ambiente sobrepõem a mudança

ARGVUS aplicações lidas `ARGVUS_LANGUAGE` do utilizador `locale.conf`Se estiver ausente, resolvem o problema regional. `LANG` valor para um catálogo instalado. Variáveis locais herdadas não sobrepõem uma explícita `ARGVUS_LANGUAGE` preferência. As aplicações de terceiros continuam a utilizar o sistema regional.

Inspeccionar os valores efectivos com:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Verifica isso. `~/.config/locale.conf` contém um válido `ARGVUS_LANGUAGE=<catalog>` valor. Para alterar a formatação regional ou o idioma de aplicativos de terceiros, use ** Regional Locale** e selecione um locale gerado. O Painel de Controle é reiniciado após qualquer alteração de preferência; saia e entre se outros componentes de sessão ainda mostrarem o idioma de interface antigo. Ver [variáveis do ambiente](/docs/reference/environment-variables/) para os contratos técnicos e precedência.
