---
title: Çıkarılabilir cihazlar
description: ARGVUS'tan çıkarılabilir depolamayı bağlayın ve yönetin.
---

`argvus-removable-devices`, UDisks2'yi izler ve ARGVUS Görev Çubuğu depolama modülünü ve cihaz menüsünü sağlar. Menü ayrıca Rofi tarzı ayrı bir arayüz olarak da mevcuttur, böylece aynı cihaz eylemleri görev çubuğundan veya başlatıcı iş akışından kullanılabilir.

Kullanılabilir eylemler şunlardır: `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` ve `copy`. `list`, `watch` ve `menu` komutları, entegrasyonlar için ilgili durumu, görev çubuğu akışını ve menü arayüzlerini ortaya çıkarır.

## Görev çubuğu menüsü

`"mode": "gui"` (sistem varsayılanı) ile görev çubuğu depolama simgesine tıklandığında, takvim açılır penceresi gibi ARGVUS Görev Çubuğuna eklenen bir GTK menüsü açılır: görev çubuğu ekranın üst kısmındayken çubuğun altında ve görev çubuğu altta olduğunda çubuğun üstünde. Her aygıt satırında yalnızca durumunun izin verdiği eylemleri içeren bir eylem alt menüsü bulunur.
- **İşaretçi:** Bir aygıt satırının üzerine gelindiğinde veya tıklandığında alt menüsü açılır. İşaretçi menünün kenarlarını geçerken veya menülerden ayrılırken alt menü açık kalır ve yalnızca işaretçi orada kısa bir süre durduktan sonra başka bir satırı takip eder, dolayısıyla alt menüye giden çapraz yol onu kapatmaz. Her iki menünün dışına yapılan bir tıklama açılır pencereyi kapatır.
- **Klavye:** `Up`/`Down` bir satır seçin, `Right`, `Enter` veya `Space` alt menüyü açın (veya **Yenile**/**Çık** komutunu çalıştırın), `Enter` veya `Space` seçilen eylemi çalıştırır, `Left` veya `Escape` cihaz listesine geri döner ve `Escape` orada açılır pencereyi kapatır.

**Yolu kopyala**, `copy_command` (varsayılan `wl-copy`, `wl-clipboard`'dan) aracılığıyla bağlama noktasını panoya koyar; böylece yol, menü kapandıktan sonra da orada kalır; `show_notification` açık olduğunda bir bildirim bunu doğrular. **Kilit**, kilidi açılmış her şifrelenmiş birim için sunulur ve gerektiğinde ilk olarak bu birimin bağlantısını kaldırır.

Varsayılan dosya yöneticisi eylemi, ARGVUS Kontrol Merkezi'nde seçilen dosya yöneticisini takip eder. Bu, çıkarılabilir cihaz eylemlerinin kullanıcının uygulama varsayılanlarıyla tutarlı olmasını sağlar.

Sistem yapılandırması `/etc/argvus/removable-devices/config.json`'dur. ARGVUS tarafından yönetilen kullanıcı başına projeksiyon `~/.config/argvus/data/removable-devices/config.json`'dur ve yanında eşleşen bir `theme.css` bulunur. Bu stil sayfası aynı zamanda standart tema, vurgu ve moddan `~/.config/argvus/data/generated/removable-devices/theme.css`'ya yansıtılır ve özel bir çıkarılabilir cihaz varlığı göndermeyen temalar için bir aile/varyant geri dönüşü sağlanır, böylece bir tema değişikliği modülün herhangi bir manuel adım olmadan tutarlı kalmasını sağlar. Eski bileşen yolları geçiş girdileridir; daha sonraki kullanıcı yapılandırması sistem varsayılanlarını geçersiz kılar.

Kullanışlı ayarlar arasında hiçbir cihaz mevcut olmadığında modülün gizlenip gizlenmeyeceği (`hide_when_empty`), adların ve kapasitelerin gösterilip gösterilmeyeceği, cihaz sıralama düzeni, bildirim davranışı, menü arka ucu ve durum simgeleri yer alır. Stil sayfası aktif ARGVUS temasından (tema yapısının `removable-devices/theme.css`'su) oluşturulur ve her tema veya aksan değişikliğinde panelin `theme.css`'suna kopyalanır.

Arayüz, $XDG_CONFIG_HOME/locale.conf'daki `LANG` kullanıcı dilini kullanır ve yapılandırılmamış bir kullanıcı için varsayılan İngilizcedir. Devralınan yerel ayar değişkenleri arayüz dilini seçmez.

## Paylaşılan dil katalogları

Arayüz, on dilin tümü için paylaşılan  `$XDG_CONFIG_HOME/locale.conf` `argvus-i18n` kataloglarını kullanır. Metinler ana çalışma alanındaki `resources/locales/<locale>/removable-devices.json`'da saklanır ve `argvus` tarafından yüklenir. İngilizce geri dönüştür; yeni diller artık yerel iki dilli bir sözlüğe ihtiyaç duymuyor.