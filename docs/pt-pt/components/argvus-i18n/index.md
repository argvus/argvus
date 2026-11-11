---
title: Internacionalização ARGVUS
description: Catálogos, seleção de idioma e suporte Unicode.
---

# Internacionalização ARGVUS

O ARGVUS inclui inglês (`en-US`), português brasileiro (`pt-BR`), espanhol (`es-ES`),
francês (`fr-FR`), italiano (`it-IT`), alemão (`de-DE`), russo (`ru-RU`), chinês
simplificado (`zh-CN`), japonês (`ja-JP`) e coreano (`ko-KR`). O inglês é o catálogo
canônico e o fallback. As oito traduções adicionais foram produzidas com auxílio
de tradução automática e correções técnicas; não há certificação de revisão
independente por falantes nativos.

## Seleção do idioma

Abra Control Center > Preferências > Idioma e região > Idioma ou Definições >
Idioma. A página descobre os catálogos instalados e mostra seus nomes nativos.
A escolha grava `LANG` em `$XDG_CONFIG_HOME/locale.conf` (padrão
`~/.config/locale.conf`). Habilite antes o locale UTF-8 correspondente em System
Locales. Exemplos: `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` e `ko_KR.UTF-8`.

A seleção preserva comentários e categorias regionais e não altera
`/etc/locale.conf`. O Control Panel é reiniciado após guardar; saia e entre de novo
para aplicar o idioma à sessão inteira. Utilizadores sem preferência usam inglês.
Variáveis herdadas e ficheiros legados de idioma não substituem essa preferência.
O greeter executa antes de conhecer a preferência do utilizador e usa o idioma da
sua própria conta de serviço, inglês por padrão.

Variantes regionais usam o catálogo instalado do idioma quando não existe um
catálogo exato: `es_MX.UTF-8`, por exemplo, usa `es-ES`. `zh-Hans` usa chinês
simplificado. Chinês tradicional (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) não é
convertido silenciosamente para simplificado e usa o fallback inglês.

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
família principal escolhida pelo utilizador. Aplicações e terminais precisam permitir
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
