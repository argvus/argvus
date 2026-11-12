---
title: ARGVUS Yapılandırması
description: ARGVUS tarafından kullanılan mantıksal JSON yapılandırma sözleşmesi.
---

ARGVUS, yönetilen kullanıcı tercihlerini `$XDG_CONFIG_HOME/argvus/config/` (normalde `~/.config/argvus/config/`) altındaki modüler JSON dosyalarında tutar. ARGVUS tarafından yönetilen tercihler için kanonik mantıksal modeli oluştururlar ve `argvus-config`, `data/generated/`'nun altındaki dosyaları yazan tek bileşendir. Yerel dosyalar ve oluşturulan dosyalar, uyumluluk projeksiyonları veya uygulamaya özel geçersiz kılmalar olarak kalır.

## Kullanım kılavuzu: Hızlı başlangıç

Çoğu kullanıcı için yapılandırma ARGVUS Kontrol Merkezi veya bu komut satırı aracı aracılığıyla gerçekleşir:

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

Yöneticiler ve gelişmiş yapılandırma için aşağıdaki okumaya devam edin.

Kullanıcı kökü yalnızca standart belgeyi ve yönetilen verileri içerir:

```text
~/.config/argvus/
├── config.json
└── data/
```

Dosya atomik olarak yazılır, kullanıcıya özel tutulur ve projeksiyon sırasında değiştirilebilen data/ ağacının dışında `.config.lock` tarafından korunur. Önceki bir sürüm  `data/` `data/backups/config.json.bak` adresinde depolanıyor.

## Komut satırı

`argvus config` alt komutu, `argvus` paketi tarafından işlem sırasında sağlanır:

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

`ensure`, şema varsayılanlarıyla veya dolgularla yeni bir `config.json` gerçekleştirir
açık değerleri değiştirmeden eksik alanlar. Hatalı biçimlendirilmiş JSON'da başarısız olur.
`migrate`, değerleri değiştirmeden mevcut eski görünüm durumunu içe aktarır
`config.json`'da zaten mevcut. `argvus sessionctl`, `recover`, `ensure`'yu çalıştırır,
Yapılandırma değişikliklerinden sonra yeniden yükleme uygulamadan önce `migrate` ve `project`:

```sh
argvus sessionctl apply-config
```

`apply` ve `rebuild`, her projeksiyonu yeniden oluşturmak için kullanılan takma adlardır.
`config.json`; `project` tarihsel yazımdır ve `--force`'yu kabul eder.
`apply` ayrıca taahhüt sonrası çalışma zamanı yeniden yükleme işlemini `argvus sessionctl`'ya devreder.
Hazırlık veya izole testler için `ARGVUS_NO_RUNTIME=1`'yu ayarlayın.
Oluşturma aşamalandırma sırasında gerçekleşir ve `data/` ağacı ancak sonuçta değiştirilir
projektörler başarılı. Bu nedenle `data/generated/` silinip çalıştırılıyor
`argvus config apply`, kısmi bir belge yayınlamadan tekrarlanabilir bir yeniden yapılanmadır
Hyprland veya Görev Çubuğu çıktılarının sırası. Türetilen komut kayıtları
bölüm karmaları ve çalışma zamanı kararı
`$XDG_STATE_HOME/argvus/config-projection.json` (normalde
`~/.local/state/argvus/config-projection.json`). Ne tezahür ne de
oluşturulan dosyalar gerçeğin kaynaklarıdır. `apply` taahhüt sonrası yeniden yüklemeyi gerçekleştirir
`argvus sessionctl` aracılığıyla; `project`, uyumluluk açısından yalnızca projeksiyon olarak kalır.

Her kalıcı mutasyon aynı kuralı kullanır: Kanonik belge değişirse,
CLI, taahhütten sonra merkezi bir `argvus sessionctl reload`'yu çağırır.
yeniden yükleme sonucu `data/internal/last-reload.json`'ya kaydedilir.

`argvus sessionctl reload` bu nedenle bir yansıtma adımı değil, *uygulama* adımıdır.
Gelen bildirim, bekleyen çalışmayı bildirdiğinde, hedefleneni geri yükler.
plan tarafından açıklanan ve `argvus config project`'yu tekrar aramayan tüketiciler;
yeniden projelendirme aynı ağacı yeniden inşa eder ve mutasyon planını iptal eder
yeni üretildi. Yalnızca taahhüt edilmediğinde geçişe ve projeksiyona geri döner
örneğin oturumun başlatılması sırasında plan mevcut.

### Hedefleri yeniden yükle

`argvus config session-plan`, yeniden yüklemeye hedefleriyle yanıt verir.
değişen bölümler vardır ve `argvus sessionctl` yalnızca bunları çalıştırır. Sıçrama
yalnızca hedef bir yüzeyi yeniden oluşturduğunda yeniden yüklemeyi kapsar: bir Yol Çubuğu çubuğu veya
Kontrol Paneli ekrana geri döndüğünde her pencere değişir. Diğer her şey
yerine yeniden yüklenir.

| Değiştir | Hedefler | Sıçrama |
| --- | --- | --- |
| Tema, vurgu rengi veya GTK modu | `hyprland`, `consumers`, `wallpaper`, `polkit` | hayır |
| Yalnızca duvar kağıdı | `wallpaper` (hyprpaper IPC; hizmet yalnızca swaybg için yeniden başlatılır) | hayır |
| `effects` (bulanıklaştırma, animasyonlar, şeffaflık, gölge) | `hyprland`, `consumers` | hayır || `layout` pencereleri (boşluklar, kenarlıklar) | `hyprland` | hayır |
| `layout.taskbar` (konum, kenar boşlukları) | `hyprland`, `taskbar`, `widget_telemetry` | evet |
| `fonts` | `consumers` | hayır |
| `control_panel` (anahtar, kartlar) | `control_panel` | evet |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | hayır |
| `power` | `hypridle` | hayır |

`consumers`, `argvus config theme-runtime consumers`'yu (çubuk stil sayfaları,
IPC üzerinden Kontrol Paneli, Hyprland renkleri, Dunst temasının yeniden yüklenmesi, btop ve
kedicik). Dunst, Rahatsız Etmeyin etkinken temanın yeniden yüklenmesini erteler.
`wallpaper`, `argvus config theme-runtime wallpaper`'yu çalıştırır;
etkili `/appearance/wallpaper`. `layout.taskbar` karması tutulur
projeksiyon bildirimindeki pencere düzeninden ayrı olarak, bir boşluk değişikliği
çubukları yeniden başlatmayın. Taahhütlü bir plan olmadığında kullanılan yedek `full` planı
mevcutsa, sıçramanın altındaki her yüzeyi yeniden başlatır.

Her etkili konfigürasyonun operasyonel bir üretimi vardır. İçinde saklanır
`data/internal/generation.json` ve aşağıdaki projeksiyon bildiriminde
`$XDG_STATE_HOME/argvus/config-projection.json`; nokta dosyalarının bir parçası değil
konfigürasyon. Değiştirilmemiş etkili bir konfigürasyonun yeniden uygulanması,
nesil. `status`, bildirim karmasının, oluşturma meta verilerinin ve
oluşturulan tüm dosyalar aynı fikirdedir; `apply` eski veya eksik projeksiyonu onarır.

`explain /path` doğrudan çözümleyiciyi okur ve etkin değeri yazdırır,
kaynak, tema ve varyant. Oluşturulan dosyaları ayrıştırmaz veya ikinci bir dosya oluşturmaz
öncelik kuralı.

Tema ve efekt projeksiyonu `argvus-config`'nun kendisinde uygulanmaktadır.
Tema işlemi (theme-apply'yu çalıştıran `argvus theme-switch`),  `theme-apply` `argvus config accent` ve `argvus effects-toggle` şunları yansıtmaz:
bu CLI aracılığıyla doğrularlar, kilitlerler, taahhüt ederler ve ardından
`config.json` okuyamayan harici tüketicilerin mutabakatını sağlayın. `argvus effects-toggle`
kendine ait çıktısı olmayan zayıf bir delegedir. Görünüm yardımcıları asla
projektör tarafından çağrılır ve asla `data/generated/`'nun altına yazmamalıdırlar.

### Projeksiyon yüzeyleri

`project_effective`, ARGVUS'a ait her tüketiciyi tek bir hazırlama ağacında oluşturur
yayınlanmadan önce:

| Yüzey | Çıkış |
| --- | --- |
| Hyprland | `generated/hypr/`, tema profilleri dahil, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` ve `monitors.lua` |
| Düzen/efektler | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Yol Çubuğu | `generated/waybar/`, `argvus-taskbar.css` ve `argvus-widget-telemetry.{jsonc,css}` dahil |
| Widget telemetri temaları | `generated/widget-telemetry/` ve `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Bildirimler | `generated/notifications/` ve `dunstrc` |
| Kilit ekranı | `generated/hypr/hyprlock.conf` ve Hyprtoolkit tema profilleri |
| GTK | `generated/gtk/` |
| Yazı | `generated/yazi/` |
| Süper dosya | `generated/superfile/` |
| Terminal ve Qt profilleri | `generated/terminal/`, `generated/qt6ct/` |
| Yazı Tipleri | `generated/fonts.conf` artı Waybar stil sayfalarında yönetilen yazı tipi blokları |
| Çıkarılabilir cihazlar | `generated/removable-devices/theme.css` |
| Duvar Kağıdı | `generated/hypr/hyprpaper.conf` ve `data/hypr/hyprpaper.conf` |
| Denetim Masası ve varsayılanlar | `data/control-panel/`, `data/control-center/defaults.json` |
| Takvim, giriş, klavye, güç | takvim TOML, giriş parçası, klavye durumu, uyanık tutma işaretçisi |

Tema katmanı ayrıca oluşturulan kökün dışına da yansıtılır:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` ve
`data/hypr/application-style.conf`. Bu kopyalar ambalajlı olarak değiştirilir
görünüm bölümü değiştiğinde varsayılanlar olur ve projektör daha sonra yeniden yazar
yalnızca sınırlandırılmış yönetilen blokları - Waybar stil sayfalarındaki yazı tipi bloğu ve
telemetri profilindeki `ARGVUS_TELEMETRY_*` blokları — yani metin
işaretçiler bir tema değişikliğinden sağ çıkamaz. `fonts.targets` ve
`control_panel.widget_telemetry_blocks` standart olmaya devam ediyor: Kontrol Merkezibunları `argvus-config` aracılığıyla sürdürür ve bu dosyaları asla kendisi yazmaz.

Widget telemetri bloğu seçimi şu tarihte standarttır:
`/control_panel/widget_telemetry_blocks`, yedi blok adından oluşan bir dizi
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`).
eski `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` dosyası yalnızca bir dosya olarak okunur
geçiş geri dönüşü ve asla yazılmaz. `argvus widget-telemetry-geçiş blokları
set` and ` tek bir atomik yazmada kanonik diziyi tüm yamaları engeller ve
`apply-state`, arayan kişinin seçime zaten devam ettiğini varsayar.

Hyprland aralığı ve kenar tercihleri ​​aynı sınırı takip eder.
argvus config boşlukları ve argvus config sınırları komutları (ve bunların geçmişleri)
uyumluluk sarmalayıcıları) güncelleme/düzen; Pas oluşur
veri/oluşturulan/spaces-etkin.conf ve veri/oluşturulan/borders-etkili.conf
doğrulama ve canlı hiprctl uygulamasının sahibidir. .boşluklar ve .kenarlıklar
yalnızca göçe açık bir durumdur ve asla gerçeğin kaynağı değildir.

Accent ve GTK modu durumu aynı Rust sınırını kullanır. 'argvus-config'i kullanın
vurgu COLOR` or `argvus yapılandırma vurgusu - vurgu sahipliği için tema-varsayılan`,
ve GTK modu için `argvus config mode toggle|set dark|light`. Tarihsel
vurgu ve mod komut dosyaları uyumluluk sarmalayıcılardır; `.accent-color`,
`.accent-custom` ve `.gtk-mode` yalnızca geçiş/projeksiyon dosyalarıdır.

Mantıksal ARGVUS yolları da `argvus config paths` tarafından çözümlenir. Miras
`paths.sh` dosyası yalnızca uyumluluk işlevlerini gösterir; yerel geçersiz kılmalar, kullanıcı
veriler, oluşturulan veriler ve sistem geri dönüşü Rust yol modeliyle çözümlenir.

`audio`, varsayılan çıkış havuzunu değiştirir ve sonucu tek adımda `/audio`'ya kaydeder: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` ve `argvus config audio mute <on|off|toggle>`. PipeWire zaten değere sahip olduğundan, yapılandırmayı yeniden yüklemeden depoya yazar. Waybar ses modülü, Hyprland medya anahtarları ve Kontrol Paneli bunu kullandığından, /audio'yu havuza yansıtan `argvus config project`, asla eski bir birimi geri yüklemez. %100'ün üzerindeki bir hacim (Hyprland tuşları %150'ye kadar izin verir) 100 olarak depolanır.

 `/audio` `patch`, anahtarları JSON işaretçileri olan bir JSON nesnesini kilitli bir durumda uygular
okuma-değiştirme-yazma işlemi. `apply-theme` eşdeğer atomik işlemi gerçekleştirir
tema işlemidir ve yönetilen görünüm bölümlerini değiştirir.
temanın isteğe bağlı görünüm profili veya paketlenmiş varsayılanları.
Mutasyona uğrayan komutlar bir `NextUserConfig` oluşturur, bir `EffectiveConfig`'yu çözer, aşama
tüm projektörler ve ancak bundan sonra standart JSON'u yayınlayın. Operasyonel
`.config-transaction.json` işaretleyici, `argvus config recover`'nun yeniden projeksiyon yapmasına olanak tanır
veriler ile JSON yayını arasındaki kilitlenme sonrasında önceki yapılandırma.
Tema varsayılanları her temanın manifestosu tarafından bildirilir
(`themes/<theme>/theme.toml`): varsayılan vurgu (`palette.accent`), GTK renk şeması
ve duvar kağıdı. CSS, Lua, Rasi ve Qt dosyaları bundan tema yapısına dönüştürülür.

Açık/Koyu da dahil olmak üzere temayı değiştirmek, vurgu rengini temanın varsayılan rengine sıfırlar.
kullanıcı özel bir renk seçse bile: özel renk yalnızca bir sonraki temaya kadar korunur
değiştirin. Tema değişikliğinden sonra tekrar ayarlamak için `argvus config accent` kullanın. Bir uygulama
`argvus config theme-apply` içeren tema `/appearance/accent` ve `/appearance/gtk_mode` yazar
bildirimden ve `argvus config accent` seçilen rengi `accent_custom` ile yazar
tek komutla ayarlayın.

`apply-theme`, varyantı açıkça `--variant sticky|spaced` olarak alır ve
talep edilen adı manifest aracılığıyla çözer, böylece bir istek asla
diğer varyantın varlıklarına çözümleyin. `theme-manifest THEME` aynı
salt okunur olarak gösterilen çözünürlük: bildirim girişini,
`asset_id` ve `layout_variant` çözümlendi; arayan kişinin yapması gereken de budur
paketlenmiş varlıkları adresleyin. Yapışkan, eklenmemiş kimliktir ve yalnızca Spaced bir karakter taşır.`-spaced` son eki, dolayısıyla `dracula-sticky` ve `argvus-dark-dracula-sticky`
kabul edilir ve `dracula`'ya normalleştirilir.

`--raw` ile dize değerleri JSON tırnak işaretleri olmadan yazdırılırken nesneler ve diziler JSON değerleri olarak kalır. Tema ve düzen nokta dosyaları artık projeksiyon yardımcısı tarafından yazılmıyor; `config.json` bu tercihler için tek kalıcı kaynak olmaya devam ediyor. Yansıtma başarısız olursa, masaüstü yüzeyleri yeniden başlatılmadan önce oturumun yeniden yüklenmesi durdurulur, böylece daha eski bir kısmi tema güncel olarak sunulmaz.

## Dışa aktarma ve içe aktarma

Profil değişimi için aynı mantıksal model kullanılır:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```

Yönetilen masaüstü modelinin tamamı için `--scope desktop` kullanın. Eksik alanlar ARGVUS varsayılanlarını kullanır; oluşturulan çıktı doğrudan düzenlenmemelidir.

Kontrol Merkezi teması, bir temayı ve görsel profilini dışa ve içe aktarır: arşiv, $XDG_DATA_HOME/argvus/themes/ altına yüklenen bir şema 2 tema dizinini (`<id>/theme.toml`, isteğe bağlı `appearance.json` ve `effects.conf`, `overrides/`, duvar kağıdı) içerir. Profilde görünüm, düzen geometrisi, görsel efektler, yazı tipleri ve panel sunumu bulunur. Ses, klavye, ağ ve diğer görsel olmayan tercihler temaların dışında kalır. Görünüm yapılandırmasının tamamını taşımak için yukarıdaki  `$XDG_DATA_HOME/argvus/themes/` `appearance` kapsamını ve tam bir yapılandırma yedeklemesi için `desktop` kapsamını kullanın.

`appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps` altındaki mevcut şema grupları tercihleri, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` ve `removable_devices`. `default_apps`, tercih edilen uygulamalar için tek kalıcı kaynaktır ve `keyboard_shortcuts`, dize değerlerine sahip kararlı İngilizce, yerel ayardan bağımsız anahtarları veya devre dışı bırakılan bağlamalar için `null`'yu saklar.

Kontrol Merkezi, Görünüm sayfasını açtığında kurallı görünüm değerlerini okur ve değişiklikleri `argvus-config` aracılığıyla yazar. Mevcut yerel durum dosyaları, görünüm komut dosyaları ve eski kurulumlar için projeksiyon olarak kalır.

Kontrol Paneli, kart sırası ve görünürlük için `control_panel.cards`'yu ve panel ana anahtarı için `control_panel.enabled`'yu kullanır. `control-panel/cards.json` ve `state/control-panel` dosyaları uyumluluk projeksiyonları olarak tutulur. Bu nedenle Kontrol Panelinde yapılan değişiklikler `desktop` dışa aktarma kapsamının bir parçasıdır.

Yazı tipi tercihleri, görev çubuğu, telemetri, Denetim Masası, sistem, uygulamalar, terminal ve tarayıcı hedeflerinin ailesi, stili ve boyutu için `fonts.targets`'yu kullanır. `fonts.rendering` kenar yumuşatma, ipucu, alt piksel sırası ve DPI tercihlerini saklar. `fonts.conf`, fontconfig kuralı, masaüstü ayarları ve Waybar stil sayfaları içindeki yönetilen font blokları projeksiyonlardır: Kontrol Merkezi, `/fonts/targets/*`'yu `argvus-config`'ya kadar sürdürür ve projektör yalnızca kendi sınırlandırılmış bloğunu yeniden yazar.

Hyprland tercihleri, kısayollar için `keyboard_shortcuts`'yu, fare ve dokunmatik yüzey ayarları için `hyprland.input`'yu ve XKB düzeni, çeşidi ve seçenekleri için `hyprland.keyboard`'yu kullanır. Açık bağlama kimliği, kararlı bir `config_key` ile eşleşir (örneğin, `window.close` → `close_window` ve `window.drag_mouse` → `drag_window__floating_window_only`). Ekran tercihleri ​​`displays.monitors`, `displays.workspaces` ve `displays.primary_monitor`'yu kullanır. Yerel TOML dosyaları ve `data/generated/` kapsamında oluşturulan Lua dosyaları, Hyprland tarafından tüketilen projeksiyonlar olmaya devam ediyor.Her `effects` anahtarı, sanal makinelerde ve düşük güçlü oturumlarda da varsayılan olarak etkindir (animasyonlar, bulanıklaştırma, widget telemetrisi ve her şeffaflık yüzeyi); pencere gölgesi (`shadow_enabled`) ve yalpalama (`wobble_enabled`) varsayılan olarak kapalıdır. Efekt tercihleri, Animasyonlar ana anahtarı için `effects`'yu kullanır (`animations`, blur_global_enabled artı blur_global_enabled olan Efektler anahtarından bağımsız olarak yansıtılan hypr/state.lua'dan `hyprland.lua`'ya uygulanır)  transparency_<surface>_enabled  blur_global_enabled  `hypr/state.lua`  `blur_global_enabled`  `transparency_<surface>_enabled` `taskbar`, `control-panel`, `widget-telemetry`, `terminal` ve `launchers`'nun transparency_<surface>_enabled'su ve bunlardan herhangi biri etkinleştirildiğinde etkin olarak okunur; `argvus effects-toggle animations|effects` bunları değiştirir), genel bulanıklık anahtarı (`blur_global_enabled`), Hyprland bulanıklık parametreleri (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), pencere gölgesi (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), Animasyon değerleri (`animation_<leaf>_speed`, `_curve`, `_style`, Her animasyon için `_popin` ve `_enabled`, ayrıca özel animasyonlar için `animation_bezier_x1` - `animation_bezier_y2` ve `animation_spring_mass`, `animation_spring_stiffness` ve `animation_spring_dampening` eğriler) ve yüzey başına şeffaflık değerleri. Efektler değişkene ait değil, temaya aittir: Yapışkan ve Aralıklı aynı `<theme>.conf` efekt projeksiyonunu kullanırken düzen projeksiyonları farklıdır. Yüzeyler şu anda `taskbar`, `control-panel` ve `widget-telemetry`'yu içermektedir; şeffaflık değerleri `0` ila `100` arasındaki yüzdelerdir (Kontrol Merkezi ve başlatıcı yüzeyleri varsayılan olarak `15`'dur, diğerleri `50`'dur), bulanıklık parametreleri `schema.json`'da bildirilen aralıklar içindeki sayılardır ve etkin durumlar boolean'lardır. Animasyon değerleri `argvus-config` tarafından da doğrulanır: hızlar `1` ile `10` arasındaki sayılardır (ondalık sayılara izin verilir), açılır yüzdeler `1` ile `100` arasındaki tam sayılardır, eğriler Animasyonlar'daki adlar olmalıdır sayfasında, stiller Animasyonlar sayfasındaki adlar olmalı ve animasyonun kendi yaprağı tarafından kabul edilmelidir (örneğin, animation_workspace_in_style için `popin` reddedilir,  `animation_workspace_in_style` `gnomed` yalnızca pencere açma ve kapatma için kabul edilir ve pencere ve katman yapraklarında slayt çeşitleri reddedilir), Bézier Y noktaları arasında kalır `-1` ve `2`, Hyprland 0.56.2 aralığı ayrıştırır ve diğer eğri parametreleri belgelenen aralıklarında kalır. `layers` ve `switching` yaprakları artık mevcut olmadığından, kaldırılmadan önce kaydedilen yapılandırmalar, yüklendiklerinde `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` ve `animation_switching_*` anahtarlarını kaybeder. Geçerli kümenin dışındaki stiller çıkarılır ve `-1` ile `2` arasındaki Bézier Y noktaları sabitlenir, böylece paketlenen varsayılanlar, kaldırılanlara yeniden uygulanır.

Güç tercihleri ​​`power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` ve kapak/güç düğmesi politikalarını kullanır. Yansıtılan formlar `data/generated/hypr/hypridle.conf` ve `data/power/keep-awake` işaretçisidir; `.keep-awake` yalnızca bir geçiş girişidir. `session.language` kullanımdan kaldırıldı: mevcut değerler yok sayıldı, yeni yazma işlemleri reddedildi ve hiçbir dil uyumluluk dosyası yansıtılmadı. Bunun yerine locale.conf kullanıcısında `LANG`'yu ayarlayın. Takvim tercihleri ​​ `locale.conf` `calendar` kullanır ve görev çubuğu-takvim kullanıcısı TOML'ye yansıtılır. Çıkarılabilir cihaz tercihleri ​​`removable_devices`'yu kullanır; çıkarılabilir aygıtlar uygulaması bu bölümü, paketlenmiş varsayılanlar üzerinde kullanıcı tarafından geçersiz kılma olarak ele alır.`migrate`, mevcut kurallı değeri koruyarak eski kök dosyalarını ve bölümleri bir kez içe aktarır. Eski `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, nokta dosyaları, `generated/` ve bileşen dizinleri `data/`'nun altına taşınır; tekrarlanan yeniden yüklemeler eski kök düzenini yeniden oluşturmaz. Geçerli temanın yeniden seçilmesi önemsizdir: Kanonik durum ve projeksiyonlar zaten aynı fikirdeyse, projeksiyon planı değişen hiçbir bölüm olmadığını bildirir ve oturum, sıçramayı atlar ve hizmet yeniden başlatılır.

Güç aynı projeksiyon sözleşmesini takip eder: Kontrol Merkezi, config.json'da `/power` yazar,  `config.json` `argvus config project`, `data/hypr/hypridle.conf`'yu atomik olarak işler ve ancak o zaman `argvus sessionctl`, `argvus-hypridle.service`'yu yeniden başlatabilir. Bu dosyanın kaldırılması `argvus config project` ile kurtarılabilir; `0` değeri karşılık gelen zaman aşımını atlar ve Hiçbir zaman anlamına gelir.

Kilit ekranı görünümü Rust tarafından standart tema, vurgu,
efektleri, duvar kağıdını ve yazı tipi durumunu `data/hypr/hyprlock.conf`'ya dönüştürün. Tarihsel
`hyprlock-theme.sh` giriş noktası yalnızca uyumlulukla ilgilidir ve
`argvus config lock apply`; `--invalidate` yalnızca türetilen bulanıklığı geçersiz kılar
duvar kağıdı önbelleği.

Güç zaman aşımı değişiklikleri `/power/lock_minutes`'yu kullanır ve
`/power/screen_off_minutes` onların gerçeğin kaynağı olarak. `argvus config project`
mevcut Hypridle oluşturucuyu yeniden kullanır ve `idle-timeout.sh` giriş noktası
yalnızca bir uyumluluk adaptörü; hizmetin yeniden başlatılması şu kişiye devredilmiş olarak kalır:
`argvus sessionctl`.

Bildirim görünümü standart tema, aksan ve
efektleri `data/notifications/dunstrc`'ya aktarın. Tarihsel bildirimler
`theme.sh` dosyası yalnızca çağrılan uyumluluk işlevlerini ortaya çıkarır
`argvus config notifications`; Dunst yapılandırmasını ayrıştırmaz veya değiştirmez.

Ekran bulma ve Bluetooth kontrolü gibi donanım odaklı bağdaştırıcılar
artık Rust görüntü alanı tarafından içe aktarılıyor: `nwg-displays` harici bir içe aktarıcıdır,
`/displays` ve `data/generated/hypr/monitors.lua` kanonik kalırken ve
sırasıyla türetilmiş durum. Görüntüleme araçları düzeni şu şekilde değiştirir:
`argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` ve `monitor-remove`; ekran komutları `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`'dur. Hyprland dosyası `/displays`'dan yansıtılmıştır; artık hiçbir kabuk betiği yazmıyor. Duvar kağıdı seçimi, görev çubuğu yardımcı grup modu,
ve varsayılan uygulama araması da `argvus-config` API'lerini kullanır; onların tarihi Kabuğu
giriş noktaları kullanıcı arayüzü veya uyumluluk sarmalayıcılarıdır. Uyanık Tut kalıcı bir uygulamadır
`/power/keep_awake` tercihi `data/power/keep-awake` işaretleyicisine yansıtılır,
lock-DPMS, oturum çalışma zamanı durumu iken
`$XDG_CACHE_HOME/argvus/lock-dpms`. Klavye düzeni artık Rust'a ait
`argvus-keyboard-layout` yardımcı: yapılandırılmış sipariş, oluşturulan siparişten gelir
Etkin düzen kalırken Hyprland giriş projeksiyonu veya yerel ayar geri dönüşü
Hyprland'dan keşfedilen oturum çalışma zamanı durumu. Döngü giriş noktası ve
klavye düzeni arka plan programı yalnızca uyumluluk sarmalayıcılardır; kullanıcı hizmeti çalışır
doğrudan Rust arka plan programıdır ve `argvus sessionctl keyboard-layout cycle`
arayanlar için kararlı komut.

Bluetooth çalışma zamanı durumu harici adaptör sınırını takip eder ancak bunun bir parçası değildir
`argvus-config`: `argvus bluetooth`, yazılan ayrıştırmaya sahiptir ve
`bluetoothctl` etrafında doğrulama, sınırlı yeniden denemeler, sistem hizmeti kontrolleri ve
RFKill durumu. `argvus bluetoothctl` giriş noktasıdır. Adaptör gücü, eşleştirilmiş cihazlar ve bağlı cihazlar
standart JSON durumu yerine BlueZ çalışma zamanı durumu olarak kalır; donanım canlı
Bluetooth doğrulaması çevreye bağlıdır.
## Çalışma zamanı kimliği ve mutabakatı

Tam ikili dosyayı incelemek için `argvus config status --verbose` kullanın, derleyin
kimlik, konfigürasyon kökü, oluşturulan kök, varlık kökü, bildirim kaynağı ve
aktif nesil. `synchronized`, `stale`, `incomplete`, `legacy` raporlarını verir
ve `pending-transaction` projeksiyon durumları.

`argvus config --version` paket sürümünü bildirir, sağlandığında revizyon oluştururyapı ortamına ve profile göre. Geliştirme testleri kendi
ikili ve varlıklar açıkça; sessizce `argvus config` kullanmamalıdırlar
veya `~/.config/argvus`. `apply` eski birleştirilmiş tanımlayıcıları güvenli bir şekilde normalleştirir
`argvus-dark-spaced` gibi ayrı tema ve değişken değerlerine ayırmadan önce
projeksiyonları yeniden inşa ediyoruz. Paketlenmiş varlık kimliği bu sürümdeki bildirime kaydedilmemiştir.

## Gece Lambası

**Masaüstü ve Giriş > Ekranlar > Gece Işığı**'nı açın veya `argvus control-center displays night-light` komutunu çalıştırın. `argvus config display night-light` aracılığıyla sıcaklığı, manuel aktivasyonu ve sabit yerel programları yapılandırın. Komutlar, durum ve çalışma zamanı gereksinimleri için [Gece Işığı](/docs/user-guide/night-light/) konusuna bakın.

### Wobble ayarları (Hyprland 0,57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`,
`wobble_intensity` ve `wobble_mesh` doğrudan `hypr/state.lua`'ya yansıtılır.
Bunları okumak için `argvus effects-toggle wobble-settings get` kullanın veya
`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`
Kaydetmek ve yeniden yükleme istemek için. `enabled=disabled` efekti kapatır.

Kontrol Merkezi, Animasyonlar taslağını paylaşıyor: `animations-settings get` ayrıca
`wobble_*` girişlerini döndürür ve `animations-settings set` bu tam adları kabul eder
mevcut animasyon argümanlarının yanı sıra. Bu, tüm taslağın tek bir yerde kalmasını sağlar
doğrulanmış yama ve bir yeniden yükleme talep ediyor. Örneğin:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```

Küresel `enabled`, animasyonları kontrol eder; `wobble_enabled` bağımsızdır
tercih. Genel animasyonların devre dışı bırakılması, Lua'daki yalpalamayı silmeden bastırır
bu tercih. Geçersiz değerler yamanın tamamını reddeder. Eksik anahtarlar alınıyor
ARGVUS, mevcut seçenekleri değiştirmeden varsayılan değerlere geçer. Bkz. [Görünüm](/docs/user-guide/appearance/#window-wobble)
sınırlar ve varsayılanlar için.


### Kullanıcı tarafından oluşturulan klavye kısayolları

`/keyboard_shortcuts`, resmi bildirim eylemleri için mevcut dize veya boş değerlerini korur. Özel genel klavye kısayolları, config/hyprland.json'da depolanan ayrı bir `/hyprland/custom_shortcuts` nesnesini kullanır. Kararlı  `config/hyprland.json` `custom_` kimlikleri açıklamalardan ve yerel ayarlardan bağımsızdır. Her kayıtta `description`, `keys`, `enabled` ve yazılan bir `action` bulunur:

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

Örnek, tam bir yapılandırma belgesi değil, `/hyprland/custom_shortcuts` değeridir. Sevk görevlisi adları, desteklenen Lua sevk görevlisi fabrikalarından gelir; bağımsız değişkenler isteğe bağlı bir JSON nesnesidir veya `null`'dur; hiçbir zaman değerlendirilmeyen koddur. Özel kısayollarda alt harita, fare bağlama veya gelişmiş bayraklar yoktur. Projeksiyon, `data/generated/hypr/custom-shortcuts.lua`'yu, son kayıt kaldırıldığında boş bir tablo da dahil olmak üzere, çıkışlı veri tablosu olarak yazar. Hem tam projeksiyon hem de `project --only keyboard_shortcuts` bunu oluşturur. Hyprland, etkin girişleri `hl.bind` ile kaydeder; isteğe bağlı dosya ve bireysel kayıt hataları muhafaza edilir ve günlüğe kaydedilir.

Kontrol Merkezi, bir `argvus config patch` işlemini kullanarak kayıtları oluşturur/düzenler ve onaylanmış çakışmaları devre dışı bırakır. Mevcut yapılandırma hizmeti aracılığıyla başvurmadan önce kaydeder. Başarısız bir uygulama, başarılı bir kaydetme işlemini geri almaz. Resmi varsayılanların geri yüklenmesi, özel kayıtları korur ve etkin bir özel kısayolla çakışması durumunda reddedilir. Kullanıcı açıklamaları tüm yerel ayarlarda ve etkili bir şekilde oluşturulmuş kopya sayfasında gerçek metin olarak kalır.

## Genel pencere modu

**Denetim Merkezi → Masaüstü ve Giriş → Hyprland → Windows → Mod**'u açın ve **Pencereleri Döşe (Varsayılan)** veya **Kayan Pencereler**'i seçin. Aynı seçiciye görev çubuğu yardımcı program grubundaki pencere modu simgesinden de ulaşılabilir.

Tercih her monitör ve çalışma alanı için geçerlidir ve oturum kapatıldıktan sonra hatırlanır. Kayan, mevcut uygulama pencerelerini ve yeni açılan pencereleri dönüştürür. Döşeme yalnızca bu politikanın dönüştürdüğü pencereleri geri yükler; Halihazırda yüzen diyaloglar ve diğer pencereler yüzer durumda kalır. Kayan etkinken açıkça sabitlenen bir pencere, iğnesini korumak için kayan durumda kalır. Döşenmiş düzen yeniden hesaplanır; önceki bölünmüş ağacın tamamı geri yüklenmez.`SUPER + SHIFT + Space` Tile'da pencere başına normal davranışını koruyor. Floating'de bir pencereyi döşemeye geri döndürmez. Paneller, görev çubuğu ve bildirimler katman kabuğu yüzeyleridir ve etkilenmez. Tam ekran ve büyütme, Hyprland'ın normal davranışını kullanmaya devam ediyor.

Kanonik tercih $XDG_CONFIG_HOME/argvus/config/hyprland.json'da `/hyprland/window_mode`'dur. Oluşturulan projeksiyonu  `$XDG_CONFIG_HOME/argvus/config/hyprland.json` `data/generated/hypr/window-mode.lua`'dur; düzenlemeyin. Pencere başına geri yükleme kaydı, `$XDG_RUNTIME_DIR/argvus` kapsamında bulunur, kapsam birleştirici örneğine ayarlanır ve yapılandırma yeniden yüklemelerinden sağ çıkar. Yeni bir oturumda tekrar kullanılmaz.

Uygulama beklemedeyse veya başarısız olursa, arayüz bunu kaydedilen tercihten ayırır. Yeniden denemek için seçeneği tekrar seçin. Bir besteci olmadan, tercih bir sonraki oturum açma işlemi için kaydedilir.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```

## Pencere düğmeleri ve uygulama çubuğu

Kayan mod, normal uygulama pencerelerine **Küçült**, **Büyüt / Geri Yükle** ve **Kapat** içeren bir ARGVUS başlık çubuğu ekler. Başlık çubuğuna çift tıklamak büyütmeyi değiştirir. Ek dekorasyon Döşeme ve tam ekranda gizlidir. Kendi denetimlerini çizen uygulamalar her iki çubuğu da görüntüleyebilir; tek tip ARGVUS kontrolleri kasıtlıdır.

Her monitörde ana görev çubuğunun karşısındaki kenarda ikinci bir Yol Çubuğu görünür. Yalnızca o monitörün mevcut çalışma alanına ait pencereleri listeler; söz konusu çalışma alanındaki simge durumuna küçültülmüş pencereler de dahil. Açık özel bir çalışma alanı önceliklidir. Sabitlenmiş pencereler görünür durumda kalır; gizli grup sekmeleri fazladan giriş oluşturmaz ve simge durumuna küçültülmüş bir grupta bir kurtarma girişi bulunur. Bir uygulamanın içindeki sekmeler ayrı olarak numaralandırılmaz. Küçültülmüş girişler orijinal monitör/çalışma alanı ilişkilerini korur. Etkinleştirmek için etkin olmayan bir pencereyi, simge durumuna küçültmek için etkin bir pencereyi veya geri yüklemek için simge durumuna küçültülmüş bir pencereyi tıklayın; içerik menüsü geri yükleme, simge durumuna küçültme, büyütme/geri yükleme ve kapatma olanağı sunar. Bilinmeyen uygulama simgeleri bir pencere sembolüne geri döner. Bir girişi etkinleştirmek veya geri yüklemek işaretçinin konumunu korur. İkincil çubuk, yüksekliği ve yansıtılmış kenar boşlukları karşı kenarda olacak şekilde etkili birincil CSS'yi (renkler, yazı tipleri, kenarlık ve yuvarlama) kullanır.

`SUPER + Alt + Up`, ana çubuğu üste ve pencere çubuğunu aşağıya yerleştirir. `SUPER + Alt + Down` bunun tersini yapar. Bir yönün tekrarlanması konumları değiştirmeden bırakır. Döşemenin yalnızca ana çubuğu vardır. Döşemeye geri dönmek, pencere çubuğunu kaldırmadan önce simge durumuna küçültülmüş tüm pencereleri geri yükler ve ardından orijinal döşemeli/değişken ilkesini korur.

Minimizasyon, ayrılmış `special:argvus-minimized` çalışma alanını kullanır. Bu çalışma alanını manuel olarak kullanmayın. `argvus windows service` işlemi, `$XDG_RUNTIME_DIR/argvus` altında birleştirici örneğine özgü çalışma zamanı kaydında kimliği, orijinal çalışma alanını/monitörü, geometriyi, tam ekranı/büyütmeyi ve pin durumunu kaydeder. Kayıtlar, hizmetin yeniden başlatılmasından veya yapılandırmanın yeniden yüklenmesinden sonra hayatta kalır, ancak derleyici oturumları arasında geçiş yapmaz. Bir pencerenin kapatılması kaydının kaldırılmasına neden olur. Bir monitör kaybolursa, geri yükleme mevcut bir çıkışı kullanır. Eksik veya arızalı bir kurtarma çubuğu, yeni en aza indirmeleri önler; başarısız restorasyon, kaydı ve kurtarma çubuğunu korur.

Oturum, `argvus-windows.service` ve `argvus-window-bar.service`'ya sahiptir. İkincisi, Waybar çocuğunu Tile'da hareketsiz tutuyor. Hyprbars eklentisi, x86_64 üzerinde Hyprland 0.57.0 ABI için oluşturulmuş `argvus-hyprland` tarafından paketlenmiştir; kullanıcının ana dizininde derlenmez. Eklentiyi bir besteci ABI güncellemesiyle yeniden oluşturun.

Komutlar: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` ve `minimize|restore|activate|maximize|close ADDRESS`. Pencere adresleri listeden gelir. `position top --persist`, ana çubuğu yeniden başlatmadan tercihi saklar/yansıtır. Yapılandırma ve çalışma zamanı hatası ayrıntıları günlüğe yazılır; görünür eylem hataları da çevrilmiş bir bildirim gösterir.**Aralıklı + Yüzen** modunda pencereler dış boşluklara ve her iki çubuğun ayırdığı alanlara saygı gösterir. Sınırlar, aynı pencere kenarlığının içinde yer alan kenarlığı ve başlığı içerir. ARGVUS kontrolleri aracılığıyla maksimuma çıkarmak bu alanı kenar boşluklarıyla doldurur; geri yükleme, hizmetin yeniden başlatılması da dahil olmak üzere önceki geometriyi kurtarır. Tam ekran hala çıktının tamamını kullanıyor. Yüzen pencereler üst üste gelebilir; iç boşlukların döşenmesi onları ayırmaz.

Pencere dış boşluklarında çubukların ayırdığı iç kenar boşlukları iki kez sayılmaz. Simetrik aralıkta, çubukların değiştirilmesi sonrası da dahil olmak üzere görünür pencere-çubuk mesafesi üstte ve altta eşittir.

`SUPER + Alt + Space`, görev çubuğu ve Kontrol Merkezi ile aynı tercihi kullanarak genel Döşeme/Kayan mod arasında geçiş yapar. Eşdeğer komut `argvus config window-mode toggle`'dur. Eşzamanlı çağrılar serileştirilir; Geçiş, uygulama beklemedeyken bile kaydedilen tercihi kullanır. Kısayol, Klavye Kısayolları'nda değiştirilebilir veya devre dışı bırakılabilir; `SUPER + Shift + Space` bireysel pencere eylemini koruyor.
