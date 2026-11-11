---
title: ARGVUS Saati
description: Saat, dünya saatleri, kronometre, geri sayım sayacı ve sesli alarmlar.
---

ARGVUS Clock, geçerli saati ve zaman araçlarını içeren bir GTK 4 uygulamasıdır: dünya saatleri, kronometre, geri sayım sayacı ve alarmlar. Başlatıcıdan, görev çubuğundaki saate tıklayarak **SUPER + Shift + C** veya şunu kullanarak açın:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```

ARGVUS Saat, görev çubuğuyla aynı hizada, 4 piksel boşlukla açılır bir pencere olarak açılır. Görev çubuğu kök tıklama koordinatlarını iletir, böylece açılır pencere tıklanan monitörü kullanır; koordinatlar olmadan başlatmak işaretçi konumunu kullanır. İşaretçi 5 saniye boyunca açılır pencerenin dışında kalırsa otomatik olarak kapanır; İşaretçiyi açılır pencereye taşımak geri sayımı iptal eder. Ana sekmedeki **Esc** yine de onu hemen gizler ve tam sayfada **Esc** önceki sekmeye döner. `argvus --clock` bir takma addır. Görev çubuğundaki tarihe tıklamak yine de takvimi açar; ARGVUS Saati olayları göstermez.

ARGVUS Focus, ARGVUS Clock ve Kontrol Paneli aynı ekran kenarını paylaşır. Saatin açılması önce Odak ve Kontrol Panelini kapatır; bu yüzeylerden herhangi birinin açılması Saati kapatır.

## Sayfa

- **Saat**: büyük haneli saat (varsayılan olarak 24 saat, ayarlarda 12 saat, isteğe bağlı olarak saniye), yerelleştirilmiş tarih, yerel saat dilimi ve bunun UTC farkı ve bir sonraki alarm.
- **Dünya**: Bir şehir veya IANA saat dilimi aranarak eklenen şehirler (vurgular ve büyük/küçük harf dikkate alınmaz). Her biri kendi saatini, yerel saat farkını ve orada dün, bugün veya yarın olup olmadığını gösterir. Yaz saati uygulaması IANA saat dilimi veritabanından gelir; hiçbir şey sabit ofsetlerle hesaplanmaz. Şehirler yeniden sıralanabilir ve kaldırılabilir.
- **Kronometre**: başlatma, duraklatma, devam ettirme, sıfırlama ve turlar (tur süresi ve toplam).
- **Zamanlayıcı**: saat, dakika ve saniye veya ön ayarlar 5, 10, 15, 25, 30 ve 60 dakika. Duraklatın, devam ettirin, yeniden başlatın ve iptal edin. Bittiğinde alarm gibi çalıyor. Bu zamanlayıcı ARGVUS Focus oturumlarından bağımsızdır.
- **Alarmlar**: alarmları oluşturun, düzenleyin, açın, kapatın ve silin. Her birinin bir saati, isteğe bağlı bir adı, tekrarladığı hafta içi günleri (hiçbiri bir kez çaldığı anlamına gelmez), bir sesi ve bir erteleme süresi vardır.

Klavye: **Ctrl+1** ile **Ctrl+5** arasında geçiş yapar, **Ctrl+N** bir alarm oluşturur, **Ctrl+,** ayarları açar, **Esc** geri döner, **Ctrl+W** pencereyi kapatır ve **Ctrl+Q**'dan çıkar.

## Alarm sesi

Alarmlar, siz onları yanıtlayana kadar gerçek bir ses çıkarır:

- Sesler: **Chime** (varsayılan) ve **Beacon**, `/usr/share/argvus/sounds/` ile gönderilir.
- **Alarm ses seviyesi** (10 ila 100, varsayılan 80) ve **Alarm sesini dinle** ayarlardadır; editörün seçilen ses için bir test düğmesi vardır.
- Çalan bir alarm, ekranın üst kısmında, diğer pencerelerin üzerinde bir kart ve hem **Erteleme** hem de **Kapat** ile bir bildirim gösterir. Kart odaklandığında **Enter** kapanır ve **S** erteler.
- Alarmın erteleme süresi dolduktan sonra erteleme tekrar çalar (1 ila 60 dakika, varsayılan 10 dakika).
- Yanıtlanmayan bir alarm **Şu süreden sonra zili durdur** süresinden sonra durur (5, 10, 15 veya 30 dakika, varsayılan 10) ve bir bildirim bırakır.
- Birlikte çalan iki alarmın her ikisi de listelenir; ikincisi asla birinciyi susturmaz.
- Ses çalınamıyorsa, kart ve pencere bunu hatayla birlikte belirtir ve hata günlüğe kaydedilir. ARGVUS Saati hiçbir zaman çalınmayan bir sesi bildirmez.

Oynatma, GTK'nın medya desteğini (GStreamer) kullanır, dolayısıyla PipeWire, PulseAudio veya ALSA ile çalışır. Paket, `gst-plugins-base` (Ogg Vorbis) ve `gst-plugins-good`'ya (ses çıkışı) bağlıdır.

## Alarmlar çaldığında

Alarmlar yalnızca oturumunuzda ARGVUS Saati çalışırken çalar:

- Pencerenin kapatılması, alarmlar, çalışan bir zamanlayıcı veya çalışan bir kronometre ihtiyaç duyduğunda ARGVUS Saatinin çalışmasını sağlar (**Pencere kapatıldığında çalışmaya devam et**, varsayılan olarak açıktır). Hiçbir şey beklemede kalmayınca sonlandırılıyor.
- **Oturumla başla**, oturum açtıktan sonra argvus clock --background'yu çalıştıran `argvus-clock.service` kullanıcı birimini etkinleştirir. Varsayılan olarak kapalıdır.- Oturumunuz kapalıyken, bilgisayar kapalıyken veya askıya alınmışken alarmlar çalamaz.
- 5 dakikaya kadar geç fark edilen bir alarm hala çalıyor. Bundan sonra (saat çalışmıyordu ya da sistem uykudaydı ya da kapalıydı) çalmıyor: bir bildirimde ve Alarmlar sayfasında **cevapsız** olarak rapor ediliyor. Tekrarlanan bir alarm ertesi güne kadar devam eder; bir kerelik alarm kapanır.
- Her olay, çalmadan önce işlendiği şekilde kaydedilir, böylece ARGVUS Saatinin yeniden açılması asla aynı alarmı iki kez çalmaz ve oturum başına yalnızca bir örnek çalışır.
- Tekrarlanan alarmlar, seçilen hafta içi günlerde yerel saate göre hesaplanır, böylece gün ışığından yararlanma değişikliklerine göre zamanlarını korurlar. Aralığın ardından ilk dakikada yay değişim halkalarının atladığı süre; iki kez gerçekleşen bir zaman ilk kez çalar.
- Bir zamanlayıcı, askıya alma sırasında saymaya devam eder. Sistem uykudayken bittiyse uyandığınızda çalıyor ve saatin ne kadar geç olduğunu söylüyor. ARGVUS Saati kapatıldıktan sonra çalışan bir zamanlayıcı veya kronometre duraklatılmış olarak geri gelir.

## Yollar

| Yol | İçerik |
| --- | --- |
|  `argvus clock --background` `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Ayarlar ve dünya saatleri |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarmlar |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Zamanlayıcı ve kronometre kontrol noktası |
| `/usr/lib/systemd/user/argvus-clock.service` | Oturumla başlayın |
| `/usr/share/argvus/sounds/` | Paketlenmiş sesler |

Dosyalar atomik olarak yazılır; okunamayan bir dosya `<name>.corrupt-<time>` olarak yeniden adlandırılır ve varsayılanlar kullanılır. Renkler etkin ARGVUS temasını takip eder ve rengi vurgular (tema tüketicisi `time`). Dizeler `clock` ve `time` çeviri etki alanlarından gelir.

Görev çubuğu takvimi, ARGVUS Focus, ARGVUS Saati ve Kontrol Paneli birbirini dışlar: birinin açılması diğerlerini kapatır.