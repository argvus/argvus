# Argvus

Argvus is the desktop configuration repository for the Argvus project: a complete desktop, ready to use, focused on Hyprland.

It contains the packaged defaults for Hyprland, Waybar, Quickshell, Rofi,
Kitty, Dunst, Wlogout, terminal tools and themes. The package installs those
immutable defaults under `/usr/share/argvus`.

Argvus does not need to copy dotfiles into `$HOME` before the desktop can
start. Runtime entrypoints use this priority:

```text
$XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-setup` is optional. Use it only when you want to copy packaged defaults
into your user config for customization, for example:

```sh
argvus-setup --copy hypr
argvus-setup --copy waybar
argvus-setup --copy-all
```

Once copied, files in `$XDG_CONFIG_HOME` are user-owned overrides and package
upgrades do not overwrite them.

Hyprland reads the active theme, accent and spacing preferences from
`$XDG_CONFIG_HOME/argvus`, matching the scripts that update those files.
Shared wallpapers and bundled fonts live in the separate `argvus-appearance`
package and are referenced from system paths.
The packaged Hyprland bindings launch Kitty with the resolved Argvus
`kitty.conf`, so clean users still receive the packaged terminal theme before
they create any user overrides. Hyprlock generates its lockscreen wallpaper
cache under `$XDG_CACHE_HOME/argvus/hypr`.
Session startup infrastructure lives in the separate `argvus-session` package:
`argvus-session`, `argvus-start`, `argvus-tty` and the Wayland session entry.

Arch Linux packaging is owned by this repository through
`packaging/arch/PKGBUILD`. Pushing a `v*` tag builds the signed package and
publishes both the `.pkg.tar.zst` and `.sig` files to the `packages` repository
under `public/arch/x86_64`.

## Documentation

- User documentation: https://argvus.github.io/docs/environment/
- Official apps: https://argvus.github.io/docs/official-apps/
- Development workflow: [DEVELOPMENT.md](./DEVELOPMENT.md)
- Contribution guide: [CONTRIBUTING.md](./CONTRIBUTING.md)

## Related repositories

- [`argvus-storage`](https://github.com/argvus/argvus-storage)
- [`argvus-appearance`](https://github.com/argvus/argvus-appearance)
- [`argvus-session`](https://github.com/argvus/argvus-session)
- [`packages`](https://github.com/argvus/packages)
- [`site-src`](https://github.com/argvus/site-src)
