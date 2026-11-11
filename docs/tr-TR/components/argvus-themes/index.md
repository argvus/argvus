---
title: Temalar
description: ARGVUS temalarını yükleyin, seçin, özelleştirin, içe ve dışa aktarın.
---

ARGVUS'taki bir tema, masaüstünde kullanılan renkleri, yüzeyleri, duvar kağıdı varsayılanlarını ve düzen geometrisini ayarlar: Hyprland, Waybar, Kontrol Paneli, görev çubuğu, kilit ekranı, bildirimler ve ARGVUS uygulamaları. Bu kılavuz, kullanıcı olarak temalarla çalışmak için ihtiyacınız olan her şeyi kapsar.

## Bir temaya neler dahildir?

Her resmi tema, iki modla eksiksiz bir görsel kimliktir:

- **Yapışkan** — küçük pencere boşluklarına sahip, dış boşlukları olmayan ve ekranın kenarına yakın bir görev çubuğuna sahip kompakt bir düzen.
- **Aralıklı** — görev çubuğu ve kabuk yüzeyleri çevresinde daha büyük boşluklara, yuvarlatılmış köşelere ve kenar boşluklarına sahip geniş bir düzen.

Bir temanın seçilmesi onun kimliğini değiştirmez: palet, varsayılan vurgu rengi ve duvar kağıdı ilişkisi modlar arasında aynı kalır. Mod ayrı bir ayardır; bkz. [Resmi temalar](/docs/components/argvus-themes/official-themes/).

## Bu kılavuzdaki sayfalar

- [Temaları yükleme](/docs/components/argvus-themes/installing-themes/) — resmi temaların nereden geldiği ve `pacman` ile bunlardan birinin veya tamamının nasıl kurulacağı.
- [Resmi temalar](/docs/components/argvus-themes/official-themes/) — katalog, her temanın varsayılan vurgu rengi ve bir temanın ve modunun nasıl seçileceği.
- [Vurgu rengini özelleştirme](/docs/components/argvus-themes/highlight-color/) — vurgu rengini temadan bağımsız olarak değiştirin.
- [Temaları içe ve dışa aktarma](/docs/components/argvus-themes/import-and-export/) — Etkin temayı, vurgu renginiz ve duvar kağıdınızla birlikte kendi temanız olarak dışa aktarın ve başka bir makineye içe aktarın.

## Hızlı başvuru

| Görev | Nerede |
| --- | --- |
| Bir tema seçin | **Kontrol Merkezi → Görünüm → Temalar → Resmi Temalar** |
| Yapışkan ve Aralıklı Anahtar | **Kontrol Merkezi → Görünüm → Mod** |
| Vurgu rengini değiştirme | **Kontrol Merkezi → Görünüm → Rengi vurgula** |
| Etkin temayı dışa aktar | **Kontrol Merkezi → Görünüm → Temalar → Temayı dışa aktar** |
| Bir temayı içe aktar | **Kontrol Merkezi → Görünüm → Temalar → Temayı içe aktar** |
| Klavyeden tema seçiciyi açın | `SUPER + SHIFT + T` |

## Temalar nereden geliyor?

ARGVUS, `argvus-appearance` paketiyle birlikte **ARGVUS Dark** ve **ARGVUS Light** olmak üzere iki yerleşik tema sunar. Diğer tüm resmi temalar ayrı bir `argvus-theme-<name>` paketidir. Bir tema paketinin yüklenmesi onu tema seçiciye ekler; oturumun yeniden başlatılmasına gerek yoktur. Dışa aktarılan veya içe aktarılan kendi temalarınız **Özel Temalar** olarak ayrı olarak listelenir.

## Klavye ipuçları

Kontrol Merkezi tema sayfaları, anahtarlarını alt bilgide gösterir:

- `↑` / `↓` — gezinin.
- `Enter` — uygula veya aç.
- `e` — Etkin temayı Temalar sayfasında dışa aktarın.
- `i` — Temalar sayfasında bir temayı içe aktarın.
- `d` — özel bir temayı silin.
- `Esc` — geri dönün.

## Sorun Giderme

- **Yüklediğim bir tema görünmüyor.** Paketin `pacman -Q argvus-theme-<name>` ile yüklendiğini doğrulayın ve ardından Kontrol Merkezini yeniden açın. Paket, `argvus-appearance` mevcut olacak şekilde kurulmalıdır.
- **Temayı değiştirdikten sonra vurgu rengi değişti.** Özel bir vurgu rengi, tema geçişlerinde geçerliliğini korur. Temaya kendi rengini geri vermek için **Tema varsayılanına sıfırla** seçeneğini kullanın.
- **İçe aktarma için bir tema arşivi listelenmemiştir.** Doğrudan ana dizininizde bulunmalı ve `<id>/theme.toml`'ya sahip olmalıdır. Bkz. [Temaları içe ve dışa aktarma](/docs/components/argvus-themes/import-and-export/).