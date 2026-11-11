---
title: Resmi temalar
description: Resmi ARGVUS tema kataloğu, varsayılan vurgu renkleri ve bir temanın ve onun Yapışkan veya Aralıklı modunun nasıl seçileceği.
---

Bu sayfa, tema paketleri olarak mevcut olan resmi temaları listeler ve bunlardan birinin nasıl seçileceğini açıklar. Bunları yüklemek için, bkz. [Temaları yükleme](/docs/components/argvus-themes/installing-themes/).

## Bir tema seçin

1. **Kontrol Merkezi → Görünüm → Temalar**'ı açın.
2. **Resmi Temalar**'ı seçin.
3. **Koyu** veya **Açık**'ı seçin.
4. Tema ailesini seçin. Tema, seçtiğiniz anda uygulanır.

Seçilen tema tüm masaüstüne aynı anda uygulanır: çalışan görev çubuğu, paneller, bildirimler, pencereler ve duvar kağıdı, yükleme ekranı olmadan ve yeniden başlatmaya gerek kalmadan yerinde renk değiştirir. ARGVUS, tema kendi duvar kağıdı ilişkilendirmesiyle uygulanmadığı sürece duvar kağıdı seçiminizi korur.

Tema seçiciyi klavyeden açmak için `SUPER + SHIFT + T` tuşuna da basabilirsiniz. Kategoriler arasında gezinmek için ok tuşlarını, seçmek için `Enter`'yu ve kapatmak için `Esc`'yu kullanın.

## Yapışkan ve Aralıklı

Her tema iki modda çalışır. Mod temadan bağımsızdır: temayı değiştirmek geçerli modu korur, modu değiştirmek de geçerli temayı korur.

| Modu | Pencere boşlukları | Dış boşluk | Pencere yuvarlama | Görev çubuğu ve kabuk kenar boşluğu |
| --- | ---: | ---: | ---: | ---: |
| Yapışkan | `2` | `0` | `0` | kenara yakın |
| Aralıklı | `10` | `18` | `4` | `18` |

Modu değiştirmek için **Kontrol Merkezi → Görünüm → Mod**'u açın ve **Yapışkan** veya **Aralıklı**'yı seçin. Bu düzen değerlerinde yaptığınız manuel ayarlamalar, bir sonraki açık tema seçimine kadar geçerli kalır.

## Karanlık temalar

| Tema | Paket | Varsayılan vurgulama |
| --- | --- | --- |
| Bir Karanlık | `argvus-theme-one-dark` | `#61AFEF` |
| Drakula | `argvus-theme-dracula` | `#BD93F9` |
| Gümüş Koyu | `argvus-theme-silver-dark` | `#595959` |
| Arduvaz Koyu | `argvus-theme-slate-dark` | `#7391A5` |
| Evren | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox Yüksek Koyu | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Koyu | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Gül Çamı | `argvus-theme-rose-pine` | `#C4A7E7` |
| Tokyo Gecesi | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Yalnızlık | `argvus-theme-solitude` | `#798186` |
| Gün batımı | `argvus-theme-sunset` | `#E2BE8A` |
| Matris | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Koyu | `argvus-theme-monokai-dark` | `#78DCE8` |
| Bayan Sis | `argvus-theme-lady-mist` | `#354B78` |

## Işık temaları

| Tema | Paket | Varsayılan vurgulama |
| --- | --- | --- |
| GitHub Işık | `argvus-theme-github-light` | `#0969DA` |
| Solarize Işık | `argvus-theme-solarized-light` | `#268BD2` |
| Kuzey Işık | `argvus-theme-nord-light` | `#5E81AC` |
| Bir Işık | `argvus-theme-one-light` | `#4078F2` |
| Everforest Işığı | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Işık | `argvus-theme-gruvbox-light` | `#458588` |
| Suluboya | `argvus-theme-watercolor` | `#1A639C` |
| Obsidiyen Işık | `argvus-theme-obsidian-light` | `#334A78` |

## Yerleşik temalar

**ARGVUS Dark** ve **ARGVUS Light**, `argvus-appearance` tarafından sağlanır ve her zaman **Resmi Temalar** altında mevcuttur. ARGVUS Dark, `#3590BD` varsayılan vurgusunu kullanır; ARGVUS Light, `#181818`'yu kullanır.

## Bir temanın kontrol ettiği şeyler

Tema, masaüstü bileşenleri tarafından kullanılan paleti ve yüzey işlemini, varsayılan vurgu rengini, varsayılan duvar kağıdı ilişkisini ve şeffaflık ve bulanıklık varsayılanlarını tanımlar. Masaüstü bileşenleri kendi kopyalarını saklamak yerine bu durumu okur, böylece bir tema değişikliği besteciye, görev çubuğuna, Denetim Masasına, kilit ekranına, bildirimlere ve uygulamalara birlikte ulaşır.Varsayılan vurgu rengi yalnızca başlangıç ​​noktasıdır. Bunu altı basamaklı herhangi bir renkle değiştirebilirsiniz, bkz. [Vurgu rengini özelleştirme](/docs/components/argvus-themes/highlight-color/).