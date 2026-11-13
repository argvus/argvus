---
title: Internacionalização ARGVUS
description: Catálogos, seleção de idioma e suporte Unicode.
---

# Internacionalização ARGVUS

ARGVUS naves 18 línguas de interface: árabe (`ar`), Bengali (`bn-BD`), alemão (`de-DE`),
Inglês (`en-US`), Espanhol (`es-ES`), Francês (`fr-FR`), Hindi (`hi-IN`), indonésio
(`id-ID`), Italiano (`it-IT`), Japonês (`ja-JP`), Coreano (`ko-KR`), Português Brasileiro
(`pt-BR`), português europeu (`pt-PT`), Russo (`ru-RU`), Turco (`tr-TR`), Urdu
(`ur-PK`), Vietnamita (`vi-VN`) e chinês simplificado (`zh-CN`). Inglês é o
catálogo canônico e retrocesso. As traduções foram produzidas com AI assistência e
Correcções técnicas; não foi certificada a revisão independente de falantes nativos.

## Seleção do idioma

Centro de Controle Aberto > Preferências > Idioma e Região > Idioma. A página descobre
catálogos instalados e mostra seus nomes nativos. Salva `ARGVUS_LANGUAGE` em
`$XDG_CONFIG_HOME/locale.conf` (padrão `~/.config/locale.conf`), independentemente de
`LANG`; não é necessário gerar um sistema locale. Português do Brasil (`pt-BR`) e
Português Europeu (`pt-PT`) são escolhas separadas.
A preferência não modifica `/etc/locale.conf` ou o regional `LANG` Preparar.
Comentários e configurações de categoria regional são preservadas. O painel de controle é reiniciado
após salvar; inicie uma nova sessão para aplicar o idioma em todos os componentes da sessão.
Usuários não configurados usam o inglês. Controlos de preferência regionais distintos
`LANG`, formatação regional e aplicações de terceiros. A saudação de login é executada
antes que a preferência do usuário esteja disponível e mantenha sua própria conta de serviço
idioma, inglês por padrão.
As variantes regionais usam o catálogo instalado para o seu idioma quando um exato
catálogo está ausente; por exemplo `es_MX.UTF-8` utilizações `es-ES`. `zh-Hans` seleciona
Chinês simplificado. Chinês tradicional (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) é
não silenciosamente mapeado para chinês simplificado e cai de volta para o Inglês.

## Catálogos e contribuições

A fonte fica em `resources/locales/<locale>/<domínio>.json` no repositório
`argvus`. Os catálogos instalados ficam em `/usr/share/argvus/i18n/` e são
entregues pelo pacote principal `argvus`. `locale.json` informa o identificador
e os nomes nativo e de apresentação. Não há catálogo PO separado nem plataforma
online obrigatória para contribuir.

Use chaves semânticas e preserve placeholders, comandos, opções, caminhos e
atalhos. Traduza mensagens completas, evitando montar frases com fragmentos.
Novas chaves canônicas precisam de tradução nos dez idiomas. Revise a terminologia
no contexto, sobretudo em autenticação, permissões e ações destrutivas.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR` permite selecionar outro diretório para desenvolvimento e testes
do conteúdo empacotado. A validação verifica domínios, chaves, tipos, placeholders,
traduções vazias e formas plurais para contagens inteiras. Ela comprova integridade
estrutural, não a qualidade linguística.

## Unicode e fontes

`argvus-fonts` depende das fontes Noto CJK além das famílias latinas, de símbolos e
emoji. O Fontconfig fornece fallback para chinês, japonês e Hangul preservando a
família principal escolhida pelo usuário. Aplicações e terminais precisam permitir
fallback de fontes.

Os utilitários compartilhados do Ratatui medem células do terminal e preservam
grafemas ao truncar e quebrar textos. Os editores movem e apagam grafemas inteiros.
Uma área de uma célula não comporta um glifo de duas células; nesse caso, o helper
de quebra rígida usa o caractere de substituição. Emoji e caracteres de largura
ambígua ainda podem variar conforme terminal e fonte; a política do terminal deve
concordar com a largura Unicode normal usada pelo Ratatui.

A escolha do idioma não instala método de entrada nem muda o layout do teclado.
Para digitar chinês, japonês e coreano, use um método de entrada compatível com
Wayland.
