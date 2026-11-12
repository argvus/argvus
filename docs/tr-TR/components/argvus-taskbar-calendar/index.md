---
title: Takvimi
description: ARGVUS görev çubuğu takvimini kullanın.
---

`argvus-taskbar-calendar`, Waybar tarih modülünden veya aşağıdakilerle açılan GTK4 takvim açılır penceresidir:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

Yerel etkinlikleri, hatırlatıcıları ve ICS içe/dışa aktarımını destekler. Kullanıcı hizmeti `argvus-taskbar-calendar.service`'dur. Yüklü sürüm tarafından desteklenen seçenekler için `argvus taskbar-calendar --help` kullanın.

Takvim, Yapışkan ve Aralıklı değişkenler de dahil olmak üzere aktif ARGVUS temasını takip ediyor. Tema değişiklikleri, oluşturulan kullanıcı önbelleği aracılığıyla uygulanır; Çalışan bir açılır pencerenin açık bir şekilde yenilenmesi gerekiyorsa `argvus taskbar-calendar reload` kullanın.

Görev çubuğu tarih modülünden açıldığında açılır pencere çubuğun kenarına sabitlenir: tarih modülünün sağ kenarına hizalanmış olarak üst görev çubuğunun altında ve alt görev çubuğunun üzerinde açılır. Görev çubuğu geometrisi olmayan doğrudan başlatmalar, işaretçiyi veya monitör geri dönüşünü kullanır.

## Komutlar

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```

## Etkinlikler ve hatırlatıcılar

Etkinlikler varsayılan olarak kapalıdır. **Ayarlar > Etkinlikler** altındaki açılır pencereden bunları açın; the
seçim durum dizininde tutulur ve ayrıca etkinleştirilir ve başlatılır
Hatırlatıcıları gösteren `argvus-taskbar-calendar.service`. Bunları kapatmak şunları durdurur:
hizmet.
- **+ OLAY** (veya `n`) seçilen güne ait düzenleyiciyi açar. Etkinlikler oluşturulabilir ve
  bugünden itibaren değişti; geçmiş bir güne ait bir etkinlik hâlâ açılıp silinebilir.
- Yeni bir etkinlik saat 09:00'da başlar ve Ayarlar'da belirlenen varsayılan süre kadar sürer. Taşıma
  başlangıç zamanı uzunluğu korur; başlangıç zamanından daha erken bir bitiş zamanı, etkinlik anlamına gelir
  ertesi gün biter.
- Durum ne olursa olsun, her etkinlik başlangıcında sesli bir bildirim gösterir.
  **Hatırlatıcı** ve **Tekrar** kutuları şunu söylüyor. Başladıktan sonra kaydedilen bir etkinlik (tüm gün süren bir etkinlik)
  örneğin gün içerisinde oluşturulan bir olay) 30 saniye içinde bildirim yapar.
- **Hatırlatma**, başlangıçtan önce seçilen süreye (saatte) daha erken bir bildirim ekler.
  bu süre geçtikten sonra başlayın).
- Tüm gün süren bir etkinlikte zaman alanı **BİLDİRİM** olur: etkinlik tüm günü kapsar
  gün ve o saatte bildirim yapar (00:00 gün başında bildirim yapar). Zaman
  etkinlik yeniden açıldığında saklanır.
- Tüm gün süren etkinlikler için **Tekrar**, o saatten sonra seçilen aralıkta tekrar bildirim gönderir.
- Bir etkinliğin başlangıcını değiştirmeden düzenlemek, halihazırda gösterilen hatırlatıcıların tekrarlanmasına neden olmaz.
- Bildirimler takvim dilini takip eder (Ayarlar > Dil).
- Bir gün, bir günün her günü de dahil olmak üzere, üzerinde bir etkinlik gerçekleştiğinde bir nokta (`•`) gösterir.
  gecelik veya çok günlük etkinlik. Günler yerel saat dilimini takip eder.
- Yerel etkinlikler bittikten 10 dakika sonra otomatik olarak kaldırılır. Bir etkinlik zaten varsa
  kaydedildiğinde sona erer, kaydedildikten sonra en az 10 dakika boyunca kullanılabilir durumda kalır.

İçe aktarılan `.ics` dosyaları tarihlerini, saat dilimlerini (`TZID`) ve alarmlarını (`VALARM`) korur.
İçe aktarılan olaylar, kendi alarmlarının yanı sıra başlangıçta da bildirim verir. Tekrarlanan etkinlikler (`RRULE`)
saklanır ve dışarı aktarılır, ancak yalnızca ilk oluşumları gösterilir ve hatırlatılır.
Bir paket güncellemesinden sonra hatırlatma hizmetinden çıkar ve systemd yeni sürümü başlatır
30 saniye içinde. `sync` yalnızca yapılandırılmış uzak takvimleri bildirir; CalDAV senkronizasyonu değil
henüz uygulanmadı.

## Yollar

- Yapılandırma: `/etc/argvus/taskbar/calendar/config.toml` (sistem genelinde; varsayılan, paketle birlikte gönderilir).
- Stiller: `/etc/argvus/taskbar/calendar/style.css` ve `theme.css`; Aktif ARGVUS temasının renkleri aşağıdaki önbellek aracılığıyla tema yapısından gelir.
- Veri: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (veya `~/.local/share/argvus-taskbar-calendar/`).
- Durum: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (veya `~/.local/state/...`); olaylar geçişi orada tutulur.
- Önbellek: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (veya `~/.cache/argvus-taskbar-calendar/`).

Vurgu rengi `~/.config/argvus/data/.accent-color`'dan okunur ve çalışma zamanında uygulanır
Temanın renklerinin üstünde.

Otomatik dil seçimi, varsayılan olarak İngilizce olmak üzere `locale.conf` kullanıcısını takip eder. Açık takvim dili tercihleri ​​hâlâ otomatik seçimi geçersiz kılmaktadır.Görev çubuğu takvimi, ARGVUS Focus, ARGVUS Saati ve Kontrol Paneli birbirini dışlar: birinin açılması diğerlerini kapatır.

Takvim okuma hataları, boş ajandadan ayrı olarak görüntülenir. Veritabanının açılması, günlük olayların yüklenmesi ve aylık göstergelerin yüklenmesinin ayrı mesajları vardır. Yeniden denemek için tekrar bir gün seçin. Başarısız bir yenileme, önceden yüklenen verileri yalnızca aynı gün veya ay için korur; diğer sorgunun geçerli sonuçları mevcut kalır.

## Paylaşılan dil katalogları

Arayüz, on dilin tümü için paylaşılan `argvus-i18n` kataloglarını kullanır. Metinler ana çalışma alanındaki `resources/locales/<locale>/taskbar-calendar.json`'da saklanır ve `argvus` tarafından yüklenir. İngilizce geri dönüştür; yeni diller artık yerel iki dilli bir sözlüğe ihtiyaç duymuyor.
