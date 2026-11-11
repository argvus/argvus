---
title: Yapılandırması ve durumu
description: Paylaşılan durum, geçersiz kılmalar ve oluşturulan çalışma zamanı dosyaları.
---

Normal kullanıcı kökü `$XDG_CONFIG_HOME/argvus`'dur (genellikle `~/.config/argvus`). Yönetilen mantıksal tercihler `config.json`'da saklanır. Tema, vurgu, duvar kağıdı, aralık ve kenarlık tercihleri ​​burada standarttır; nokta dosyaları mevcut Hyprland Lua ve kabuk tüketicileri için türetilmiş uyumluluk projeksiyonları olmaya devam ederken, bu tüketiciler `data/`'ya geçiş yapıyor.

`argvus-config`, `generated/`'nun altındaki tek yazardır. ARGVUS'a ait her
tüketici dosyası, Hyprland, GTK, Qt6ct dahil olmak üzere `config.json`'nun bir projeksiyonudur.
Waybar, Quickshell, Dunst, Hyprlock, Waybar görev çubuğu ve widget telemetrisi
profilleri, Yazi ve Superfile yapılandırma ağaçları, terminal ve Qt profili
ağaçlar, yazı tipi yönetimli bloklar ve çıkarılabilir cihazlar stil sayfası. Bileşenler
açık kullanıcı geçersiz kılmalarını çözümleyin, ardından çıktıyı oluşturun ve ardından paket varsayılanlarını çözün. Rofi bir projeksiyon değildir: `argvus-launcher`, dosyalarının sahibidir ve kullanıcı, `data/rofi/` altında canlı olarak kopyalar.
Normal tema/yeniden yükleme projeksiyonu hiçbir zaman oluşturulan veya paketlenmiş dosyaları desteklemez
bir kullanıcı geçersiz kılma işlemine. Dokümantasyon ve araçlar mantıksal kaynağı tanımlamalıdır
kullanıcılardan oluşturulan çıktıyı düzenlemelerini istemek yerine.

`argvus-appearance` bu nedenle tema ve vurgu orkestrasyonunun sahibidir.
`argvus-config` kanonik duruma ve yapılandırılmış projeksiyona sahiptir.
`argvus theme-switch` (`argvus config theme-apply` işlemi) ve `argvus config accent` aracılığıyla doğrulama, kilitleme ve taahhüt etme
`argvus-config` mevcut olduğunda uyumluluk tüketicilerini uzlaştırın
`config.json` doğrudan okunamıyor. Bu adaptörler
yerel Qt6ct dosyası, GTK `settings.ini` ve `gsettings` ipuçları, terminal ve
sistem monitörü uygulamaları, ayak, süper dosyanın kendi yapılandırması
dizin ve ön kimlik doğrulama karşılayıcısı. Yeni bir adaptöre ihtiyaç duyan bir yardımcı
oraya eklemelisiniz, asla `generated/`'nun içine eklemeyin.

Çalışma zamanı sahipliği kasıtlı olarak sorumluluklara göre bölünmüştür: Rust,
kanonik depo, şema doğrulama, geçişler, yol modeli, kilitler, atomik
yazma, projeksiyon karmaları ve yapılandırılmış oluşturucular. Kabuk yalnızca şu şekilde kalır:
gibi harici tüketiciler için küçük uyumlu giriş noktaları veya adaptörler
`hyprctl` ve varlığa ait tema komut dosyaları. Python testler ve denetimler için ayrılmıştır
ve geliştirme yardımcıları; kritik masaüstü çalışma zamanının bir parçası değildir.

`argvus config apply` yeniden oluşturma sınırıdır (`project`, yeniden oluşturma sınırıdır)
uyumluluk yazımı). Kanonik olduğunda bile eksik çıktıları yeniden oluşturur.
belge değişmez, projektörleri hazırlamada işler ve atomik olarak değiştirir.
`data/` ağacı yalnızca her projeksiyon başarılı olduktan sonra. Oturum yaşam döngüsünün sahibi olmaya devam eder
Hedeflenen yeniden yüklemeleri planlamak için bu bildirimi kullanan `argvus sessionctl`.

Arayan kişi zaten bir nesil taahhüt ettiğinde - `apply-theme`, bir aksan
değişiklik, `set`/`patch` veya widget telemetri bloğu değişikliği — `reload` **geçerlidir**
onu tekrar yansıtmak yerine o nesle. Planı şuradan okuyor:
manifest, hedeflenen tüketicileri geri yükler ve yalnızca manifest
devamsızlık yapıyor veya bekleyen bir işi yok. Bu noktada yeniden projelendirme,
aynı ağacı seçin ve mutasyonun az önce ürettiği planı atın.

Manifest aynı zamanda operasyonel üretimi, etkin konfigürasyonu da kaydeder.
karma, değişen alanlar ve oluşturulan dosya listesi. `data/internal/generation.json`
tek kullanımlık yerel projeksiyon işaretleyicisidir ve asla kullanıcının tercihi değildir.
`argvus config status` bu kayıtları çözümlenen yapılandırmayla karşılaştırır
ve eksik veya fazladan oluşturulmuş dosyaları bildirir.

Mutasyonlar aynı motor sınırını takip eder: `UserConfig` bir
`NextUserConfig`, doğrulandı, `EffectiveConfig`'ya çözümlendi, yansıtıldı ve
ancak o zaman ısrar etti. `.config-transaction.json` operasyonel kurtarma durumudur;`argvus config recover` önceki kanonik değeri deterministik olarak yeniden yansıtır
`data/` ve `config.json` yayınlama arasındaki kilitlenme sonrasında belge. Yeniden yükleme:
yalnızca taahhütten sonra talep edilir ve yeniden yükleme hatası,
tutarlı kalıcı yapılandırma.

## Çözünürlük modeli

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```

Kesin adaylar bileşene göre değişir, ancak prensip tutarlıdır: Kullanıcı geçersiz kılma kazanır, oluşturulan çıktı türetilmiş durumdur ve paketlenmiş dosyalar salt okunur girdiler olarak kalır. Oturum `ARGVUS_CONFIG_HOME` ve `ARGVUS_SYSTEM_CONFIG`'yu dışa aktarır, böylece yardımcılar ve kullanıcı hizmetleri aynı kökleri çözer.

Yeniden yükleme kurtarma işlemi `recover → ensure → migrate → apply → runtime reload`'yu takip eder,
ve gelen bildirim zaten bir durumu tanımladığında yansıtma adımı atlanır.
kararlı nesil. `ensure`
`config.json`'yu şema varsayılanlarından yalnızca profil
eksik ve açık `false`, `0`'yu değiştirmeden eksik alanları doldurur veya
özel değerler. Mevcut hatalı biçimlendirilmiş bir belge, bir hata olmaktan ziyade bir hata olarak kalır.
sessizce değiştiriliyor.

Kalıcı mutasyonlar çözümlenmiş eski ve yeni konfigürasyonları karşılaştırır.
`ChangedDomains`. Taahhütten sonra, bir yeniden yükleme isteği gönderilir
`argvus sessionctl`; yeniden yükleme hatası yuvarlanmadan raporlanır ve kaydedilir
taahhüt edilen yapılandırmayı geri alın. Oturum başlatma, kurtarmayı daha önce gerçekleştirir
önyükleme materyalizasyonu ve projeksiyonu, böylece tüketiciler bir
işlemden vazgeçildi.

| Devlet | Amaç |
| --- | --- |
| `config.json` | Kanonik yönetilen mantıksal tercihler; `generated/`'nun altındaki her şey için tek kaynak. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Eski geçiş girdileri; çalışma zamanı kaynakları değil ve yeniden oluşturulmadı. |
| `.wallpaper-custom` | `appearance.wallpaper` için eski geçiş girişi; çalışma zamanı kaynağı değil. |
| `.spaces`, `.borders` | Eski düzen geçiş girişleri; kanonik değerler config.json'da `layout` altında saklanır. |
|  `config.json` `fonts.conf` | Kurallı yazı tipi seçimlerinin uyumluluk projeksiyonu. |
| `generated/effects` | Kanonik efektlerin uyumluluk projeksiyonu; tek kullanımlık. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Tahmin bildirimi: değişen bölümler, etkilenen tüketiciler ve yeniden yüklemenin gerekli olup olmadığı. `reload` tarafından okundu; asla gerçeğin kaynağı değildir. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Eski telemetri bloğu tercihleri. Yalnızca geçiş yedeği olarak okuyun; kanonik değerler `/control_panel/widget_telemetry_blocks`'da yayınlanır ve dosya artık yazılmaz. |

Bir özellik, `config.json`'dan okunduğunda ve yansıtıldığında kanoniktir.
orada. Bir özellik yansıtıldığında türetilir ancak aynı zamanda bir malzeme olarak tüketilebilir.
geçiş girişi. Bir bileşenin çalışma zamanında yazdığı her şey (önbellek, yerel)
Karşılama durumu dosyası olan `$XDG_CONFIG_HOME/<app>` altındaki config dosyası bir bağdaştırıcıdır
hedeftir ve yapılandırma olarak tekrar okunmamalıdır.

Projeksiyon, Hyprland, GTK/Qt, Waybar, Quickshell, terminaller, bildirimler, kilit ekranı ve yerel uygulamalara yönelik tüketicileri kapsar. Hyprland, seçilen temadaki etkin ve etkin olmayan kenarlık renklerini okur; açık bir `appearance.accent_custom=true`, aktif/grup sınırını kullanıcı aksanıyla geçersiz kılan tek durumdur. Tema değiştirme, bu değerleri derleyicinin yeniden yüklenmesi yoluyla yeniden uygular, böylece önceki temanın renkleri korunamaz. Kullanıcı yapılandırması yeniden oluşturulursa efektler varsayılan olarak animasyonların etkinleştirilmesine ayarlanır. `generated/`'nun silinmesi ve `argvus config apply`'nun çalıştırılması, onu `config.json`'dan bayt bayt yeniden oluşturur; oluşturulan dosyaları doğrudan düzenlemek dayanıklı değildir.
Çalışma zamanı teşhisi, tam yürütme tuple'ını tanımlamalıdır: ikili yol,
sürüm/revizyon, varlık kökü, yapılandırma kökü, veri kökü, oluşturulan kök, dahili
kök ve projeksiyon manifestosu. `status --verbose` bu tanımlama grubunu ortaya çıkarır ve
yayınlanan nesli/karmayı mevcut çözümlenmiş yapılandırmayla karşılaştırır.
Geliştirme ve kurulu paket testleri ayrı modlardır; ikisi de düşmeyebilirilgisiz bir yürütülebilir dosyaya veya geliştiricinin evine sessizce geri dönün. `apply`:
ayrıca eski birleştirilmiş tema tanımlayıcıları için mutabakat sınırı: a
`argvus-dark-spaced` gibi bir değer `theme=argvus-dark` olur ve
`variant=spaced` çözümlenmeden önce. Paketlenmiş varlık kimliği bu sürümde bir bildirime kaydedilmediğinden, onu açıklayan hiçbir tanılama dosyası yoktur.