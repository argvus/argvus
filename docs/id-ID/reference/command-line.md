---
title: Baris perintah
description: Perintah argvus, sub-perintahnya, flag yang disimpan untuk pengikatan kunci, dan di mana masing-masing perintah dijalankan.
---

`/usr/bin/argvus` adalah titik masuk publik ARGVUS. Ini adalah salah satu program Rust yang dibuat dari peti `argvus-cli`. Setiap subperintah dijalankan di dalam program itu, menjalankan helper yang dapat dieksekusi dari direktori perintah, atau menjalankan skrip yang dikemas.

Jalankan `argvus --help` untuk daftar yang diinstal dengan versi Anda. Sub-perintah ditulis tanpa awalan `--`. Formulir `--` disimpan hanya jika pengikatan kunci atau entri desktop menggunakannya.

## Tempat setiap perintah dijalankan

- **Dalam proses:** perintahnya adalah pustaka ruang kerja dan dijalankan di dalam `argvus`. Tidak ada executable terpisah. Ini adalah `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` dan `setup`.
- **Helper:** executable terpisah di `/usr/lib/argvus/commands/`, diinstal oleh paket `argvus`. Ini adalah `control-center` (aplikasi terminal), `argvus-du` (diluncurkan dengan `argvus du`) dan aplikasi GTK 4 `taskbar-calendar`, `removable-devices`, `focus` dan `clock`.
- **Private executable:** executable yang diinstal paket lain di `/usr/lib/argvus/commands/`. `argvus` hanya memulainya. Ini termasuk `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` dan `greeter-*` titik masuk.
- **Skrip:** skrip shell terpaket yang dijalankan dengan `/bin/sh`. Ini adalah perintah skrip Hyprland yang tercantum di bawah.

`ARGVUS_COMMANDS_DIR` mengubah direktori perintah, untuk pengujian dan pemeriksaan `DESTDIR`. Eksekusi pribadi yang tidak ditemukan di sana dicari di `PATH`.

## Subperintah

| Perintah | Berjalan | Catatan |
| --- | --- | --- |
| `argvus config ...` | Dalam proses | Model dan alat konfigurasi. Juga dijangkau oleh `theme-switch` dan `effects-toggle`. |
| `argvus theme ...` | Dalam proses | `apply ID` adalah pengalih tema (sama dengan `theme-switch`). Pembuatan tema: `list` tema yang diinstal, `check DIRECTORY` tema terhadap template yang diinstal, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Lihat [Penampilan dan tema](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | Dalam proses | Terjemahan: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | Dalam proses | Manajer kunci SSH. `gpg` adalah alias untuk subperintah GPG. |
| `argvus gpg ...` | Dalam proses | Kunci GPG: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | Dalam proses | Pemilih proyek. |
| `argvus terminal ...` | Dalam proses | Profil terminal ARGVUS. |
| `argvus system-monitor ...` | Dalam proses | Monitor sistem btop; `--gpu` memilih kotak khusus GPU, dan `--panel [--cpu|--mem|--gpu]` menyematkannya di shell ARGVUS TUI. |
| `argvus du` | Pembantu (aplikasi terminal) | Membuka ARGVUS Disk Usage Analyzer di jendela Kitty khusus dan menjalankan ncdu. |
| `argvus network ...` | Dalam proses | Status dan kontrol NetworkManager (`networkctl` disimpan sebagai alias). |
| `argvus bluetooth ...` | Dalam proses | Status dan kontrol Bluetooth (`bluetoothctl` disimpan sebagai alias). |
| `argvus widget-telemetry block ...` | Dalam proses | Blok telemetri Waybar. Ini adalah jalur panas bar. || `argvus widget-telemetry toggle ...` | Dalam proses | Mengaktifkan dan menonaktifkan blok telemetri, serta urutannya. |
| `argvus game-snake` | Eksekusi pribadi | Game terminal Snake, dari proyek independen `argvus-game-snake`, merupakan paket opsional. Di luar terminal, ia membukanya. |
| `argvus control-center [page]` | Pembantu (aplikasi terminal) | Membuka Pusat Kontrol, atau memfokuskan jendela yang sudah terbuka. |
| `argvus welcome [--if-needed]` | Operator → Pusat Kontrol | Membuka halaman Selamat Datang yang terintegrasi di Pusat Kontrol. `--if-needed` dibuka satu kali kecuali kemajuan per pengguna mencatat bahwa Selamat Datang telah ditampilkan, diselesaikan, atau ditutup. |
| `argvus taskbar-calendar [args]` | Aplikasi GTK 4 | Defaultnya adalah `toggle`. |
| `argvus removable-devices [args]` | Aplikasi GTK 4 | Defaultnya adalah `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Aplikasi GTK 4 | Munculan Fokus ARGVUS. Satu contoh per sesi. |
| `argvus focus status --waybar [--follow]` | Dalam proses | Memancarkan indikator bilah tugas Fokus sebagai Waybar JSON; `--follow` mengalirkan perubahan tanpa memuat GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Aplikasi GTK 4 | Munculan Jam ARGVUS. Satu contoh per sesi; `--background` menjaga alarm tetap aktif tanpa popup. |
| `argvus loading-theme ...` | Dikemas `splash` di `/usr/lib/argvus/loading-theme/splash` | Hamparan isi ulang sesi langsung; saklar tema tidak menggunakannya. |
| `argvus session`, `start`, `tty`, `sessionctl` | Eksekusi pribadi | Titik masuk sesi dan pengontrol sesi. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Eksekusi pribadi | Titik masuk yang lebih ramah. |
| `argvus accounts`, `firewall` | Eksekusi pribadi | Akun dan alat firewall. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Eksekusi pribadi | Alat peluncur dan pemetik. |
| `argvus tui-terminal` | Dalam proses | Pembungkus terminal untuk aplikasi TUI. |
| `argvus sensors` | Dalam proses | Ikhtisar suhu langsung dari `sensors -j`. |
| `app-profiles-yazi-keymap` | Eksekusi pribadi | Pembantu peta kunci. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Eksekusi pribadi | Pemberitahuan, tampilan, dan tindakan bilah tugas. |
| `argvus setup [options]` | Dalam proses | Menyalin paket default ke dalam konfigurasi pengguna. Lihat di bawah. |

Argumen diteruskan tanpa perubahan, termasuk spasi dan nilai kosong. Input standar, output, kesalahan, sinyal, dan status keluar adalah milik perintah yang dijalankan. Nama yang tidak dikenal gagal dengan petunjuk untuk menjalankan `argvus --help`. Perintah yang berjalan dalam proses menolak argumen yang tidak valid UTF-8 (status keluar 2) alih-alih meneruskan nilai yang diubah; perintah yang menjalankan program terpisah menerima byte tidak berubah.

## Bendera dan alias

Nama-nama ini ada agar pengikatan tombol, entri desktop, dan skrip lama tetap berfungsi. Lebih suka formulir subperintah dalam kode baru.

| Bendera atau alias | Setara |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (dengan halaman `about` dan `apps`) |
| `--displays`, `--monitors` halaman | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | pengelola file (`spf`) |
| `--yazi`, `--yazy` | peramban file (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | memulai `tty` (tidak ada argumen yang diteruskan) |
| `--version`, `-V`, `version` | mencetak `ARGVUS <version>` |
| `help`, `--help`, `-h` | mencetak penggunaan || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Pusat Kontrol menerima tanda halaman lama sebagai argumen pertamanya dan memetakannya ke nama halaman. Contoh: `--apps` menjadi `apps`, `--keyboard-layout` menjadi `locale`, dan `--hostname` menjadi `system`.

## Perintah skrip Hyprland

Ini menjalankan skrip yang dikemas dan digunakan oleh konfigurasi Hyprland:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Skrip dalam konfigurasi pengguna akan menggantikan skrip yang dipaketkan. Urutan pencariannya sama dengan `_config_path` di `hyprland.lua`: salinan konfigurasi pengguna, lalu salinan data ARGVUS, lalu salinan status, lalu salinan yang dihasilkan, dan baru kemudian file paket di bawah `/usr/share/argvus`.

## Contoh

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## Pengaturan

`argvus setup` menyalin paket default dari `/usr/share/argvus/<app>` ke `~/.config/argvus/data/<app>`, sehingga Anda dapat menyesuaikannya. Itu tidak pernah mengedit file yang dikemas.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

Tujuan yang sudah ada dilaporkan dan dibiarkan saja, kecuali `--force` diberikan. Dengan `--force`, direktori yang ada diubah namanya menjadi `<name>.bak-<timestamp>` sebelum penyalinan.

Aplikasi `argvus` istimewa: ia menyalin konten direktori paketnya ke root konfigurasi. Karena root konfigurasi sudah ada saat pengaturan dijalankan, `--copy argvus` memerlukan `--force` dalam kode saat ini.

Penyiapan memvalidasi setiap opsi sebelum mengubah apa pun. Nama aplikasi yang tidak valid (kosong, dimulai dengan `.`, atau berisi `/`) ditolak.

## Sesi dan muat ulang

`SUPER + SHIFT + R` menjalankan `argvus sessionctl reload`. Pengontrol sesi menerapkan konfigurasi yang dihasilkan dan hanya memulai ulang komponen yang dimilikinya. Itu tidak menyalin `/usr/share/argvus` ke direktori home, dan tidak menggantikan file JSON yang salah format.

## Cahaya Malam

Buka **Desktop & Input > Tampilan > Lampu Malam** atau jalankan `argvus control-center displays night-light`. Konfigurasikan suhu, aktivasi manual, dan jadwal lokal tetap melalui `argvus config display night-light`. Lihat [Night Light](/docs/user-guide/night-light/) untuk perintah, status, dan persyaratan waktu proses.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` menjalankan perintah interaktif di dalam bingkai terminal ARGVUS bersama. Ctrl+E menutup bingkai; peneleponnya menyediakan footer yang dilokalkan. Gunakan ini di dalam terminal, atau sebagai perintah yang diteruskan ke `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` menggunakan konfigurasi Kitty khusus dengan tab tersembunyi dan hanya pintasan clipboard. Penelepon memberikan perintah dan gelar; penempatan jendela ditangani oleh aturan Hyprland untuk kelas `argvus-ai`.
