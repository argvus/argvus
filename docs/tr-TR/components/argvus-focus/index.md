---
title: ARGVUS Odak
description: Pomodoro ve aralar, bildirimler ve yerel oturum geçmişi içeren odak oturumları.
---

ARGVUS Focus, aralarla ayrılmış odak oturumlarında çalışmaya yönelik bir GTK 4 uygulamasıdır. Görev çubuğunun yanında açılır pencere olarak açılır. Başlatıcıdan görev çubuğu göstergesine tıklayarak **SUPER + Shift + F** veya şunu kullanarak açın:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```

Görev çubuğu kök tıklama koordinatlarını iletir, böylece açılır pencere tıklanan monitörü kullanır ve çubukla hizalanır; koordinatları olmayan bir başlatma, işaretçi konumunu kullanır. İşaretçi 5 saniye boyunca açılır pencerenin dışında kalırsa otomatik olarak kapanır; İşaretçiyi açılır pencereye taşımak, geri sayımı iptal eder. Ana sayfadaki **Esc** yine de onu anında gizler ve alt sayfadaki **Esc** önceki sayfaya döner. Görev çubuğu göstergesi boştayken yalnızca simgesini ve bir aşama sırasında kalan `MM:SS`'yu aşama rengiyle gösterir; duraklatılmış bir aşama kalan süresini korur ve duraklatılmış bir durum ekler. `status` komutu, GTK'yı başlatmadan `argvus` içinde çalışır. `argvus --focus` bir takma addır. Oturum başına yalnızca bir Focus örneği çalıştırılır: ikinci bir başlatma, seçeneklerini çalışana aktarır.

ARGVUS Focus, ARGVUS Clock ve Kontrol Paneli aynı ekran kenarını paylaşır. Odağı açmak önce Saat'i ve Kontrol Panelini kapatır; bu yüzeylerden herhangi birinin açılması Focus'u kapatır.

## Oturum

- **Pomodoro** modu, odaklamayı ve kısa araları değiştirir ve her döngüyü uzun bir arayla bitirir. Varsayılan uzunluklar 25, 5 ve 15 dakikadır ve 4 odaklanma seansından sonra uzun bir ara verilir.
- **Tek oturum** modu, ara vermeden, kendi uzunluğunda (varsayılan olarak 45 dakika) bir odaklanma oturumu çalıştırır.
- Pencere aşamayı (odak, kısa mola, uzun mola, her biri kendi renginde), ilerleme halkası içinde kalan süreyi, döngüdeki konumu ve bugünün toplamını gösterir.
- **Odaklanmayı başlat / Arayı başlat**, **Duraklat**, **Devam Et**, **Oturumu iptal et** (aşamadan vazgeçilir ve hiçbir şey kaydedilmez) ve **Şimdi tamamla** (odaklanma oturumunu erken bitirir ve gerçekte harcanan zamanı kaydeder). Mola sırasında **Molayı atla** tekrar odaklanmaya döner.
- **Otomatik olarak ara vermeyi başlat** veya **Aradan sonra odağı otomatik olarak başlat** açık olmadığı sürece, bir aşama sona erdiğinde bir sonraki aşama sizi bekler.

Klavye: **Boşluk** başlatır veya duraklatır, **Ctrl+,** ayarları açar, **Ctrl+H** geçmişi açar, **Esc** geri döner, **Ctrl+W** pencereyi gizler ve **Ctrl+Q** çıkar.

## Ayarlar

Ön ayarlar (Klasik 25/5/15, Genişletilmiş 50/10/30, Derin çalışma 90/20/30) uzunlukları doldurur ve düzenlenebilir kalır. Ayarlar aynı zamanda uzun bir aradan önceki oturumları, otomatik başlatmaları, bir aşamanın sonundaki bildirimi ve **oturum seslerini** içerir: kendi ses düzeyine sahip kısa bir zil ve **Test sesi** düğmesi. Bu sesler, [ARGVUS Saati](/docs/components/argvus-clock/) alarm seslerinden ayrıdır.

## Tarih

Tamamlanan her odak oturumu, başlangıcı, bitişi ve harcanan süre ile birlikte kaydedilir; Aralar ve iptal edilen seanslar geçerli değildir. Geçmiş sayfası bugünün oturumlarını ve dakikalarını, toplamları ve son oturumları gösterir. 365 günden eski kayıtlar silinir. **Geçmişi temizle** onay ister.

## Pencere, arka plan ve askıya alma

- Bir döngü sırasında pencereyi kapatmak yalnızca onu gizler: oturum devam eder ve aşama sona erdiğinde sizi bilgilendirir. Hiçbir şey yapılmadığından kapanış sonlandırılır.
- Bir aşama çalışırken sistem askıya alınırsa, askıya alma anında aşama duraklatılır ve sistemin ne kadar süre uyuduğunu belirten bir mesaj görüntülenir. Uyumak odaklanma süresinden sayılmaz.
- Oturum her değişiklikte ve her 30 saniyede bir kaydedilir. ARGVUS Focus bir aşamada kapatılırsa (oturum kapatma, kilitlenme, **Ctrl+Q**), bir sonraki açılışında o noktada duraklatılmış olarak geri gelir; asla bildirimde bulunamadığı sürece zamanlayıcının çalışmaya devam ettiğini iddia etmez.
- Süreler sistemin önyükleme zamanı saatinde ölçülür, dolayısıyla duvar saatini veya saat dilimini değiştirmek bir aşamayı kısaltmaz veya uzatmaz.## Yollar

| Yol | İçerik |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Ayarlar |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Tamamlanan oturumlar |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Geçerli oturumun denetim noktası |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Oturum zili |

Dosyalar atomik olarak yazılır. Okunamayan bir dosya `<name>.corrupt-<time>` olarak yeniden adlandırılır ve varsayılanlar kullanılır; bir mesaj size nereye gittiğini söyler.

Renkler etkin ARGVUS temasını takip eder ve rengi vurgular (tema tüketicisi `time`). Dizeler `focus` ve `time` çeviri etki alanlarından gelir.

Görev çubuğu takvimi, ARGVUS Focus, ARGVUS Saati ve Kontrol Paneli birbirini dışlar: birinin açılması diğerlerini kapatır.