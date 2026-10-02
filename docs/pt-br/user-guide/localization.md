---
title: Localização
description: Use o ARGVUS em inglês ou português.
slug: pt/0.4.0/docs/user-guide/localization
---

`argvus-i18n` fornece catálogos compartilhados, `argvus-i18n validate` e o módulo QML `org.argvus.i18n`. Aplicativos nativos, shell e helpers da sessão consomem esses catálogos.

Use **Control Center → Localidade e região** para configurar idioma, localidade regional, locales de sistema gerados, fuso horário, data/hora e teclado. Os catálogos atuais do ARGVUS fornecem inglês e português; isso é separado da lista de locales que o sistema operacional gerou.

Alterar o idioma da interface grava a preferência do ARGVUS e reinicia automaticamente o `argvus-control-panel.service`, para que o Control Panel reflita a alteração imediatamente. Outros aplicativos e superfícies podem exigir reload da sessão ou uma nova sessão. Gerar ou alterar entradas de `/etc/locale.gen` exige autorização e executa `locale-gen`; selecionar um locale que não foi gerado é rejeitado. Layout, variante e mapa do console são configurações separadas e não devem ser confundidos com o idioma da interface.

## Quando as variáveis de ambiente sobrescrevem a alteração

Os aplicativos ARGVUS detectam o idioma primeiro pelo arquivo persistente de idioma do ARGVUS e depois por `LC_ALL`, `LC_MESSAGES` e `LANG`. Variáveis locais podem fazer o Control Center parecer ignorar a troca de idioma, especialmente quando um serviço de usuário ou a sessão gráfica mantém um ambiente antigo.

Inspecione os valores efetivos com:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

Se `~/.config/locale.conf` existir e definir `LANG`, `LC_ALL` ou `LC_MESSAGES`, renomeie-o temporariamente para que ele não reintroduza esses valores. Mantenha um backup em vez de apagá-lo:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

Aplique o idioma novamente em **Control Center → Localidade e região**. O Control Panel será reiniciado automaticamente; se outros componentes ainda exibirem o idioma anterior, saia e entre novamente para que a sessão gráfica e o `systemd --user` recebam o novo ambiente. Repita os comandos de diagnóstico depois disso. Veja [Variáveis de ambiente](../../reference/environment-variables/) para os contratos técnicos e a precedência.
