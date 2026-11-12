---
title: Konfigurasi ARGVUS
description: Kontrak konfigurasi JSON logis yang digunakan oleh ARGVUS.
---

ARGVUS menyimpan preferensi pengguna terkelola dalam file JSON modular di bawah `$XDG_CONFIG_HOME/argvus/config/` (biasanya `~/.config/argvus/config/`). Mereka membentuk model logis kanonik untuk preferensi yang dikelola oleh ARGVUS, dan `argvus-config` adalah satu-satunya komponen yang menulis file di bawah `data/generated/`. File asli dan file yang dihasilkan tetap merupakan proyeksi kompatibilitas atau penggantian khusus aplikasi.

## Panduan pengguna: Mulai cepat

Bagi sebagian besar pengguna, konfigurasi dilakukan melalui Pusat Kontrol ARGVUS atau alat baris perintah ini:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```

Untuk administrator dan konfigurasi lanjutan, lanjutkan membaca di bawah.

Akar pengguna hanya berisi dokumen kanonis dan data terkelola:

```text
~/.config/argvus/
├── config.json
└── data/
```

File ditulis secara atom, dijaga kerahasiaannya bagi pengguna, dan dilindungi oleh `.config.lock`, di luar pohon `data/` yang dapat ditukar selama proyeksi. Versi sebelumnya disimpan di `data/backups/config.json.bak`.

## Baris perintah

Subperintah `argvus config` disediakan dalam proses oleh paket `argvus`:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```

`ensure` mewujudkan `config.json` baru dengan skema default, atau terisi
bidang yang hilang tanpa mengganti nilai eksplisit. Gagal pada JSON yang salah format.
`migrate` mengimpor status tampilan lama yang ada tanpa mengganti nilai
sudah ada di `config.json`. `argvus sessionctl` menjalankan `recover`, `ensure`,
`migrate` dan `project` sebelum menerapkan pemuatan ulang setelah perubahan konfigurasi:

```sh
argvus sessionctl apply-config
```

`apply` dan `rebuild` adalah alias untuk membangun kembali setiap proyeksi dari
`config.json`; `project` adalah ejaan historis dan menerima `--force`.
`apply` juga mendelegasikan pemuatan ulang runtime pasca-komitmen ke `argvus sessionctl`.
Atur `ARGVUS_NO_RUNTIME=1` untuk persiapan atau pengujian terisolasi.
Generasi terjadi dalam pementasan dan pohon `data/` hanya ditukar
proyektor berhasil. Oleh karena itu hapus `data/generated/` dan jalankan
`argvus config apply` adalah pembangunan kembali yang dapat direproduksi tanpa menerbitkan sebagian
urutan keluaran Hyprland atau Taskbar. Catatan perintah diturunkan
bagian hash dan keputusan runtime di
`$XDG_STATE_HOME/argvus/config-projection.json` (biasanya
`~/.local/state/argvus/config-projection.json`). Baik manifes maupun
file yang dihasilkan adalah sumber kebenaran. `apply` melakukan pemuatan ulang pasca-komitmen
melalui `argvus sessionctl`; `project` tetap hanya proyeksi untuk kompatibilitas.

Setiap mutasi persisten menggunakan aturan yang sama: jika dokumen kanonik diubah,
CLI memanggil satu `argvus sessionctl reload` terpusat setelah komit. Itu
hasil reload dicatat di `data/internal/last-reload.json`.

Oleh karena itu, `argvus sessionctl reload` adalah langkah *terapkan*, bukan langkah proyeksi.
Ketika manifes yang masuk melaporkan pekerjaan yang tertunda, manifes tersebut akan memulihkan targetnya
konsumen dijelaskan oleh rencana dan tidak menelepon `argvus config project` lagi;
proyeksi ulang akan membangun kembali pohon yang identik dan membuang rencana mutasi
baru saja diproduksi. Ini kembali ke migrasi dan proyeksi hanya jika tidak ada komitmen
rencana ada, misalnya saat sesi dimulai.

### Muat ulang target

`argvus config session-plan` menjawab reload dengan target tersebut
bagian yang berubah, dan `argvus sessionctl` hanya menjalankan bagian tersebut. Percikan
mencakup pemuatan ulang hanya ketika target membuat ulang permukaan: bilah Waybar atau
Panel Kontrol yang dipindahkan kembali ke layar akan menggeser setiap jendela. Segala sesuatu yang lain
memuat ulang di tempatnya.

| Ubah | Sasaran | Percikan |
| --- | --- | --- |
| Tema, warna sorotan, atau mode GTK | `hyprland`, `consumers`, `wallpaper`, `polkit` | tidak |
| Wallpaper saja | `wallpaper` (hyprpaper IPC; layanan dimulai ulang hanya untuk swaybg) | tidak |
| `effects` (blur, animasi, transparansi, bayangan) | `hyprland`, `consumers` | tidak || `layout` jendela (celah, batas) | `hyprland` | tidak |
| `layout.taskbar` (posisi, margin) | `hyprland`, `taskbar`, `widget_telemetry` | ya |
| `fonts` | `consumers` | tidak |
| `control_panel` (saklar, kartu) | `control_panel` | ya |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | tidak |
| `power` | `hypridle` | tidak |

`consumers` menjalankan `argvus config theme-runtime consumers` (bar stylesheet,
Panel Kontrol melalui IPC, warna Hyprland, pemuatan ulang tema Dunst, btop dan
kucing). Dunst menunda pemuatan ulang temanya saat Jangan Ganggu aktif.
`wallpaper` menjalankan `argvus config theme-runtime wallpaper`, yang menunjukkan
`/appearance/wallpaper` yang efektif. Hash `layout.taskbar` disimpan
terlepas dari tata letak jendela dalam manifes proyeksi, sehingga terjadi perubahan celah
tidak me-restart bar. Paket cadangan `full`, digunakan ketika tidak ada rencana yang berkomitmen
ada, mulai ulang setiap permukaan yang terkena percikan.

Setiap konfigurasi efektif memiliki generasi operasional. Itu disimpan di
`data/internal/generation.json` dan dalam manifes proyeksi di bawah
`$XDG_STATE_HOME/argvus/config-projection.json`; itu bukan bagian dari dotfiles
konfigurasi. Menerapkan kembali konfigurasi efektif yang tidak berubah akan mempertahankan
generasi. `status` melaporkan apakah hash manifes, metadata generasi, dan
semua file yang dihasilkan setuju; `apply` memperbaiki proyeksi yang basi atau tidak lengkap.

`explain /path` membaca penyelesai secara langsung dan mencetak nilai efektif,
sumber, tema dan varian. Itu tidak mengurai file yang dihasilkan atau membuat file kedua
aturan yang diutamakan.

Proyeksi tema dan efek diimplementasikan di `argvus-config` itu sendiri.
Transaksi tema (`argvus theme-switch`, yang menjalankan `theme-apply`), `argvus config accent` dan `argvus effects-toggle` tidak memproyeksikan:
mereka memvalidasi, mengunci, melakukan melalui CLI ini dan kemudian
merekonsiliasi konsumen eksternal yang tidak dapat membaca `config.json`. `argvus effects-toggle`
adalah delegasi kurus yang tidak mempunyai keluaran sendiri. Penampilan tidak pernah membantu
dipanggil oleh proyektor, dan mereka tidak boleh menulis di bawah `data/generated/`.

### Permukaan proyeksi

`project_effective` merender setiap konsumen milik ARGVUS dalam satu pohon pementasan
sebelum dipublikasikan:

| Permukaan | Keluaran |
| --- | --- |
| Hyprland | `generated/hypr/`, termasuk profil tema, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` dan `monitors.lua` |
| Tata letak/efek | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Bilah Jalan | `generated/waybar/`, termasuk `argvus-taskbar.css` dan `argvus-widget-telemetry.{jsonc,css}` |
| Tema telemetri widget | `generated/widget-telemetry/` dan `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Pemberitahuan | `generated/notifications/` dan `dunstrc` |
| Layar kunci | `generated/hypr/hyprlock.conf` dan profil tema Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| File Super | `generated/superfile/` |
| Profil terminal dan Qt | `generated/terminal/`, `generated/qt6ct/` |
| Font | `generated/fonts.conf` ditambah blok font terkelola di lembar gaya Waybar |
| Perangkat yang dapat dilepas | `generated/removable-devices/theme.css` |
| kertas dinding | `generated/hypr/hyprpaper.conf` dan `data/hypr/hyprpaper.conf` |
| Panel Kontrol dan default | `data/control-panel/`, `data/control-center/defaults.json` |
| Kalender, masukan, keyboard, daya | TOML kalender, fragmen masukan, status keyboard, penanda tetap terjaga |

Lapisan tema juga dicerminkan di luar root yang dihasilkan:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` dan
`data/hypr/application-style.conf`. Salinan tersebut diganti dari kemasannya
default bila bagian tampilan berubah, dan proyektor kemudian menulis ulang
hanya blok terkelola yang dibatasi — blok font di lembar gaya Waybar dan
blok `ARGVUS_TELEMETRY_*` di profil telemetri — jadi teks di luar
penanda tidak dapat bertahan jika terjadi pergantian tema. `fonts.targets` dan
`control_panel.widget_telemetry_blocks` tetap kanonik: Pusat Kontrolmenyimpannya melalui `argvus-config` dan tidak pernah menulis file ini sendiri.

Pemilihan blok telemetri widget bersifat kanonik di
`/control_panel/widget_telemetry_blocks`, array dari tujuh nama blok
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). Itu
file `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` lama hanya dibaca sebagai a
fallback migrasi dan tidak pernah ditulis. `argvus widget-telemetri-toggle blok
set` and `blocks all` menambal array kanonik dalam satu penulisan atom, dan
`apply-state` mengasumsikan pemilihan sudah dilakukan oleh pemanggilnya.

Jarak Hyprland dan preferensi perbatasan mengikuti batas yang sama. Itu
argvus config space dan perintah argvus config border (dan historinya
pembungkus kompatibilitas) perbarui/tata letak; Karat dihasilkan
data/generated/spaces-efektif.conf dan data/generated/borders-efektif.conf
dan memiliki validasi dan aplikasi hyprctl langsung. .spaces dan .borders adalah
negara yang hanya melakukan migrasi dan tidak pernah menjadi sumber kebenaran.

Status mode Aksen dan GTK menggunakan batas Rust yang sama. Gunakan `argvus-config
aksen COLOR` or `argvus config aksen --theme-default` untuk kepemilikan aksen,
dan `argvus config mode toggle|set dark|light` untuk mode GTK. Yang bersejarah
skrip aksen dan mode adalah pembungkus kompatibilitas; `.accent-color`,
`.accent-custom` dan `.gtk-mode` hanya merupakan file migrasi/proyeksi.

Jalur ARGVUS logis juga diselesaikan oleh `argvus config paths`. Warisan
File `paths.sh` hanya memperlihatkan fungsi kompatibilitas; penggantian asli, pengguna
data, data yang dihasilkan, dan fallback sistem diselesaikan dengan model jalur Rust.

`audio` mengubah wastafel keluaran default dan mencatat hasilnya di `/audio` dalam satu langkah: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` dan `argvus config audio mute <on|off|toggle>`. Ia menulis penyimpanan tanpa memuat ulang konfigurasi, karena PipeWire sudah memiliki nilainya. Modul volume Waybar, tombol media Hyprland, dan Panel Kontrol menggunakannya, jadi `argvus config project`, yang memproyeksikan `/audio` ke wastafel, tidak pernah memulihkan volume lama. Volume di atas 100% (tombol Hyprland mengizinkan hingga 150%) disimpan sebagai 100.

`patch` menerapkan objek JSON yang kuncinya merupakan penunjuk JSON dalam satu objek terkunci
operasi baca-modifikasi-tulis. `apply-theme` melakukan atom yang setara
transaksi tema dan menggantikan bagian tampilan terkelola dari
profil tampilan opsional tema atau paket default.
Perintah mutasi membangun `NextUserConfig`, menyelesaikan `EffectiveConfig`, tahap
semua proyektor, dan baru kemudian mempublikasikan JSON kanonik. Sebuah operasional
Penanda `.config-transaction.json` memungkinkan `argvus config recover` memproyeksikan ulang
konfigurasi sebelumnya setelah terjadi crash antara data dan publikasi JSON.
Default tema dideklarasikan oleh setiap manifes tema
(`themes/<theme>/theme.toml`): aksen default (`palette.accent`), skema warna GTK
dan kertas dinding. File CSS, Lua, Rasi dan Qt dirender darinya ke dalam pembuatan tema.

Mengubah tema, termasuk Terang/Gelap, akan menyetel ulang warna sorotan ke default tema,
bahkan ketika pengguna memilih warna khusus: warna khusus hanya disimpan hingga tema berikutnya
perubahan. Gunakan `argvus config accent` setelah perubahan tema untuk mengaturnya kembali. Menerapkan a
tema dengan `argvus config theme-apply` menulis `/appearance/accent` dan `/appearance/gtk_mode`
dari manifes, dan `argvus config accent` menulis warna yang dipilih dengan `accent_custom`
atur, dalam satu perintah.

`apply-theme` mengambil varian secara eksplisit sebagai `--variant sticky|spaced` dan
menyelesaikan nama yang diminta melalui manifes, jadi permintaan tidak akan pernah bisa
memutuskan aset varian lainnya. `theme-manifest THEME` juga sama
resolusi terbuka hanya-baca: ia mencetak entri manifes bersama dengan
`asset_id` dan `layout_variant` diselesaikan, itulah yang perlu dilakukan penelepon
alamat aset yang dikemas. Sticky adalah id tanpa akhiran dan hanya Spasi yang membawa aAkhiran `-spaced`, jadi `dracula-sticky` dan `argvus-dark-dracula-sticky` adalah
diterima dan dinormalisasi ke `dracula`.

Dengan `--raw`, nilai string dicetak tanpa tanda kutip JSON, sedangkan objek dan array tetap berupa nilai JSON. File dot tema dan tata letak tidak lagi ditulis oleh pembantu proyeksi; `config.json` tetap menjadi satu-satunya sumber tetap untuk preferensi tersebut. Jika proyeksi gagal, pemuatan ulang sesi akan berhenti sebelum memulai ulang permukaan desktop sehingga sebagian tema lama tidak disajikan sebagai tema terkini.

## Ekspor dan impor

Model logis yang sama digunakan untuk pertukaran profil:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```

Gunakan `--scope desktop` untuk model desktop terkelola lengkap. Bidang yang hilang menggunakan default ARGVUS; keluaran yang dihasilkan tidak boleh diedit secara langsung.

Ekspor dan impor tema Pusat Kontrol menukar tema dan profil visualnya: arsip berisi direktori tema skema 2 (`<id>/theme.toml`, opsional `appearance.json` dan `effects.conf`, `overrides/`, wallpaper) yang dipasang di bawah `$XDG_DATA_HOME/argvus/themes/`. Profil tersebut berisi tampilan, geometri tata letak, efek visual, font, dan presentasi panel. Audio, keyboard, jaringan, dan preferensi non-visual lainnya tetap berada di luar tema. Gunakan cakupan `appearance` di atas untuk memindahkan konfigurasi tampilan lengkap, dan cakupan `desktop` untuk cadangan konfigurasi lengkap.

Skema saat ini mengelompokkan preferensi di bawah `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar`, dan `removable_devices`. `default_apps` adalah satu-satunya sumber persisten untuk aplikasi pilihan, dan `keyboard_shortcuts` menyimpan kunci bahasa Inggris yang stabil dan tidak bergantung pada lokal dengan nilai string atau `null` untuk pengikatan yang dinonaktifkan.

Pusat Kontrol membaca nilai tampilan kanonik saat membuka halaman Penampilan dan menulis perubahan melalui `argvus-config`. File keadaan asli yang ada tetap merupakan proyeksi untuk skrip tampilan dan instalasi lama.

Panel Kontrol menggunakan `control_panel.cards` untuk urutan dan visibilitas kartu, dan `control_panel.enabled` untuk sakelar master panel. File `control-panel/cards.json` dan `state/control-panel` disimpan sebagai proyeksi kompatibilitas. Oleh karena itu, perubahan yang dilakukan di Panel Kontrol merupakan bagian dari cakupan ekspor `desktop`.

Preferensi font menggunakan `fonts.targets` untuk kelompok, gaya, dan ukuran bilah tugas, telemetri, Panel Kontrol, sistem, aplikasi, terminal, dan target browser. `fonts.rendering` menyimpan antialiasing, petunjuk, urutan subpiksel, dan preferensi DPI. `fonts.conf`, aturan fontconfig, pengaturan desktop, dan blok font terkelola di dalam stylesheet Waybar adalah proyeksi: Pusat Kontrol tetap mempertahankan `/fonts/targets/*` hingga `argvus-config`, dan proyektor hanya menulis ulang blok yang dibatasi.

Preferensi Hyprland menggunakan `keyboard_shortcuts` untuk pintasan, `hyprland.input` untuk pengaturan mouse dan touchpad, dan `hyprland.keyboard` untuk tata letak, varian, dan opsi XKB. ID pengikatan manifes dipetakan ke `config_key` yang stabil (misalnya `window.close` → `close_window` dan `window.drag_mouse` → `drag_window__floating_window_only`). Preferensi tampilan menggunakan `displays.monitors`, `displays.workspaces`, dan `displays.primary_monitor`. File TOML asli dan file Lua yang dihasilkan di bawah `data/generated/` tetap menjadi proyeksi yang digunakan oleh Hyprland.Setiap sakelar `effects` secara default diaktifkan (animasi, blur, telemetri widget, dan setiap permukaan transparansi), pada mesin virtual dan sesi berdaya rendah juga; bayangan jendela (`shadow_enabled`) dan goyangan (`wobble_enabled`) default mati. Preferensi efek menggunakan `effects` untuk saklar master Animasi (`animations`, diterapkan pada `hyprland.lua` dari `hypr/state.lua` yang diproyeksikan, tidak bergantung pada saklar Efek, yaitu `blur_global_enabled` ditambah `transparency_<surface>_enabled` dari `taskbar`, `control-panel`, `widget-telemetry`, `terminal` dan `launchers`, dan dibaca sebagai diaktifkan ketika salah satunya diaktifkan; `argvus effects-toggle animations|effects` menggantinya), saklar blur global (`blur_global_enabled`), parameter blur Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), bayangan jendela (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), nilai Animasi (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` dan `_enabled` untuk setiap animasi, ditambah `animation_bezier_x1` hingga `animation_bezier_y2` dan `animation_spring_mass`, `animation_spring_stiffness` dan `animation_spring_dampening` untuk kustom kurva) dan nilai transparansi per permukaan. Efek adalah milik tema, bukan milik varian: Sticky dan Spaced menggunakan proyeksi efek `<theme>.conf` yang sama, sedangkan proyeksi tata letaknya berbeda. Permukaan saat ini mencakup `taskbar`, `control-panel`, dan `widget-telemetry`; nilai transparansi adalah persentase dari `0` hingga `100` (Pusat Kontrol dan permukaan peluncur secara default adalah `15`, yang lainnya adalah `50`), parameter blur adalah angka dalam rentang yang dinyatakan dalam `schema.json`, dan status yang diaktifkan adalah boolean. Nilai animasi juga divalidasi oleh `argvus-config`: kecepatan adalah angka dari `1` hingga `10` (desimal diperbolehkan), persentase pop-in adalah bilangan bulat dari `1` hingga `100`, kurva harus berupa nama dari Animasi halaman, gaya harus berupa nama dari halaman Animasi dan diterima oleh daun animasi itu sendiri (misalnya `popin` ditolak untuk `animation_workspace_in_style`, `gnomed` diterima hanya untuk membuka dan menutup jendela, dan varian slide ditolak pada jendela dan lapisan keluar), titik Bézier Y tetap berada di antara `-1` dan `2`, rentang Hyprland 0.56.2 diurai, dan parameter kurva lainnya tetap berada dalam rentang yang didokumentasikan. Daun `layers` dan `switching` sudah tidak ada lagi, jadi konfigurasi yang disimpan sebelum penghapusannya akan kehilangan kunci `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve`, dan `animation_switching_*` saat dimuat. Gaya di luar kumpulan saat ini dihilangkan dan titik Bézier Y di luar `-1` hingga `2` dijepit, sehingga paket default berlaku lagi untuk apa pun yang telah dihapus.

Preferensi daya menggunakan `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake`, dan kebijakan penutup/tombol daya. Bentuk yang diproyeksikan adalah `data/generated/hypr/hypridle.conf` dan penanda `data/power/keep-awake`; `.keep-awake` hanya merupakan masukan migrasi. `session.language` dihentikan: nilai yang ada diabaikan, penulisan baru ditolak, dan tidak ada file kompatibilitas bahasa yang diproyeksikan. Tetapkan `LANG` di pengguna `locale.conf` sebagai gantinya. Preferensi kalender menggunakan `calendar` dan diproyeksikan ke TOML pengguna kalender taskbar. Preferensi perangkat yang dapat dilepas menggunakan `removable_devices`; aplikasi perangkat yang dapat dilepas memperlakukan bagian ini sebagai penggantian pengguna atas paket default.`migrate` mengimpor file dan bagian root lama satu kali, mempertahankan nilai kanonik yang ada. `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, file dot, `generated/` dan direktori komponen dipindahkan ke bawah `data/`; memuat ulang berulang kali tidak membuat ulang tata letak root yang lama. Memilih kembali tema saat ini adalah idempoten: jika keadaan kanonik dan proyeksi sudah sesuai, rencana proyeksi melaporkan tidak ada bagian yang diubah dan sesi melompati splash dan layanan dimulai ulang.

Daya mengikuti kontrak proyeksi yang sama: Pusat Kontrol menulis `/power` di `config.json`, `argvus config project` merender `data/hypr/hypridle.conf` secara atom, dan hanya dengan demikian `argvus sessionctl` dapat memulai ulang `argvus-hypridle.service`. Menghapus file itu dapat dipulihkan dengan `argvus config project`; nilai `0` menghilangkan batas waktu yang sesuai dan berarti Tidak Pernah.

Tampilan layar kunci diproyeksikan oleh Rust dari tema kanonik, aksen,
efek, wallpaper dan status font menjadi `data/hypr/hyprlock.conf`. Yang bersejarah
Titik masuk `hyprlock-theme.sh` hanya untuk kompatibilitas dan didelegasikan ke sana
`argvus config lock apply`; `--invalidate` hanya membatalkan keburaman turunan
cache wallpaper.

Perubahan batas waktu daya menggunakan `/power/lock_minutes` dan
`/power/screen_off_minutes` sebagai sumber kebenarannya. `argvus config project`
menggunakan kembali penyaji Hypridle yang ada, dan titik masuk `idle-timeout.sh` adalah
hanya adaptor kompatibilitas; restart layanan tetap didelegasikan ke
`argvus sessionctl`.

Tampilan notifikasi berasal dari tema kanonik, aksen dan
efek menjadi `data/notifications/dunstrc`. Pemberitahuan sejarah
File `theme.sh` hanya memperlihatkan fungsi kompatibilitas yang dipanggil
`argvus config notifications`; itu tidak mengurai atau mengubah konfigurasi Dunst.

Adaptor berorientasi perangkat keras seperti penemuan tampilan dan kontrol Bluetooth
sekarang diimpor oleh domain tampilan Rust: `nwg-displays` adalah importir eksternal,
sementara `/displays` dan `data/generated/hypr/monitors.lua` tetap kanonik dan
keadaan turunan masing-masing. Alat tampilan mengubah tata letak dengan
`argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` dan `monitor-remove`; perintah tampilannya adalah `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. File Hyprland diproyeksikan dari `/displays`; tidak ada skrip shell yang menulisnya lagi. Pemilihan wallpaper, mode grup utilitas bilah tugas,
dan pencarian aplikasi default juga menggunakan API `argvus-config`; Shell bersejarah mereka
titik masuk adalah pembungkus UI atau kompatibilitas. Keep Awake adalah sebuah persisten
Preferensi `/power/keep_awake` diproyeksikan ke penanda `data/power/keep-awake`,
sementara lock-DPMS adalah status runtime sesi di bawah
`$XDG_CACHE_HOME/argvus/lock-dpms`. Tata letak keyboard kini dimiliki oleh Rust
`argvus-keyboard-layout` helper: pesanan yang dikonfigurasi berasal dari yang dihasilkan
Proyeksi masukan Hyprland atau penggantian lokal, sementara tata letak aktif tetap ada
status waktu proses sesi ditemukan dari Hyprland. Titik masuk siklus dan
daemon tata letak keyboard hanya merupakan pembungkus kompatibilitas; layanan pengguna berjalan
daemon Rust secara langsung dan `argvus sessionctl keyboard-layout cycle` adalah
perintah stabil untuk penelepon.

Status waktu proses Bluetooth mengikuti batas adaptor eksternal tetapi bukan merupakan bagian
dari `argvus-config`: `argvus bluetooth` memiliki penguraian yang diketik dan
validasi sekitar `bluetoothctl`, percobaan ulang terbatas, pemeriksaan layanan sistem dan
Status RFKill. `argvus bluetoothctl` adalah titik masuknya. Daya adaptor, perangkat berpasangan, dan perangkat yang terhubung
tetap menggunakan status runtime BlueZ, bukan status JSON kanonik; perangkat keras-hidup
Validasi Bluetooth bergantung pada lingkungan.
## Identitas dan rekonsiliasi runtime

Gunakan `argvus config status --verbose` untuk memeriksa biner yang tepat, build
identitas, root konfigurasi, root yang dihasilkan, root aset, sumber manifes, dan
generasi aktif. Ini melaporkan `synchronized`, `stale`, `incomplete`, `legacy`
dan status proyeksi `pending-transaction`.

`argvus config --version` melaporkan versi paket, revisi versi saat disediakanoleh lingkungan build, dan profil. Tes pengembangan harus menyuntikkannya
biner dan aset secara eksplisit; mereka tidak boleh menggunakan `argvus config` secara diam-diam
atau `~/.config/argvus`. `apply` dengan aman menormalkan pengidentifikasi gabungan lama
seperti `argvus-dark-spaced` menjadi nilai tema dan varian yang terpisah sebelumnya
membangun kembali proyeksi. Identitas aset yang dikemas tidak dicatat dalam manifes dalam rilis ini.

## Cahaya Malam

Buka **Desktop & Input > Tampilan > Lampu Malam** atau jalankan `argvus control-center displays night-light`. Konfigurasikan suhu, aktivasi manual, dan jadwal lokal tetap melalui `argvus config display night-light`. Lihat [Night Light](/docs/user-guide/night-light/) untuk perintah, status, dan persyaratan waktu proses.

### Pengaturan goyangan (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`,
`wobble_intensity` dan `wobble_mesh` memproyeksikan langsung ke `hypr/state.lua`.
Gunakan `argvus effects-toggle wobble-settings get` untuk membacanya, atau
`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`
untuk menyimpan dan meminta memuat ulang. `enabled=disabled` mematikan efeknya.

Pusat Kontrol juga membagikan draf Animasi: `animations-settings get`
mengembalikan entri `wobble_*`, dan `animations-settings set` menerima nama lengkap ini
di samping argumen animasi yang ada. Ini mempertahankan keseluruhan draf menjadi satu
tambalan yang divalidasi dan meminta satu muat ulang. Misalnya:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```

`enabled` global mengontrol animasi; `wobble_enabled` bersifat independen
preferensi. Menonaktifkan animasi global akan menekan goyangan di Lua tanpa menghapusnya
preferensi itu. Nilai yang tidak valid menolak keseluruhan patch. Kunci yang hilang diterima
default ARGVUS tanpa mengganti pilihan yang ada. Lihat [Penampilan](/docs/user-guide/appearance/#window-wobble)
untuk batasan dan default.


### Pintasan keyboard yang dibuat pengguna

`/keyboard_shortcuts` mempertahankan nilai string-atau-null yang ada untuk tindakan manifes resmi. Pintasan keyboard global khusus menggunakan objek `/hyprland/custom_shortcuts` terpisah, disimpan di `config/hyprland.json`. ID `custom_` yang stabil tidak bergantung pada deskripsi dan lokal. Setiap catatan memiliki `description`, `keys`, `enabled` dan `action` yang diketik:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```

Contohnya adalah nilai `/hyprland/custom_shortcuts`, bukan dokumen konfigurasi lengkap. Nama operator berasal dari pabrik operator Lua yang didukung; argumen adalah objek JSON opsional atau `null`, kode tidak pernah dievaluasi. Pintasan khusus tidak memiliki subpeta, pengikatan mouse, atau tanda lanjutan. Proyeksi menulis `data/generated/hypr/custom-shortcuts.lua` sebagai tabel data yang lolos, termasuk tabel kosong ketika catatan terakhir dihapus. Proyeksi penuh dan `project --only keyboard_shortcuts` menghasilkannya. Hyprland mendaftarkan entri yang diaktifkan dengan `hl.bind`; file opsional dan kegagalan pendaftaran individu terkandung dan dicatat.

Pusat Kontrol membuat/mengedit catatan dan menonaktifkan konflik yang dikonfirmasi menggunakan satu transaksi `argvus config patch`. Ini menghemat sebelum mendaftar melalui layanan konfigurasi yang ada. Penerapan yang gagal tidak membatalkan penyimpanan yang berhasil. Memulihkan default resmi akan mempertahankan catatan khusus dan ditolak jika akan bertabrakan dengan pintasan khusus yang aktif. Deskripsi pengguna tetap berupa teks literal di semua lokal dan lembar contekan yang dihasilkan secara efektif.

## Mode jendela global

Buka **Pusat Kontrol → Desktop & Input → Hyprland → Windows → Mode** dan pilih **Tile Windows (Default)** atau **Floating Windows**. Pemilih yang sama tersedia dari ikon mode jendela di grup utilitas bilah tugas.

Preferensi berlaku untuk setiap monitor dan ruang kerja dan diingat setelah logout. Mengambang mengubah jendela aplikasi yang ada dan jendela yang baru dibuka. Tile hanya memulihkan jendela yang diubah kebijakannya; dialog dan jendela lain yang sudah mengambang tetap mengambang. Sebuah jendela yang secara eksplisit disematkan ketika Floating aktif juga tetap mengambang untuk mempertahankan pinnya. Tata letak ubin dihitung ulang; pohon terbelah sebelumnya tidak dipulihkan.`SUPER + SHIFT + Space` mempertahankan perilaku normal per jendela di Tile. Dalam Mengambang, jendela tidak dikembalikan ke ubin. Panel, bilah tugas, dan notifikasi merupakan permukaan lapisan cangkang dan tidak terpengaruh. Layar penuh dan maksimalisasi terus menggunakan perilaku normal Hyprland.

Preferensi kanoniknya adalah `/hyprland/window_mode` di `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Proyeksi yang dihasilkannya adalah `data/generated/hypr/window-mode.lua`; jangan mengeditnya. Registri pemulihan per jendela berada di bawah `$XDG_RUNTIME_DIR/argvus`, tercakup dalam instance kompositor, dan bertahan saat memuat ulang konfigurasi. Itu tidak digunakan kembali di sesi baru.

Jika aplikasi tertunda atau gagal, antarmuka akan membedakannya dari preferensi yang disimpan. Pilih opsi lagi untuk mencoba lagi. Tanpa kompositor, preferensi disimpan untuk login berikutnya.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```

## Tombol jendela dan bilah aplikasi

Mode mengambang menambahkan bilah judul ARGVUS dengan **Minimalkan**, **Maksimalkan / Pulihkan**, dan **Tutup** ke jendela aplikasi normal. Mengklik dua kali bilah judul akan mengaktifkan maksimalisasi. Dekorasi tambahan disembunyikan di Tile dan layar penuh. Aplikasi yang menggambar kontrolnya sendiri dapat menampilkan kedua bilah; kontrol ARGVUS yang seragam memang disengaja.

Waybar kedua muncul di setiap monitor, di tepi seberang taskbar utama. Ini hanya mencantumkan jendela milik ruang kerja monitor saat ini, termasuk jendela yang diperkecil dari ruang kerja tersebut. Ruang kerja khusus yang terbuka diutamakan. Jendela yang disematkan tetap terlihat; tab grup tersembunyi tidak membuat entri tambahan, dan grup yang diperkecil memiliki satu entri pemulihan. Tab di dalam aplikasi tidak dihitung secara terpisah. Entri yang diperkecil mempertahankan asosiasi monitor/ruang kerja aslinya. Klik jendela yang tidak aktif untuk mengaktifkannya, jendela aktif untuk memperkecilnya, atau jendela yang diperkecil untuk memulihkannya; menu konteksnya menawarkan pemulihan, perkecil, maksimalkan/pulihkan, dan tutup. Ikon aplikasi yang tidak dikenal kembali ke simbol jendela. Mengaktifkan atau memulihkan entri akan mempertahankan posisi penunjuk. Bilah sekunder menggunakan CSS primer yang efektif (warna, font, batas, dan pembulatan), dengan tinggi dan margin cermin di tepi yang berlawanan.

`SUPER + Alt + Up` menempatkan bilah utama di bagian atas dan bilah jendela di bagian bawah. `SUPER + Alt + Down` melakukan sebaliknya. Mengulangi suatu arah membuat posisinya tidak berubah. Ubin hanya memiliki bilah utama. Kembali ke Ubin akan memulihkan semua jendela yang diperkecil sebelum menghapus bilah jendela, dan kemudian mempertahankan kebijakan ubin/mengambang yang asli.

Minimalkan menggunakan ruang kerja `special:argvus-minimized` yang dicadangkan. Jangan gunakan ruang kerja itu secara manual. Proses `argvus windows service` mencatat identitas, ruang kerja/monitor asli, geometri, layar penuh/maksimalisasi, dan status pin dalam registri runtime khusus instance kompositor di bawah `$XDG_RUNTIME_DIR/argvus`. Catatan bertahan saat layanan dimulai ulang atau memuat ulang konfigurasi, tetapi tidak melintasi sesi kompositor. Menutup jendela akan menghapus catatannya. Jika monitor hilang, pemulihan menggunakan output yang tersedia. Bilah pemulihan yang hilang atau gagal mencegah minimalisasi baru; restorasi yang gagal mempertahankan catatan dan bilah pemulihan.

Sesi ini memiliki `argvus-windows.service` dan `argvus-window-bar.service`. Yang terakhir membuat anak Waybarnya tidak aktif di Tile. Plugin hyprbars dibundel oleh `argvus-hyprland`, dibuat untuk Hyprland 0.57.0 ABI pada x86_64; itu tidak dikompilasi di direktori home pengguna. Bangun kembali plugin dengan pembaruan ABI kompositor.

Perintah: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all`, dan `minimize|restore|activate|maximize|close ADDRESS`. Alamat jendela berasal dari daftar. `position top --persist` menyimpan/memproyeksikan preferensi tanpa memulai ulang bilah utama. Detail kesalahan konfigurasi dan runtime ditulis ke jurnal; kegagalan tindakan yang terlihat juga menampilkan pemberitahuan yang diterjemahkan.Dalam **Spaced + Floating**, jendela mengikuti celah luar dan area yang disediakan oleh kedua batang. Batasnya mencakup batas dan judul, yang berada di dalam batas jendela yang sama. Memaksimalkan melalui kontrol ARGVUS mengisi area ini dengan margin; memulihkan memulihkan geometri sebelumnya, termasuk setelah layanan dimulai ulang. Layar penuh masih menggunakan seluruh keluaran. Jendela mengambang mungkin tumpang tindih; celah bagian dalam ubin tidak memisahkannya.

Margin dalam yang disediakan oleh palang tidak dihitung dua kali pada celah luar jendela. Dengan jarak simetris, jarak jendela-ke-bar yang terlihat adalah sama di atas dan di bawah, termasuk setelah bar ditukar.

`SUPER + Alt + Space` mengaktifkan mode Ubin/Mengambang global, menggunakan preferensi yang sama seperti bilah tugas dan Pusat Kontrol. Perintah yang setara adalah `argvus config window-mode toggle`. Pemanggilan serentak dibuat berseri; beralih menggunakan preferensi yang disimpan bahkan ketika aplikasi tertunda. Pintasan dapat diubah atau dinonaktifkan di Pintasan Keyboard; `SUPER + Shift + Space` mempertahankan tindakan jendela individualnya.
