---
title: Komut satırı
description: argvus komutu, alt komutları, tuş atamaları için tutulan işaretler ve her birinin çalıştığı yer.
---

`/usr/bin/argvus`, ARGVUS'un halka açık giriş noktasıdır. `argvus-cli` kasasından oluşturulmuş bir Rust programıdır. Her alt komut ya o programın içinde çalışır, komutlar dizininden çalıştırılabilir bir yardımcı dosyayı çalıştırır ya da paketlenmiş bir komut dosyasını çalıştırır.

Sürümünüzle birlikte yüklenen liste için `argvus --help` komutunu çalıştırın. Alt komutlar, baştaki `--` olmadan yazılır. `--` formları yalnızca bir tuş bağlama veya masaüstü girişinin bunları kullandığı yerde tutulur.

## Her komutun çalıştığı yer

- **İşlem içi:** komut, çalışma alanının bir kitaplığıdır ve `argvus` içinde çalışır. Ayrı bir yürütülebilir dosya mevcut değildir. Bunlar `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` ve `setup`.
- **Yardımcı:** `/usr/lib/argvus/commands/`'da `argvus` paketi tarafından yüklenen ayrı bir yürütülebilir dosya. Bunlar `control-center` (terminal uygulaması), `argvus-du` (`argvus du` ile piyasaya sürüldü) ve GTK 4 uygulamaları `taskbar-calendar`, `removable-devices`, `focus` ve GTK 4 uygulamalarıdır. `clock`.
- **Özel yürütülebilir dosya:** `/usr/lib/argvus/commands/`'da başka bir paketin yüklediği yürütülebilir dosya. `argvus` yalnızca onu başlatır. Bunlar arasında `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` ve `greeter-*` giriş noktaları.
- **Komut Dosyası:** `/bin/sh` ile çalıştırılan paketlenmiş bir kabuk komut dosyası. Bunlar aşağıda listelenen Hyprland komut dosyası komutlarıdır.

`ARGVUS_COMMANDS_DIR`, testler ve `DESTDIR` kontrolleri için komutlar dizinini değiştirir. Orada bulunmayan özel bir yürütülebilir dosya `PATH`'da aranır.

## Alt komutlar

| Komut | Koşular | Notlar |
| --- | --- | --- |
| `argvus config ...` | Süreçte | Yapılandırma modeli ve araçları. Ayrıca `theme-switch` ve `effects-toggle` tarafından da ulaşıldı. |
| `argvus theme ...` | Süreçte | `apply ID` tema anahtarıdır (`theme-switch` ile aynı). Tema yapıları: `list` yüklü temalar, `check DIRECTORY` yüklü şablonlara karşı bir tema, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Bkz. [Görünüm ve temalar](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | Süreçte | Çeviriler: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | Süreçte | SSH anahtar yöneticisi. `gpg`, GPG alt komutlarının takma adıdır. |
| `argvus gpg ...` | Süreçte | GPG anahtarları: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | Süreçte | Proje seçici. |
| `argvus terminal ...` | Süreçte | ARGVUS terminal profili. |
| `argvus system-monitor ...` | Süreçte | Btop sistem monitörü; `--gpu` yalnızca GPU kutularını seçer ve `--panel [--cpu|--mem|--gpu]` bunu ARGVUS TUI kabuğuna yerleştirir. |
| `argvus du` | Yardımcı (terminal uygulaması) | ARGVUS Disk Kullanım Analizcisini özel bir Kitty penceresinde açar ve ncdu'yu çalıştırır. |
| `argvus network ...` | Süreçte | NetworkManager durumu ve kontrolleri (`networkctl` takma ad olarak tutulur). |
| `argvus bluetooth ...` | Süreçte | Bluetooth durumu ve kontrolleri (`bluetoothctl` takma ad olarak tutulur). |
| `argvus widget-telemetry block ...` | Süreçte | Waybar telemetri bloğu. Bu barın sıcak yoludur. || `argvus widget-telemetry toggle ...` | Süreçte | Telemetri bloklarını ve sıralarını açar, kapatır. |
| `argvus game-snake` | Özel yürütülebilir dosya | Bağımsız `argvus-game-snake` projesinden terminal Snake oyunu, isteğe bağlı bir paket. Bir terminalin dışında bir tane açılır. |
| `argvus control-center [page]` | Yardımcı (terminal uygulaması) | Kontrol Merkezini açar veya zaten açık olan pencereye odaklanır. |
| `argvus welcome [--if-needed]` | Sevk Görevlisi → Kontrol Merkezi | Kontrol Merkezinde entegre Hoş Geldiniz sayfasını açar. `--if-needed`, Hoş Geldiniz'in zaten görüntülendiğini, tamamlandığını veya kapatıldığını kullanıcı başına ilerleme kaydetmediği sürece bir kez açılır. |
| `argvus taskbar-calendar [args]` | GTK 4 uygulaması | Varsayılan olarak `toggle` şeklindedir. |
| `argvus removable-devices [args]` | GTK 4 uygulaması | Varsayılan olarak `menu` şeklindedir. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 uygulaması | ARGVUS Odak açılır penceresi. Oturum başına bir örnek. |
| `argvus focus status --waybar [--follow]` | Süreçte | Focus görev çubuğu göstergesini Waybar JSON olarak yayar; `--follow`, GTK yüklenmeden değişiklikleri aktarıyor. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 uygulaması | ARGVUS Saat açılır penceresi. Oturum başına bir örnek; `--background`, alarmları açılır pencere olmadan aktif tutar. |
| `argvus loading-theme ...` | Paketlenmiş `splash`, `/usr/lib/argvus/loading-theme/splash`'da | Doğrudan oturum yeniden yüklemelerinin yer paylaşımı; bir tema anahtarı onu kullanmaz. |
| `argvus session`, `start`, `tty`, `sessionctl` | Özel yürütülebilir dosyalar | Oturum giriş noktaları ve oturum denetleyicisi. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Özel yürütülebilir dosyalar | Karşılama giriş noktaları. |
| `argvus accounts`, `firewall` | Özel yürütülebilir dosyalar | Hesaplar ve güvenlik duvarı araçları. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Özel yürütülebilir dosyalar | Başlatıcı ve seçici araçlar. |
| `argvus tui-terminal` | Süreçte | TUI uygulamaları için terminal sarıcı. |
| `argvus sensors` | Süreçte | `sensors -j`'dan canlı sıcaklığa genel bakış. |
| `app-profiles-yazi-keymap` | Özel yürütülebilir dosya | Tuş haritası yardımcısı. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Özel yürütülebilir dosyalar | Bildirimler, görüntüleme ve görev çubuğu eylemleri. |
| `argvus setup [options]` | Süreçte | Paketlenmiş varsayılanları kullanıcı yapılandırmasına kopyalar. Aşağıya bakın. |

Bağımsız değişkenler, boşluklar ve boş değerler de dahil olmak üzere değiştirilmeden iletilir. Standart giriş, çıkış, hata, sinyaller ve çıkış durumu çalışan komuta aittir. Bilinmeyen bir ad, `argvus --help`'yu çalıştırmak için bir ipucu vererek başarısız oluyor. İşlem sırasında çalışan bir komut, değiştirilmiş bir değeri iletmek yerine geçerli UTF-8 olmayan bir bağımsız değişkeni (çıkış durumu 2) reddeder; Ayrı bir programı çalıştıran komutlar, baytları değişmeden alır.

## Bayraklar ve takma adlar

Bu adlar, tuş atamalarının, masaüstü girişlerinin ve eski komut dosyalarının çalışmaya devam etmesi için mevcuttur. Yeni kodda alt komut formunu tercih edin.

| Bayrak veya takma ad | Eşdeğer |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (`about` ve `apps` sayfalarıyla) |
| `--displays`, `--monitors` sayfası | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | dosya yöneticisi (`spf`) |
| `--yazi`, `--yazy` | dosya tarayıcısı (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | `tty`'yu başlatır (hiçbir bağımsız değişken iletilmez) |
| `--version`, `-V`, `version` | baskılar `ARGVUS <version>` |
| `help`, `--help`, `-h` | kullanımı yazdırır || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Kontrol Merkezi, eski sayfa bayraklarını ilk argüman olarak kabul eder ve bunları sayfa adlarıyla eşler. Örnekler: `--apps`, `apps` olur, `--keyboard-layout`, `locale` olur ve `--hostname`, `system` olur.

## Hyprland komut dosyası komutları

Bunlar paketlenmiş komut dosyalarını çalıştırır ve Hyprland yapılandırması tarafından kullanılır:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot` layout-mode-menu  weather-location  toggle-sidebar  emoji-picker  `spaces-switch` ,  `emoji-picker`  `toggle-sidebar`  `weather-location`  `layout-mode-menu` `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`, brightness-switch, toggle-mode, idle-timeout, keep-awake, hypr-power-menu.

Kullanıcının yapılandırmasındaki bir komut dosyası, paketlenmiş olanı geçersiz kılar. Arama sırası, hyprland.lua'daki `_config_path` ile aynıdır: kullanıcının yapılandırma kopyası, ardından ARGVUS veri kopyası, ardından durum kopyası, ardından oluşturulan kopya ve ancak bundan sonra  `hyprland.lua` `/usr/share/argvus` altında paketlenmiş dosya.

## Örnekler

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

## Kurulum

`argvus setup`, paketlenmiş varsayılanları `/usr/share/argvus/<app>`'dan `~/.config/argvus/data/<app>`'ya kopyalar, böylece bunları özelleştirebilirsiniz. Paketlenmiş dosyaları asla düzenlemez.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

Zaten var olan bir hedef raporlanır ve `--force` verilmediği sürece yalnız bırakılır. `--force` ile mevcut dizin, kopyadan önce `<name>.bak-<timestamp>` olarak yeniden adlandırılır.

`argvus` uygulaması özeldir: paketlenmiş dizininin içeriğini yapılandırma köküne kopyalar. Kurulum çalıştırıldığında yapılandırma kökü zaten mevcut olduğundan, `--copy argvus`'nun geçerli kodda `--force`'ya ihtiyacı vardır.

Kurulum, herhangi bir şeyi değiştirmeden önce her seçeneği doğrular. Geçersiz uygulama adları (boş, `.` ile başlayan veya `/` içeren) reddedilir.

## Oturum ve yeniden yükleme

`SUPER + SHIFT + R`, `argvus sessionctl reload`'yu çalıştırır. Oturum denetleyicisi oluşturulan yapılandırmayı uygular ve yalnızca sahip olduğu bileşenleri yeniden başlatır. `/usr/share/argvus`'yu ana dizine kopyalamaz ve hatalı biçimlendirilmiş bir JSON dosyasının yerini almaz.

## Gece Lambası

**Masaüstü ve Giriş > Ekranlar > Gece Işığı**'nı açın veya `argvus control-center displays night-light`'yu çalıştırın. `argvus config display night-light` aracılığıyla sıcaklığı, manuel aktivasyonu ve sabit yerel programları yapılandırın. Komutlar, durum ve çalışma zamanı gereksinimleri için [Gece Işığı](/docs/user-guide/night-light/) konusuna bakın.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]`, paylaşılan ARGVUS terminal çerçevesi içinde etkileşimli bir komut çalıştırır. Ctrl+E çerçeveyi kapatır; arayan kişi yerelleştirilmiş altbilgiyi sağlar. Bunu bir terminalin içinde veya `argvus tui-terminal --term kitty -- ...`'ya iletilen komut olarak kullanın.

`argvus tui-terminal --term kitty --profile ai-agents`, gizli sekmelere ve yalnızca pano kısayollarına sahip özel bir Kitty yapılandırması kullanır. Arayan kişi komutu ve unvanı sağlar; pencere yerleşimi `argvus-ai` sınıfı için Hyprland kuralıyla gerçekleştirilir.