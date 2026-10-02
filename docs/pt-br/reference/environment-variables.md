---
title: Variáveis de ambiente
description: Contratos de ambiente usados pela sessão.
slug: pt/0.4.0/docs/reference/environment-variables
---

A sessão configura integração XDG e Wayland e pode exportar variáveis de compatibilidade para virtualização. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` e `ARGVUS_BOOTSTRAP` são usados para resolver raízes de configuração.

Trate essas variáveis como contratos de runtime e verifique o helper instalado antes de criar overrides persistentes.

O estado derivado fica na raiz de estado, e não na raiz de configuração.
`XDG_STATE_HOME` (normalmente `~/.local/state`) recebe
`argvus/config-projection.json`, o manifesto de projeção que o `argvus-config`
reconstrói e o `argvus-sessionctl reload` aplica, além de `argvus/session.log`.
Nenhum dos dois é uma fonte de verdade.

## Ambiente de idioma

A implementação compartilhada do `argvus-i18n` seleciona o locale nesta ordem:

1. `$XDG_CONFIG_HOME/argvus/data/internal/language` (normalmente `~/.config/argvus/data/internal/language`);
2. `LC_ALL`;
3. `LC_MESSAGES`;
4. `LANG`;
5. fallback `en-US`.

Valores como `pt_BR.UTF-8` são normalizados para `pt-BR`. `LANG`, `LC_ALL` e `LC_MESSAGES` são importados para os ambientes do systemd de usuário e do D-Bus pelo controlador da sessão. Um ambiente antigo do user manager pode afetar aplicativos iniciados depois que o Control Center troca o idioma.

Para diagnosticar, compare a localidade do processo com o ambiente do user manager e a configuração de localidade do sistema:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

Se `~/.config/locale.conf` estiver presente e contiver definições de locale conflitantes, preserve um backup e renomeie o arquivo; depois aplique o idioma novamente no Control Center e inicie uma nova sessão gráfica:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

`ARGVUS_CONFIG_HOME` e `ARGVUS_SYSTEM_CONFIG` controlam raízes de configuração e são separados do arquivo de seleção de idioma. `ARGVUS_I18N_DIR` pode selecionar uma raiz alternativa de catálogos para desenvolvimento ou testes e normalmente não deve ser definido como personalização do usuário.
