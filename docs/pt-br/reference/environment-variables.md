---
title: Variáveis de ambiente
description: Contratos de ambiente usados pela sessão.
---

A sessão configura integração XDG e Wayland e pode exportar variáveis de compatibilidade para virtualização. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` e `ARGVUS_BOOTSTRAP` são usados para resolver raízes de configuração.

Trate essas variáveis como contratos de runtime e verifique o helper instalado antes de criar overrides persistentes.

O estado derivado fica na raiz de estado, e não na raiz de configuração.
`XDG_STATE_HOME` (normalmente `~/.local/state`) recebe
`argvus/config-projection.json`, o manifesto de projeção que o `argvus-config`
reconstrói e o `argvus sessionctl reload` aplica, além de `argvus/session.log`.
Nenhum dos dois é uma fonte de verdade.

## Ambiente de idioma

A única preferência geral de idioma é `LANG` em `$XDG_CONFIG_HOME/locale.conf` (padrão `~/.config/locale.conf`). Sem valor válido, o ARGVUS usa inglês (`en-US`), mesmo quando `/etc/locale.conf`, PAM ou o gerenciador de login fornecem português. Valores como `pt_BR.UTF-8` são normalizados para `pt-BR` nos catálogos.

A sessão aplica essa preferência antes de iniciar os componentes e importar os ambientes do systemd de usuário e da ativação D-Bus. Substitui `LANG`, `LC_MESSAGES` e `LANGUAGE` herdados e limpa `LC_ALL`; configurações regionais `LC_*` do mesmo arquivo são preservadas. O inglês usa `en_US.UTF-8` quando gerado, ou `C.UTF-8` caso contrário. Nenhum arquivo de preferência é criado automaticamente. Saia e entre novamente após trocar o idioma.

`session.language`, `argvus/data/language` e `argvus/data/internal/language` são dados legados ignorados. Não são importados nem regenerados; os arquivos existentes são preservados. `argvus config set` e `patch` recusam novas gravações de `session.language` e orientam o uso de `locale.conf`.

Para diagnóstico, compare `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` e `systemctl --user show-environment`. `localectl status` descreve o sistema, não o idioma do ARGVUS do usuário. `ARGVUS_I18N_DIR` seleciona o diretório de catálogos para desenvolvimento, não o idioma.
