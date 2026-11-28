//! Entries of the Control Center search: one per destination, with the
//! search category, the title key, extra keywords (English and Portuguese)
//! and the route that `open_search_route` understands.

use super::*;

/// Registers every searchable destination compiled into this build. Entries
/// behind a disabled feature are left out, so search never offers a page the
/// build cannot open.
pub fn build_search_registry(lang: Lang) -> SearchRegistry {
  let mut search_registry = SearchRegistry::default();
  let english = Lang::for_locale("en-US");
  let entries: [(&str, &str, &str, &str, &str); _] = [
    #[cfg(feature = "appearance")]
    (
      "appearance.desktop_style",
      "appearance",
      "control_center.desktop_style",
      "desktop style",
      "appearance/desktop_style",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.application_style",
      "appearance",
      "control_center.application_style",
      "application style",
      "appearance/application_style",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.locale",
      "preferences",
      "control_center.language_region",
      "locale",
      "settings/locale",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.time-zone",
      "preferences",
      "control_center.time_zone",
      "time zone",
      "settings/time-zone",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.regional-locale",
      "preferences",
      "control_center.regional_locale",
      "regional locale",
      "settings/regional-locale",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.date-time",
      "preferences",
      "control_center.date_time",
      "date time",
      "settings/date-time",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.system-locales",
      "preferences",
      "control_center.system_locales",
      "system locales",
      "settings/system-locales",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.keyboard",
      "preferences",
      "control_center.keyboard",
      "keyboard",
      "settings/keyboard",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.keyboard-layout",
      "preferences",
      "control_center.keyboard_layout",
      "keyboard layout",
      "settings/keyboard-layout",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.keyboard-variant",
      "preferences",
      "control_center.variant",
      "keyboard variant",
      "settings/keyboard-variant",
    ),
    #[cfg(feature = "locale")]
    (
      "settings.console-keymap",
      "preferences",
      "control_center.console_keymap",
      "console keymap",
      "settings/console-keymap",
    ),
    #[cfg(feature = "hardware")]
    (
      "settings.mouse-touchpad",
      "card_desktop_input",
      "control_center.mouse_touchpad",
      "mouse touchpad",
      "settings/mouse-touchpad",
    ),
    #[cfg(feature = "system")]
    (
      "settings.users",
      "card_system",
      "control_center.users",
      "users",
      "settings/users",
    ),
    #[cfg(feature = "system")]
    (
      "settings.users-list",
      "card_system",
      "control_center.list",
      "users list",
      "settings/users-list",
    ),
    #[cfg(feature = "system")]
    (
      "settings.groups",
      "card_system",
      "control_center.groups",
      "groups",
      "settings/groups",
    ),
    #[cfg(feature = "system")]
    (
      "settings.groups-list",
      "card_system",
      "control_center.list",
      "groups list",
      "settings/groups-list",
    ),
    (
      "hyprland.home",
      "card_desktop_input",
      "control_center.hyprland",
      "windows animations blur borders info",
      "hyprland/home",
    ),
    (
      "hyprland.info",
      "hyprland",
      "control_center.info",
      "hyprland version",
      "hyprland/info",
    ),
    (
      "control-center.settings",
      "",
      "control_center.settings",
      "language style",
      "control-center/settings",
    ),
    (
      "control-center.style",
      "settings",
      "control_center.style",
      "style appearance control center",
      "control-center/style",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.profiles",
      "displays",
      "control_center.profiles",
      "monitor display profile",
      "displays/profiles",
    ),
    #[cfg(feature = "about")]
    (
      "about.about",
      "card_tools",
      "control_center.about",
      "about",
      "about/about",
    ),
    #[cfg(feature = "about")]
    (
      "about.donate",
      "card_tools",
      "control_center.donate",
      "donate",
      "about/donate",
    ),
    #[cfg(feature = "about")]
    (
      "about.credits",
      "card_tools",
      "control_center.credits",
      "credits",
      "about/credits",
    ),
    #[cfg(feature = "about")]
    (
      "about.copyright",
      "card_tools",
      "control_center.copyright",
      "copyright",
      "about/copyright",
    ),
    (
      "launcher",
      "card_preferences",
      "control_center.launcher_search",
      "launcher search pesquisa busca arquivos indexacao",
      "launcher/settings",
    ),
    #[cfg(feature = "appearance")]
    (
      "hyprland.wobble",
      "hyprland",
      "control_center.wobble",
      "wobble windows animations stiffness damping mesh deformacao janelas animacoes malha",
      "hyprland/animations-windows",
    ),
    (
      "welcome",
      "card_tools",
      "control_center.open_welcome",
      "welcome onboarding first run setup boas-vindas configuração inicial",
      "tools/welcome",
    ),
    #[cfg(feature = "fonts")]
    (
      "settings.fonts",
      "appearance",
      "control_center.fonts",
      "font fonts typeface typography",
      "settings/fonts",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.default_apps",
      "preferences",
      "control_center.default_apps",
      "default application apps browser terminal file manager",
      "settings/default-apps",
    ),
    #[cfg(feature = "language")]
    (
      "settings.language",
      "preferences",
      "control_center.language",
      "language interface display language idioma idioma da interface",
      "settings/language",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.window_rules",
      "hyprland_windows",
      "control_center.window_rules",
      "window rules workspace class app placement regras janelas area de trabalho",
      "hyprland/windows-rules",
    ),
    (
      "hyprland.windows",
      "hyprland",
      "control_center.hyprland_windows",
      "windows window janelas janela rules spaces mode regras espaços modo",
      "hyprland/windows",
    ),
    (
      "hyprland.window_mode",
      "hyprland_windows",
      "control_center.window_mode",
      "window mode tile tiling floating modo janelas mosaico flutuante",
      "hyprland/windows-mode",
    ),
    #[cfg(feature = "appearance")]
    (
      "hyprland.window_spaces",
      "hyprland_windows",
      "control_center.window_spaces",
      "window spaces gaps inner outer spacing espaços janelas espaçamento interno externo",
      "hyprland/windows-spaces",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.projects",
      "card_tools",
      "control_center.projects",
      "projects project folders path directory root shortcut seletor projetos pastas caminho diretorio raiz atalho",
      "settings/projects",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.snippets",
      "card_tools",
      "control_center.snippets",
      "snippets snippet text type typing shortcut picker trechos texto digitar atalho seletor",
      "settings/snippets",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.ssh_keys",
      "card_tools",
      "control_center.ssh_keys",
      "ssh keys key pair generate load unload agent passphrase chaves ssh gerar carregar descarregar agente senha par",
      "settings/ssh-keys",
    ),
    #[cfg(feature = "apps")]
    (
      "settings.gpg_keys",
      "card_tools",
      "control_center.gpg_keys",
      "gpg keys key pair generate export import pinentry passphrase chaves gpg gerar exportar importar senha pinentry",
      "settings/gpg-keys",
    ),
    #[cfg(feature = "system")]
    (
      "settings.firewall",
      "card_system",
      "control_center.firewall",
      "firewall ports rules block allow network security portas regras bloquear permitir rede seguranca",
      "settings/firewall",
    ),
    #[cfg(feature = "system")]
    (
      "settings.hostname",
      "card_system",
      "control_center.hostname",
      "hostname name machine computer host nome maquina computador",
      "settings/hostname",
    ),
    #[cfg(feature = "system")]
    (
      "settings.users_groups",
      "card_system",
      "control_center.users_groups",
      "users groups accounts usuarios grupos contas",
      "settings/users-groups",
    ),
    #[cfg(feature = "system")]
    (
      "settings.notifications",
      "card_system",
      "control_center.notifications",
      "notifications do not disturb dnd silence notificacoes nao perturbe silenciar",
      "settings/notifications",
    ),
    #[cfg(feature = "about")]
    (
      "about",
      "card_tools",
      "control_center.about_argvus",
      "about information version license system hardware boot firmware uefi bios secure virtualization kernel battery cpu gpu driver memory motherboard sobre informacoes versao licenca sistema bateria memoria placa mae",
      "about/system",
    ),
    #[cfg(feature = "network")]
    (
      "network",
      "network",
      "control_center.network",
      "network internet connectivity",
      "network/home",
    ),
    #[cfg(feature = "audio")]
    (
      "audio",
      "audio",
      "control_center.audio",
      "audio sound volume",
      "audio/home",
    ),
    #[cfg(feature = "bluetooth")]
    (
      "bluetooth",
      "bluetooth",
      "control_center.bluetooth",
      "bluetooth wireless pairing",
      "bluetooth/home",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware",
      "hardware",
      "control_center.hardware",
      "hardware system devices",
      "hardware/home",
    ),
    (
      "settings.keybindings",
      "card_desktop_input",
      "control_center.keyboard_shortcuts",
      "keyboard shortcuts keybindings hotkeys bindings atalhos teclado",
      "settings/keybindings",
    ),
    (
      "settings.shortcuts.create",
      "card_desktop_input",
      "control_center.shortcuts_create",
      "create shortcut command dispatcher criar atalho comando",
      "settings/shortcuts-create",
    ),
    (
      "settings.shortcuts.list",
      "card_desktop_input",
      "control_center.shortcuts_all",
      "all shortcuts custom todos atalhos personalizados",
      "settings/shortcuts-list",
    ),
    #[cfg(feature = "services")]
    (
      "services",
      "services",
      "control_center.services",
      "services systemd daemon",
      "services/home",
    ),
    #[cfg(feature = "dev-services")]
    (
      "dev-services",
      "dev-services",
      "control_center.developer_services",
      "dev services systemd ports containers podman docker desenvolvimento portas",
      "dev-services/home",
    ),
    #[cfg(feature = "boot")]
    (
      "boot",
      "boot",
      "control_center.boot",
      "boot startup",
      "boot/home",
    ),
    #[cfg(feature = "packages")]
    (
      "packages",
      "packages",
      "control_center.section_packages",
      "package pacman software",
      "packages/home",
    ),
    #[cfg(feature = "snapshot")]
    (
      "snapshot",
      "card_system",
      "control_center.snapshot.title",
      "snapper btrfs snapshot backup",
      "snapshot/home",
    ),
    #[cfg(feature = "storage")]
    (
      "storage",
      "storage",
      "control_center.storage",
      "storage disk filesystem",
      "storage/home",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics",
      "diagnostics",
      "control_center.system_doctor",
      "system doctor diagnostics diagnóstico health saúde",
      "diagnostics/home",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.summary",
      "hardware",
      "control_center.summary",
      "summary hardware overview",
      "hardware/summary",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.cpu",
      "hardware",
      "control_center.hardware_cpu",
      "cpu processor",
      "hardware/cpu",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.gpu",
      "hardware",
      "control_center.hardware_gpu",
      "gpu graphics video driver",
      "hardware/gpu",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.memory",
      "hardware",
      "control_center.memory",
      "memory ram",
      "hardware/memory",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.power",
      "hardware",
      "control_center.power",
      "power energy battery",
      "hardware/power",
    ),
    #[cfg(feature = "hardware")]
    (
      "hardware.devices",
      "hardware",
      "control_center.devices",
      "devices pci usb drm input",
      "hardware/devices",
    ),
    #[cfg(feature = "services")]
    (
      "services.system",
      "services",
      "control_center.system",
      "systemd services service daemon",
      "services/system",
    ),
    #[cfg(feature = "services")]
    (
      "services.user",
      "services",
      "control_center.user",
      "systemd user service",
      "services/user",
    ),
    #[cfg(feature = "services")]
    (
      "services.failed",
      "services",
      "control_center.failed",
      "failed failures",
      "services/failed",
    ),
    #[cfg(feature = "services")]
    (
      "services.logs",
      "services",
      "control_center.logs",
      "logs journal journalctl",
      "services/logs",
    ),
    #[cfg(feature = "network")]
    (
      "network.status",
      "network",
      "control_center.status",
      "network rede internet",
      "network/status",
    ),
    #[cfg(feature = "network")]
    (
      "network.interfaces",
      "network",
      "control_center.interfaces",
      "interfaces interface rede",
      "network/interfaces",
    ),
    #[cfg(feature = "network")]
    (
      "network.wifi",
      "network",
      "control_center.wi_fi",
      "wifi wireless wlan sem fio",
      "network/wifi",
    ),
    #[cfg(feature = "network")]
    (
      "network.ethernet",
      "network",
      "control_center.ethernet",
      "ethernet lan",
      "network/ethernet",
    ),
    #[cfg(feature = "network")]
    (
      "network.vpn",
      "network",
      "control_center.vpn",
      "vpn wireguard openvpn tunnel",
      "network/vpn",
    ),
    #[cfg(feature = "network")]
    (
      "network.dns",
      "network",
      "control_center.dns",
      "dns resolved nameserver",
      "network/dns",
    ),
    #[cfg(feature = "network")]
    (
      "network.connections",
      "network",
      "control_center.net_connections",
      "connections connection profiles edit nmtui conexoes conexões perfis editar",
      "network/connections",
    ),
    #[cfg(feature = "network")]
    (
      "network.activate",
      "network",
      "control_center.net_activate_connection",
      "activate connection up down ativar conexao conexão",
      "network/activate",
    ),
    #[cfg(feature = "network")]
    (
      "network.add_connection",
      "network",
      "control_center.net_add_connection",
      "add new connection bridge bond team vlan tunnel loopback wireguard hotspot pppoe dsl openvpn import adicionar nova conexao conexão importar",
      "network/add",
    ),
    #[cfg(feature = "network")]
    (
      "network.proxy",
      "network",
      "control_center.proxy",
      "proxy http https",
      "network/proxy",
    ),
    #[cfg(feature = "audio")]
    (
      "audio.output",
      "audio",
      "control_center.output",
      "audio sound som output saída",
      "audio/output",
    ),
    #[cfg(feature = "audio")]
    (
      "audio.input",
      "audio",
      "control_center.input",
      "audio microphone microfone input entrada",
      "audio/input",
    ),
    #[cfg(feature = "audio")]
    (
      "audio.devices",
      "audio",
      "control_center.devices",
      "audio devices dispositivos",
      "audio/devices",
    ),
    #[cfg(feature = "bluetooth")]
    (
      "bluetooth.state",
      "bluetooth",
      "control_center.state",
      "bluetooth bt estado",
      "bluetooth/state",
    ),
    #[cfg(feature = "bluetooth")]
    (
      "bluetooth.devices",
      "bluetooth",
      "control_center.devices",
      "bluetooth dispositivos",
      "bluetooth/devices",
    ),
    #[cfg(feature = "bluetooth")]
    (
      "bluetooth.pair",
      "bluetooth",
      "control_center.pair",
      "bluetooth pair parear",
      "bluetooth/pair",
    ),
    #[cfg(feature = "boot")]
    (
      "boot.summary",
      "boot",
      "control_center.summary",
      "boot startup inicializacao kernel",
      "boot/summary",
    ),
    #[cfg(feature = "boot")]
    (
      "boot.kernel",
      "boot",
      "control_center.kernel",
      "kernel linux",
      "boot/kernel",
    ),
    #[cfg(feature = "boot")]
    (
      "boot.bootloader",
      "boot",
      "control_center.bootloader_f1a3c5",
      "bootloader grub systemd-boot loader",
      "boot/bootloader",
    ),
    #[cfg(feature = "boot")]
    (
      "boot.initramfs",
      "boot",
      "control_center.initramfs",
      "initramfs mkinitcpio",
      "boot/initramfs",
    ),
    #[cfg(feature = "boot")]
    (
      "boot.plymouth",
      "boot",
      "control_center.plymouth",
      "plymouth splash",
      "boot/plymouth",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.official",
      "packages",
      "control_center.official_repositories",
      "pacman package pacote official oficial repositories repositórios",
      "packages/official",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.search",
      "packages",
      "control_center.search_official_install",
      "pacman package pacote install instalar official oficial search",
      "packages/search",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.installed",
      "packages",
      "control_center.search_official_remove",
      "installed instalados remove remover uninstall desinstalar official oficial",
      "packages/installed",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.updates",
      "packages",
      "control_center.search_official_update",
      "update upgrade atualizar official oficial",
      "packages/updates",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.orphans",
      "packages",
      "control_center.search_orphans",
      "orphan órfão orfao remove remover",
      "packages/orphans",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.cache",
      "packages",
      "control_center.cache",
      "cache paccache",
      "packages/cache",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.repositories-add",
      "packages",
      "control_center.search_repositories_add",
      "repository repositório add adicionar pacman.conf pacman-key gpg key chave",
      "packages/repositories-add",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.repositories-list",
      "packages",
      "control_center.search_repositories_list",
      "repository repositório list listar disable desativar remove remover pacman.conf",
      "packages/repositories-list",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.aur",
      "packages",
      "control_center.search_aur_install",
      "aur paru yay install instalar",
      "packages/aur",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.aur-remove",
      "packages",
      "control_center.search_aur_remove",
      "aur paru yay remove remover uninstall desinstalar",
      "packages/aur-remove",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.aur-updates",
      "packages",
      "control_center.search_aur_update",
      "aur paru yay update upgrade atualizar",
      "packages/aur-updates",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.history",
      "packages",
      "control_center.history",
      "history histórico pacman.log",
      "packages/history",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.downgrade",
      "packages",
      "control_center.downgrade",
      "downgrade",
      "packages/downgrade",
    ),
    #[cfg(feature = "packages")]
    (
      "packages.mirrors",
      "packages",
      "control_center.mirrors",
      "mirror reflector",
      "packages/mirrors",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.summary",
      "storage",
      "control_center.summary",
      "storage armazenamento disk disco",
      "storage/summary",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.disks",
      "storage",
      "control_center.disks",
      "disk disco nvme ssd hdd",
      "storage/disks",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.partitions",
      "storage",
      "control_center.partitions",
      "partition partição",
      "storage/partitions",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.filesystems",
      "storage",
      "control_center.filesystems",
      "filesystem sistema arquivos",
      "storage/filesystems",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.mounts",
      "storage",
      "control_center.mount_points",
      "mount montagem",
      "storage/mounts",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.smart",
      "storage",
      "control_center.smart",
      "smart health saúde",
      "storage/smart",
    ),
    #[cfg(feature = "storage")]
    (
      "storage.usage",
      "storage",
      "control_center.disk_usage",
      "space espaço uso disco",
      "storage/usage",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.summary",
      "diagnostics",
      "control_center.summary",
      "diagnostic diagnóstico health status",
      "diagnostics/summary",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.services",
      "diagnostics",
      "control_center.services",
      "services serviços failed falhos",
      "diagnostics/services",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.boot",
      "diagnostics",
      "control_center.kernel_boot",
      "boot kernel initramfs",
      "diagnostics/boot",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.graphics",
      "diagnostics",
      "control_center.graphics",
      "graphics gráficos gpu vmwgfx",
      "diagnostics/graphics",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.network",
      "diagnostics",
      "control_center.network",
      "network rede dns",
      "diagnostics/network",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.audio",
      "diagnostics",
      "control_center.audio",
      "audio áudio pipewire",
      "diagnostics/audio",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.bluetooth",
      "diagnostics",
      "control_center.bluetooth",
      "bluetooth",
      "diagnostics/bluetooth",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.storage",
      "diagnostics",
      "control_center.storage",
      "storage armazenamento smart",
      "diagnostics/storage",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.packages",
      "diagnostics",
      "control_center.section_packages",
      "packages pacotes updates",
      "diagnostics/packages",
    ),
    #[cfg(feature = "diagnostics")]
    (
      "diagnostics.argvus",
      "diagnostics",
      "control_center.argvus",
      "argvus session theme tema",
      "diagnostics/argvus",
    ),
    #[cfg(feature = "power")]
    (
      "power.summary",
      "power",
      "control_center.summary",
      "power energia energy battery bateria",
      "power/summary",
    ),
    #[cfg(feature = "session")]
    (
      "session.summary",
      "session",
      "control_center.summary",
      "session sessão inicio inicializacao",
      "session/summary",
    ),
    #[cfg(feature = "session")]
    (
      "session.components",
      "session",
      "control_center.components_7073b6",
      "components componentes desktop wayland hyprland",
      "session/components",
    ),
    #[cfg(feature = "session")]
    (
      "session.autostart",
      "session",
      "control_center.autostart",
      "autostart login inicialização iniciar",
      "session/autostart",
    ),
    #[cfg(feature = "session")]
    (
      "session.diagnostics",
      "session",
      "control_center.diagnostics",
      "diagnostic diagnóstico",
      "session/diagnostics",
    ),
    #[cfg(feature = "session")]
    (
      "session.logs",
      "session",
      "control_center.logs",
      "logs journal journalctl",
      "session/logs",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.night-light",
      "displays",
      "control_center.night_light.title",
      "night light luz noturna temperatura temperature kelvin schedule horarios",
      "displays/night-light",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.summary",
      "displays",
      "control_center.summary",
      "displays monitores monitor screens telas",
      "displays/summary",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.mode",
      "displays",
      "control_center.resolution",
      "resolution resolucao modo mode refresh",
      "displays/mode",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.scale",
      "displays",
      "control_center.scale",
      "scale escala zoom",
      "displays/scale",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.position",
      "displays",
      "control_center.position",
      "position posicao layout arrangement",
      "displays/position",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.vrr",
      "displays",
      "control_center.vrr",
      "vrr variable refresh rate freesync",
      "displays/vrr",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.hdr",
      "displays",
      "control_center.hdr",
      "hdr high dynamic range",
      "displays/hdr",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.orientation",
      "displays",
      "control_center.orientation",
      "orientation orientacao rotate rotacionar",
      "displays/orientation",
    ),
    #[cfg(feature = "displays")]
    (
      "displays.primary",
      "displays",
      "control_center.primary_monitor",
      "primary principal",
      "displays/primary",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.summary",
      "appearance",
      "control_center.summary",
      "appearance aparencia theme tema wallpaper papel de parede accent cor destaque",
      "appearance/summary",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.themes",
      "appearance",
      "control_center.themes",
      "themes temas theme tema appearance aparencia",
      "appearance/themes",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.wallpapers",
      "appearance",
      "control_center.wallpapers",
      "wallpapers papéis de parede papel wallpaper plano de fundo",
      "appearance/wallpapers",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.icons",
      "appearance",
      "control_center.icons",
      "icons ícones icones icon theme tema de ícones tema de icones papirus adwaita",
      "appearance/icons",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.accents",
      "appearance",
      "control_center.highlight_color",
      "accent cor de destaque highlight color aparencia",
      "appearance/accents",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.dock",
      "appearance",
      "control_center.dock",
      "dock magnification visibility monitor",
      "appearance/dock",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar",
      "appearance",
      "control_center.taskbar",
      "taskbar barra grupo utility group hover expandido expanded sempre always",
      "appearance/taskbar",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_position",
      "taskbar",
      "control_center.taskbar_position",
      "taskbar barra posicao position gaps espacamento margin margem",
      "appearance/taskbar-position",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_icons",
      "taskbar",
      "control_center.taskbar_icons",
      "taskbar barra icones icons audio player spotify launcher search utilities network power profile keyboard layout memory cpu temperature gpu usage",
      "appearance/taskbar-icons",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_date",
      "taskbar",
      "control_center.taskbar_date",
      "taskbar barra data date format weekday dia mes month year ano",
      "appearance/taskbar-date",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_time",
      "taskbar",
      "control_center.taskbar_time",
      "taskbar barra hora time clock relogio seconds segundos 24h am pm",
      "appearance/taskbar-time",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_height",
      "taskbar",
      "control_center.height",
      "taskbar barra altura height tamanho size 24 28 32 36 pixels",
      "appearance/taskbar-height",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.taskbar_border",
      "taskbar",
      "control_center.border",
      "taskbar barra borda border espessura thickness enable ligar desligar",
      "appearance/taskbar-border",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.widget_telemetry",
      "appearance",
      "control_center.widget_telemetry",
      "widget telemetry system cpu gpu memory storage processes network shortcuts",
      "appearance/widget-telemetry",
    ),
    #[cfg(feature = "appearance")]
    (
      "appearance.control_panel",
      "appearance",
      "control_center.control_panel",
      "control panel painel de controle cards sessions sessoes transparency transparencia",
      "appearance/control-panel",
    ),
  ];
  for (id, category, title_key, keywords, route) in entries {
    let mut words: Vec<String> = keywords.split_whitespace().map(str::to_owned).collect();
    words.push(tr(english, title_key).to_owned());
    for key in page_content_keys(route) {
      for locale in [lang, english] {
        words.push(tr(locale, key).to_owned());
      }
    }
    let _ = search_registry.register(SearchEntry {
      id: id.into(),
      category: category.into(),
      title: tr(lang, title_key).into(),
      keywords: words,
      route: route.into(),
    });
  }
  #[cfg(feature = "appearance")]
  {
    let mut model = argvus_control_center_appearance::AppearanceApp::new(lang, Theme::load());
    for &(route, page) in APPEARANCE_PAGES {
      model.page = page;
      model.lang = english;
      let (english_path, mut words) = model.search_content();
      model.lang = lang;
      let (path, labels) = model.search_content();
      words.extend(labels);
      words.extend([english_path, path.clone()]);
      let (parent, title) = path.rsplit_once(" > ").unwrap_or(("", &path));
      if let Some(entry) = search_registry.entry_for_route_mut(route) {
        entry.keywords.extend(words);
        if !parent.is_empty() {
          entry.category = parent.into();
        }
      } else {
        let _ = search_registry.register(SearchEntry {
          id: route.replace('/', "."),
          category: parent.into(),
          title: title.into(),
          keywords: words,
          route: route.into(),
        });
      }
    }
  }
  for &(route, page) in SETTINGS_PAGES {
    let label = |lang| match page {
      Page::AppSelector(category) => Some(argvus_control_center_settings::app::category_label(
        lang, category,
      )),
      Page::FontSelector(target) => Some(argvus_control_center_settings::app::font_target_label(
        lang, target,
      )),
      Page::SettingSelector(setting) => Some(argvus_control_center_settings::app::setting_label(
        lang, setting,
      )),
      _ => None,
    };
    if let Some(title) = label(lang) {
      let category = match page {
        Page::AppSelector(_) => tr(lang, "control_center.default_apps"),
        _ => tr(lang, "control_center.fonts"),
      };
      let _ = search_registry.register(SearchEntry {
        id: route.replace('/', "."),
        title: title.to_owned(),
        category: category.to_owned(),
        keywords: label(english).map(str::to_owned).into_iter().collect(),
        route: route.to_owned(),
      });
    }
  }
  search_registry
}

fn page_content_keys(route: &str) -> &[&str] {
  match route {
    "audio/devices" => &[
      "control_center.pipewire_wireplumber_is_unavailable",
      "control_center.output_devices",
      "control_center.no_output_found",
      "control_center.default",
      "control_center.muted",
      "control_center.input_devices",
      "control_center.no_input_found",
    ],
    "audio/home" => &[
      "control_center.summary",
      "control_center.output",
      "control_center.input",
      "control_center.devices",
      "control_center.active",
      "control_center.unavailable",
      "control_center.outputs",
      "control_center.inputs",
      "control_center.devices_a41e65",
    ],
    "audio/input" => &[
      "control_center.pipewire_wireplumber_is_unavailable",
      "control_center.no_devices_found",
      "control_center.default",
      "control_center.muted",
    ],
    "audio/output" => &[
      "control_center.pipewire_wireplumber_is_unavailable",
      "control_center.no_devices_found",
      "control_center.default",
      "control_center.muted",
    ],
    "boot/bootloader" => &[
      "control_center.section_entries",
      "control_center.default",
      "control_center.configuration",
      "control_center.timeout",
      "control_center.unavailable_unknown_bootloader",
      "control_center.kernel_command_line",
      "control_center.danger_zone",
      "control_center.regenerate_grub",
    ],
    "boot/home" => &[
      "control_center.secure_boot",
      "control_center.secure_boot_n_a",
      "control_center.timeout",
      "control_center.timeout_default",
      "control_center.not_installed",
      "control_center.installed",
      "control_center.summary",
      "control_center.kernels",
      "control_center.bootloader_f1a3c5",
      "control_center.initramfs",
      "control_center.plymouth",
    ],
    "boot/initramfs" => &["control_center.section_presets"],
    "boot/summary" => &[
      "control_center.default",
      "control_center.unavailable",
      "control_center.section_base_system",
      "control_center.firmware",
      "control_center.secure_boot",
      "control_center.kernel",
      "control_center.bootloader_f1a3c5",
      "control_center.boot_manager",
      "control_center.default_entry",
      "control_center.section_entries",
      "control_center.esp_path",
      "control_center.timeout",
      "control_center.section_components",
      "control_center.kernels",
      "control_center.plymouth",
      "control_center.theme",
    ],
    "diagnostics/summary" => &[
      "control_center.diagnostics_cc02b3",
      "control_center.checks_75a9fc",
      "control_center.doctor_problem",
      "control_center.errors",
      "control_center.warnings",
      "control_center.doctor_unavailable",
      "control_center.doctor_last_run",
      "control_center.system_1af4b7",
    ],
    "displays/night-light" => &[
      "control_center.refresh",
      "control_center.off",
      "control_center.unavailable",
      "control_center.none",
    ],
    "displays/summary" => &[
      "control_center.primary_ead365",
      "control_center.focused",
      "control_center.off",
      "control_center.on",
      "control_center.scale",
      "control_center.disconnected",
      "control_center.configured",
      "control_center.no_monitors_found_is_hyprland_running",
      "control_center.profiles",
      "control_center.refresh",
    ],
    "hardware/cpu" => &["control_center.active"],
    "hardware/gpu" => &[
      "control_center.unavailable_6a8fc3",
      "control_center.no_driver",
    ],
    "hardware/home" => &[
      "control_center.summary",
      "control_center.memory",
      "control_center.power",
      "control_center.devices",
      "control_center.no_gpu",
      "control_center.default",
      "control_center.devices_a41e65",
    ],
    "hardware/power" => &["control_center.active"],
    "network/add" => &[
      "control_center.net_group_hardware",
      "control_center.net_group_virtual",
      "control_center.vpn",
      "control_center.net_requires_vpn_plugin",
      "control_center.net_group_import",
      "control_center.net_import_wireguard",
      "control_center.net_import_plugin",
    ],
    "network/connections" => &[
      "control_center.section_actions",
      "control_center.net_add_connection",
      "control_center.refresh",
    ],
    "network/dns" => &[
      "control_center.automatic",
      "control_center.section_actions",
      "control_center.manual_dns",
      "control_center.automatic_dns",
      "control_center.refresh",
      "control_center.section_summary",
      "control_center.source_2c26d9",
      "control_center.servers",
      "control_center.search_domains",
    ],
    "network/ethernet" => &["control_center.no_connection", "control_center.connected"],
    "network/home" => &[
      "control_center.none_247448",
      "control_center.disabled_ac84bd",
      "control_center.disconnected",
      "control_center.inactive_6f56ef",
      "control_center.automatic",
      "control_center.active_095d39",
      "control_center.disabled",
      "control_center.status",
      "control_center.interfaces",
      "control_center.ethernet",
      "control_center.wi_fi",
      "control_center.vpn",
      "control_center.dns",
      "control_center.proxy",
      "control_center.net_connections",
      "control_center.net_activate_connection",
      "control_center.loading",
    ],
    "network/interfaces" => &["control_center.no_connection", "control_center.connected"],
    "network/proxy" => &[
      "control_center.section_actions",
      "control_center.refresh",
      "control_center.section_summary",
    ],
    "network/status" => &[
      "control_center.wi_fi",
      "control_center.loading",
      "control_center.unavailable_no_wifi_card",
      "control_center.disabled_ac84bd",
      "control_center.inactive",
      "control_center.inactive_6f56ef",
      "control_center.section_actions",
      "control_center.refresh",
      "control_center.section_summary",
      "control_center.connectivity",
      "control_center.interface_df13f7",
      "control_center.ipv4",
      "control_center.ipv6",
      "control_center.gateway",
      "control_center.wi_fi_ac2ca3",
      "control_center.dns_d80a0d",
      "control_center.vpn_fda889",
    ],
    "network/vpn" => &["control_center.active_c7cc67"],
    "network/wifi" => &["control_center.connected", "control_center.saved"],
    "packages/cache" => &[
      "control_center.section_summary",
      "control_center.cache_file_count",
      "control_center.total_size",
      "control_center.cache_files",
      "control_center.files_7093b3",
      "control_center.danger_zone",
      "control_center.clean_cache_keep_three",
      "control_center.clean_cache_keep_one",
      "control_center.clean_cache_uninstalled",
    ],
    "packages/home" => &[
      "control_center.packages_c945db",
      "control_center.pending",
      "control_center.cache",
      "control_center.files_7093b3",
      "control_center.history",
      "control_center.entries",
      "control_center.downgrade",
      "control_center.versions",
      "control_center.mirrors",
      "control_center.active_ae7190",
    ],
    "packages/mirrors" => &[
      "control_center.configure_mirrors",
      "control_center.section_actions",
      "control_center.unavailable_no_reflector",
      "control_center.section_servers",
      "control_center.mirror_enabled",
      "control_center.mirror_disabled",
    ],
    "packages/orphans" => &["control_center.danger_zone", "control_center.remove_marked"],
    "packages/repositories-add" => &[
      "control_center.repo_section_repository",
      "control_center.repo_source",
      "control_center.repo_conf_url",
      "control_center.repo_conf_text",
      "control_center.repo_name",
      "control_center.repo_server",
      "control_center.repo_sig_level",
      "control_center.repo_section_key",
      "control_center.repo_key",
      "control_center.repo_key_url",
      "control_center.repo_key_text",
      "control_center.repo_review",
    ],
    "packages/repositories-list" => &["control_center.repo_none"],
    "services/home" => &[
      "control_center.previous",
      "control_center.current_c42247",
      "control_center.system",
      "control_center.active_ae7190",
      "control_center.enabled_26a3ab",
      "control_center.user",
      "control_center.failed",
      "control_center.failed_cc0486",
      "control_center.logs",
      "control_center.boot",
      "control_center.priority",
    ],
    "session/diagnostics" => &["control_center.loading_diagnostics"],
    "session/summary" => &[
      "control_center.components",
      "control_center.active_ae7190",
      "control_center.autostart",
      "control_center.enabled_72aa06",
      "control_center.diagnostics",
      "control_center.failed_cc0486",
      "control_center.logs",
    ],
    "settings/default-apps" => &["control_center.reset_defaults"],
    "settings/shortcuts-create" => &[
      "control_center.save",
      "control_center.create",
      "control_center.cancel",
    ],
    "settings/ssh-keys" => &[
      "control_center.ssh_keys_refresh",
      "control_center.ssh_keys_unload_all",
    ],
    "storage/home" => &[
      "control_center.summary",
      "control_center.disks",
      "control_center.partitions",
      "control_center.filesystems",
      "control_center.mount_points",
      "control_center.disk_usage",
      "control_center.disks_5d5d98",
    ],
    "storage/mounts" => &["control_center.ro"],
    "storage/smart" => &[
      "control_center.no_smart_data_available",
      "control_center.smartctl_is_not_installed",
      "control_center.failed",
    ],
    "storage/summary" => &[
      "control_center.no_data_collected",
      "control_center.unavailable",
      "control_center.available",
      "control_center.used",
      "control_center.storage_summary",
      "control_center.disks_0a36a0",
      "control_center.total_size",
      "control_center.partitions_2d5f30",
      "control_center.filesystems_602f7b",
      "control_center.mount_points_78b940",
      "control_center.swap",
      "control_center.smart",
      "control_center.usage_by_filesystem",
      "control_center.no_filesystems_found",
    ],
    "storage/usage" => &["control_center.swap_0b62a3"],
    "power/summary" => &[
      "control_center.loading",
      "control_center.on_lid_close_battery",
      "control_center.on_lid_close_ac",
      "control_center.power_button",
      "control_center.screen_off_after",
      "control_center.via_argvus_hypridle",
      "control_center.lock_screen_after",
      "control_center.keep_awake",
      "control_center.enabled",
      "control_center.disabled",
      "control_center.ignore",
      "control_center.power_off",
      "control_center.reboot",
      "control_center.halt",
      "control_center.suspend",
      "control_center.hibernate",
      "control_center.suspend_hibernate",
      "control_center.lock",
    ],
    "bluetooth/state" => &[
      "control_center.adapter",
      "control_center.powered",
      "control_center.yes",
      "control_center.no",
    ],
    "bluetooth/devices" => &[
      "control_center.connected",
      "control_center.paired",
      "control_center.known",
      "control_center.trusted",
      "control_center.unnamed",
    ],
    "bluetooth/pair" => &[
      "control_center.connected",
      "control_center.paired",
      "control_center.known",
      "control_center.trusted",
      "control_center.unnamed",
    ],
    "services/system" => &[
      "control_center.no_failed_units",
      "control_center.user_e7acba",
    ],
    "services/user" => &[
      "control_center.no_failed_units",
      "control_center.user_e7acba",
    ],
    "services/failed" => &[
      "control_center.no_failed_units",
      "control_center.user_e7acba",
    ],
    "services/logs" => &[
      "control_center.log_not_found",
      "control_center.timestamp",
      "control_center.unit",
      "control_center.priority",
      "control_center.executable",
      "control_center.boot",
    ],
    _ => &[],
  }
}

#[cfg(feature = "appearance")]
pub(super) use argvus_control_center_appearance::search::SEARCH_PAGES as APPEARANCE_PAGES;

pub(super) const SETTINGS_PAGES: &[(&str, Page)] = &[
  #[cfg(feature = "apps")]
  (
    "default-apps/terminal",
    Page::AppSelector(argvus_control_center_settings::Category::Terminal),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/file-manager",
    Page::AppSelector(argvus_control_center_settings::Category::FileManager),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/text-editor",
    Page::AppSelector(argvus_control_center_settings::Category::TextEditor),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/terminal-editor",
    Page::AppSelector(argvus_control_center_settings::Category::TerminalEditor),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/browser",
    Page::AppSelector(argvus_control_center_settings::Category::Browser),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/image-viewer",
    Page::AppSelector(argvus_control_center_settings::Category::ImageViewer),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/pdf-viewer",
    Page::AppSelector(argvus_control_center_settings::Category::PdfViewer),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/video-player",
    Page::AppSelector(argvus_control_center_settings::Category::VideoPlayer),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/audio-player",
    Page::AppSelector(argvus_control_center_settings::Category::AudioPlayer),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/archive",
    Page::AppSelector(argvus_control_center_settings::Category::Archive),
  ),
  #[cfg(feature = "apps")]
  (
    "default-apps/launcher",
    Page::AppSelector(argvus_control_center_settings::Category::Launcher),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/taskbar",
    Page::FontSelector(argvus_control_center_settings::FontTarget::Taskbar),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/sysinfo",
    Page::FontSelector(argvus_control_center_settings::FontTarget::Sysinfo),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/control-panel",
    Page::FontSelector(argvus_control_center_settings::FontTarget::ControlPanel),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/system",
    Page::FontSelector(argvus_control_center_settings::FontTarget::System),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/apps",
    Page::FontSelector(argvus_control_center_settings::FontTarget::Apps),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/terminal",
    Page::FontSelector(argvus_control_center_settings::FontTarget::Terminal),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/browser",
    Page::FontSelector(argvus_control_center_settings::FontTarget::Browser),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/antialiasing",
    Page::SettingSelector(argvus_control_center_settings::SettingKind::Antialiasing),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/hinting",
    Page::SettingSelector(argvus_control_center_settings::SettingKind::Hinting),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/subpixel",
    Page::SettingSelector(argvus_control_center_settings::SettingKind::Subpixel),
  ),
  #[cfg(feature = "fonts")]
  (
    "fonts/dpi",
    Page::SettingSelector(argvus_control_center_settings::SettingKind::Dpi),
  ),
  ("settings/default-apps", Page::DefaultApps),
  ("settings/language", Page::Language),
  ("settings/projects", Page::Projects),
  ("settings/snippets", Page::Snippets),
  ("settings/ssh-keys", Page::SshKeys),
  ("settings/gpg-keys", Page::GpgKeys),
  ("settings/firewall", Page::Firewall),
  ("settings/hostname", Page::Hostname),
  ("settings/users-groups", Page::UsersGroups),
  ("settings/notifications", Page::Notifications),
  ("settings/fonts", Page::Fonts),
  ("settings/keybindings", Page::Keybindings),
  ("settings/shortcuts-create", Page::CustomShortcut),
  ("settings/shortcuts-list", Page::AllShortcuts),
  #[cfg(feature = "locale")]
  ("settings/locale", Page::LocaleRegion),
  #[cfg(feature = "locale")]
  ("settings/time-zone", Page::TimeZone),
  #[cfg(feature = "locale")]
  ("settings/regional-locale", Page::RegionalLocale),
  #[cfg(feature = "locale")]
  ("settings/date-time", Page::DateTime),
  #[cfg(feature = "locale")]
  ("settings/system-locales", Page::SystemLocales),
  #[cfg(feature = "locale")]
  ("settings/keyboard", Page::Keyboard),
  #[cfg(feature = "locale")]
  ("settings/keyboard-layout", Page::KeyboardLayout),
  #[cfg(feature = "locale")]
  ("settings/keyboard-variant", Page::KeyboardVariant),
  #[cfg(feature = "locale")]
  ("settings/console-keymap", Page::ConsoleKeymap),
  #[cfg(feature = "hardware")]
  ("settings/mouse-touchpad", Page::MouseTouchpad),
  #[cfg(feature = "system")]
  ("settings/users", Page::Users),
  #[cfg(feature = "system")]
  ("settings/users-list", Page::UserList),
  #[cfg(feature = "system")]
  ("settings/groups", Page::Groups),
  #[cfg(feature = "system")]
  ("settings/groups-list", Page::GroupList),
];

#[cfg(feature = "audio")]
pub(super) const AUDIO_PAGES: &[(&str, AudioPage)] = &[
  ("audio/summary", AudioPage::Summary),
  ("audio/home", AudioPage::Home),
  ("audio/output", AudioPage::Output),
  ("audio/input", AudioPage::Input),
  ("audio/devices", AudioPage::Devices),
];

#[cfg(feature = "hardware")]
pub(super) const HARDWARE_PAGES: &[(&str, HardwarePage)] = &[
  ("hardware/summary", HardwarePage::Summary),
  ("hardware/home", HardwarePage::Home),
  ("hardware/cpu", HardwarePage::Cpu),
  ("hardware/gpu", HardwarePage::Gpu),
  ("hardware/memory", HardwarePage::Memory),
  ("hardware/power", HardwarePage::Power),
  ("hardware/devices", HardwarePage::Devices),
];

#[cfg(feature = "boot")]
pub(super) const BOOT_PAGES: &[(&str, BootPage)] = &[
  ("boot/summary", BootPage::Summary),
  ("boot/home", BootPage::Home),
  ("boot/kernel", BootPage::Kernel),
  ("boot/bootloader", BootPage::Bootloader),
  ("boot/initramfs", BootPage::Initramfs),
  ("boot/plymouth", BootPage::Plymouth),
];

#[cfg(feature = "network")]
pub(super) const NETWORK_PAGES: &[(&str, NetworkPage)] = &[
  ("network/status", NetworkPage::Status),
  ("network/home", NetworkPage::Home),
  ("network/interfaces", NetworkPage::Interfaces),
  ("network/wifi", NetworkPage::Wifi),
  ("network/ethernet", NetworkPage::Ethernet),
  ("network/vpn", NetworkPage::Vpn),
  ("network/dns", NetworkPage::Dns),
  ("network/proxy", NetworkPage::Proxy),
  ("network/connections", NetworkPage::Connections),
  ("network/activate", NetworkPage::Activate),
  ("network/add", NetworkPage::AddConnection),
];

#[cfg(feature = "storage")]
pub(super) const STORAGE_PAGES: &[(&str, StoragePage)] = &[
  ("storage/summary", StoragePage::Summary),
  ("storage/home", StoragePage::Home),
  ("storage/disks", StoragePage::Disks),
  ("storage/partitions", StoragePage::Partitions),
  ("storage/filesystems", StoragePage::Filesystems),
  ("storage/mounts", StoragePage::Mounts),
  ("storage/smart", StoragePage::Smart),
  ("storage/usage", StoragePage::Usage),
];

#[cfg(feature = "displays")]
pub(super) const DISPLAYS_PAGES: &[(&str, DisplayPage)] = &[
  ("displays/summary", DisplayPage::Home),
  ("displays/profiles", DisplayPage::Profiles),
  ("displays/night-light", DisplayPage::NightLight),
];

#[cfg(feature = "packages")]
pub(super) const PACKAGES_PAGES: &[(&str, PackagesPage)] = &[
  ("packages/home", PackagesPage::Home),
  ("packages/orphans", PackagesPage::Orphans),
  ("packages/cache", PackagesPage::Cache),
  ("packages/repositories", PackagesPage::Repositories),
  ("packages/repositories-add", PackagesPage::RepositoryAdd),
  ("packages/repositories-list", PackagesPage::RepositoryList),
  ("packages/history", PackagesPage::History),
  ("packages/downgrade", PackagesPage::Downgrade),
  ("packages/mirrors", PackagesPage::Mirrors),
];

#[cfg(feature = "diagnostics")]
pub(super) const DIAGNOSTICS_PAGES: &[(&str, DiagnosticPage)] = &[
  ("diagnostics/summary", DiagnosticPage::Summary),
  ("diagnostics/home", DiagnosticPage::Home),
  ("diagnostics/services", DiagnosticPage::Services),
  ("diagnostics/boot", DiagnosticPage::Boot),
  ("diagnostics/graphics", DiagnosticPage::Graphics),
  ("diagnostics/network", DiagnosticPage::Network),
  ("diagnostics/audio", DiagnosticPage::Audio),
  ("diagnostics/bluetooth", DiagnosticPage::Bluetooth),
  ("diagnostics/storage", DiagnosticPage::Storage),
  ("diagnostics/packages", DiagnosticPage::Packages),
  ("diagnostics/argvus", DiagnosticPage::Argvus),
];

#[cfg(feature = "session")]
pub(super) const SESSION_PAGES: &[(&str, SessionPage)] = &[
  ("session/summary", SessionPage::Home),
  ("session/components", SessionPage::Components),
  ("session/autostart", SessionPage::Autostart),
  ("session/diagnostics", SessionPage::Diagnostics),
  ("session/logs", SessionPage::Logs),
];

#[cfg(feature = "about")]
pub(super) const ABOUT_TABS: &[(&str, Tab)] = &[
  ("about/system", Tab::System),
  ("about/about", Tab::About),
  ("about/donate", Tab::Donate),
  ("about/credits", Tab::Credits),
  ("about/copyright", Tab::Copyright),
];

#[cfg(feature = "bluetooth")]
pub(super) const BLUETOOTH_PAGES: &[(&str, BluetoothPage)] = &[
  ("bluetooth/state", BluetoothPage::State),
  ("bluetooth/home", BluetoothPage::Home),
  ("bluetooth/devices", BluetoothPage::Devices),
  ("bluetooth/pair", BluetoothPage::Pair),
];

#[cfg(feature = "services")]
pub(super) const SERVICES_PAGES: &[(&str, ServicePage)] = &[
  ("services/system", ServicePage::System),
  ("services/home", ServicePage::Home),
  ("services/user", ServicePage::User),
  ("services/failed", ServicePage::Failed),
  ("services/logs", ServicePage::Logs),
];
