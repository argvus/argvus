//! Entries of the global search on the Home: one per destination, with the
//! search category, the title key, extra keywords (English and Portuguese)
//! and the route that `open_search_route` understands.

use super::*;

/// Registers every searchable destination compiled into this build. Entries
/// behind a disabled feature are left out, so search never offers a page the
/// build cannot open.
pub(super) fn build_search_registry(lang: Lang) -> SearchRegistry {
  let mut search_registry = SearchRegistry::default();
  let entries: [(&str, &str, &str, &str, &str); _] = [
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
    #[cfg(feature = "apps")]
    (
      "settings.window_rules",
      "hyprland",
      "control_center.window_rules",
      "window rules workspace class app placement regras janela area de trabalho",
      "settings/window-rules",
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
      "control_center.diagnostics",
      "diagnostics health",
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
    let _ = search_registry.register(SearchEntry {
      id: id.into(),
      category: category.into(),
      title: tr(lang, title_key).into(),
      keywords: keywords.split_whitespace().map(str::to_owned).collect(),
      route: route.into(),
    });
  }
  search_registry
}
