---
title: Packaging
description: Como o ARGVUS é empacotado e publicado.
slug: pt/0.1.0/docs/packaging
---

O projeto ARGVUS é organizado em vários repositórios na organização `argvus` do GitHub:

* [argvus](https://github.com/argvus/argvus): configuração do desktop (config, bin/argvus-setup, share).
* [argvus-session](https://github.com/argvus/argvus-session): launchers de sessão e a entrada de sessão Wayland.
* [argvus-appearance](https://github.com/argvus/argvus-appearance): wallpapers e fontes empacotadas compartilhados.
* [argvus-storage](https://github.com/argvus/argvus-storage): o programa Rust que alimenta o módulo Waybar de armazenamento removível.
* [argvus-calendar](https://github.com/argvus/argvus-calendar): o programa Rust que alimenta o popup de calendário da Waybar.
* [argvus-greeter](https://github.com/argvus/argvus-greeter): integração com o greeter greetd.
* [argvus-pkgbuild](https://github.com/argvus/argvus-pkgbuild): os PKGBUILDs do desktop e dos pacotes de componentes.
* [packages](https://github.com/argvus/packages): o repositório de pacotes binários servido em `https://argvus.github.io/packages/`.
* [site-src](https://github.com/argvus/site-src): a fonte deste site (Astro).
* [argvus.github.io](https://github.com/argvus/argvus.github.io): o site publicado.

Os manifests Arch ficam em `arch/` no [argvus-pkgbuild](https://github.com/argvus/argvus-pkgbuild): `arch/argvus/PKGBUILD`, `arch/argvus-session/PKGBUILD`, `arch/argvus-appearance/PKGBUILD`, `arch/argvus-storage/PKGBUILD`, `arch/argvus-calendar/PKGBUILD` e `arch/argvus-greeter/PKGBUILD`. As dependências vendored (`pwvucontrol` e `snappy-switcher`) ficam em [vendors/](https://github.com/argvus/argvus-pkgbuild/tree/main/vendors).

## Empacotamento

O ARGVUS é empacotado como um spin de Hyprland para Arch Linux, dividido em pacotes de componentes focados.

A instalação dos pacotes é somente-sistema: instala defaults somente leitura em `/usr/share/`, comandos em `/usr/bin/` e a entrada de sessão em `/usr/share/wayland-sessions/`. Nada é escrito em `$HOME` durante a instalação.

## Layout instalado

O pacote `argvus` instala:

```text
/usr/bin/argvus-setup
/usr/share/argvus/
```

O `argvus-session` instala os entrypoints de sessão:

```text
/usr/bin/argvus-session
/usr/bin/argvus-start
/usr/bin/argvus-tty
/usr/share/wayland-sessions/argvus.desktop
```

O `argvus-appearance` instala os ativos compartilhados:

```text
/usr/share/backgrounds/argvus/
/usr/share/fonts/
```

O `argvus-storage` é empacotado separadamente. Ele instala `/usr/bin/argvus-storage`, a configuração padrão em `/etc/argvus-storage/` e a licença em `/usr/share/licenses/argvus-storage/`. Veja a página [Armazenamento Removível](../storage/) para uso e configuração.

O `argvus-calendar` é empacotado separadamente. Ele instala `/usr/bin/argvus-calendar`, os defaults em `/etc/argvus-calendar/` e a unit systemd de usuário em `/usr/lib/systemd/user/argvus-calendar.service`. Veja a página [Calendário](../calendar/) para uso e configuração.

O pacote `argvus` depende de `argvus-appearance`, `argvus-calendar`, `argvus-greeter`, `argvus-session` e `argvus-storage`, então instalar o ARGVUS resolve-os do mesmo repositório.

## Configuração do usuário

O ARGVUS não copia dotfiles para `$HOME` antes de o desktop poder iniciar. Os entrypoints de runtime leem overrides completos do usuário em `$XDG_CONFIG_HOME/<app>`, configuração de runtime gerada em `$XDG_STATE_HOME/argvus/config/<app>` e defaults empacotados em `/usr/share/argvus/<app>`.

`$XDG_STATE_HOME` usa o fallback `~/.local/state` quando não está definido. A configuração de runtime gerada é reconstruída a partir dos defaults atuais do pacote mais as preferências do ARGVUS, então upgrades de pacote podem atualizar `/usr/share/argvus` sem sobrescrever arquivos do usuário.

O `argvus-setup` é um helper opcional. Ele copia `/usr/share/argvus/<app>` para `$XDG_CONFIG_HOME/<app>` sob demanda (`--copy <app>` ou `--copy-all`), substituindo um destino existente apenas com `--force`, que faz backup do diretório atual como `$XDG_CONFIG_HOME/<app>.bak-<timestamp>`. Depois de copiados, os arquivos são overrides do usuário que upgrades de pacote nunca substituem.

O `argvus-tty` fornece suporte a login por TTY sem display manager. Ele registra uma sessão logind, inicia o D-Bus, define variáveis de ambiente e inicia o Hyprland quando o usuário executa `argvus-tty` manualmente após logar em um TTY. O pacote não instala um script `profile.d` de auto-start; um profile opt-in é empacotado em `/usr/share/argvus/argvus/profile` e o auto-start por TTY é uma escolha explícita do usuário.

Algumas dependências de runtime que antes exigiam o AUR, incluindo `pwvucontrol` e `snappy-switcher`, são vendored e publicadas no mesmo repositório que o ARGVUS:

```sh
sudo pacman -Syu argvus
```

## Versionamento

Os pacotes de componentes fixam seus repositórios por tag:

```sh
ENV_VER=0.1.0            # arch/argvus/PKGBUILD
pkgver=0.1.4              # arch/argvus-storage/PKGBUILD
pkgver=0.2.2              # arch/argvus-calendar/PKGBUILD
```

A versão do pacote é atualizada com `make version <X.Y.Z>` no repositório argvus-pkgbuild. Mantenha `ENV_VER` alinhado com a tag do `argvus` e atualize os PKGBUILDs dos componentes quando seus repositórios publicarem novas tags.

## Publicação

O workflow **Build and publish Arch packages** (`.github/workflows/build-and-publish.yml`) no repositório argvus-pkgbuild constrói todos os PKGBUILDs descobertos, incluindo o `argvus` e seus pacotes de componentes, assina os pacotes e bancos pacman com a chave GPG do ARGVUS, monta o banco com `repo-add` e publica os pacotes no branch `main` de [packages](https://github.com/argvus/packages) sob `public/arch/`.

O layout do repositório packages:

```text
packages
  public/arch/argvus.conf
  public/arch/argvus.gpg
  public/arch/x86_64/argvus.db
  public/arch/x86_64/argvus.db.sig
  public/arch/x86_64/*.pkg.tar.zst
  public/arch/x86_64/*.pkg.tar.zst.sig
  public/debian/
  public/rpm/
```

O workflow de deploy do site (no repositório [site-src](https://github.com/argvus/site-src)) constrói o site com Astro e espelha o `dist/` em [argvus.github.io](https://github.com/argvus/argvus.github.io). Ele não armazena pacotes binários.

Para publicar os pacotes no repositório `packages` são necessários os secrets `PACKAGES_REPO_TOKEN` (um token de acesso pessoal de escopo restrito com `contents: write` no repositório packages) e `GPG_PRIVATE_KEY` (a chave privada ASCII-armored usada apenas dentro do GitHub Actions). `GPG_PASSPHRASE` é opcional quando a chave privada tiver senha.
