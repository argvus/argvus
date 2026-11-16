---
title: Görünüm
description: ARGVUS bileşenleri tarafından paylaşılan görsel dili yapılandırın.
---

Görünüm sistemi Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock ve ARGVUS uygulamalarına bir tema durumu uygular.

## Tema aileleri

Aşağıdaki yirmi üç aile mevcut tema yüküne göre tanımlanır. Renk sütunu, ailenin varsayılan aktif kenarlık/vurgu rengidir; kullanıcılar başka bir geçerli RGB aksanı seçebilir.

| Aile | Vurgu rengini sıfırla | Temel arka plan | Modlar |
| --- | --- | --- | --- |
| Bir Karanlık | `#61AFEF` | `#282C34` | Yapışkan, Aralıklı |
| Drakula | `#BD93F9` | `#282A36` | Yapışkan, Aralıklı |
| ARGVUS Koyu | `#3590bd` | `#111316` | Yapışkan, Aralıklı |
| Koyu Gümüş | `#595959` | `#111316` | Yapışkan, Aralıklı |
| Koyu Arduvaz | `#7391a5` | `#2f3541` | Yapışkan, Aralıklı |
| Karanlık Evren | `#eeeeee` | `#000000` | Yapışkan, Aralıklı |
| ARGVUS Koyu Gruvbox Yüksek | `#D79921` | `#282828` | Yapışkan, Aralıklı |
| ARGVUS Koyu Gruvbox | `#D4BE98` | `#282828` | Yapışkan, Aralıklı |
| ARGVUS Işık | `#181818` | `#f7f7f7` | Yapışkan, Aralıklı |
| GitHub Işık | `#0969DA` | `#FFFFFF` | Yapışkan, Aralıklı |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Yapışkan, Aralıklı |
| ARGVUS Hafif Gruvbox | `#458588` | `#FBF1C7` | Yapışkan, Aralıklı |
| Gül Çamı | `#C4A7E7` | `#191724` | Yapışkan, Aralıklı |
| Tokyo Gecesi | `#7AA2F7` | `#1A1B26` | Yapışkan, Aralıklı |
| Yalnızlık | `#798186` | `#101315` | Yapışkan, Aralıklı |
| Gün batımı | `#E2BE8A` | `#0F0F0F` | Yapışkan, Aralıklı |
| Matris | `#82FB9C` | `#0B0C16` | Yapışkan, Aralıklı |
| Monokai Koyu | `#78DCE8` | `#2D2A2E` | Yapışkan, Aralıklı |
| Bayan Sis | `#354B78` | `#131313` | Yapışkan, Aralıklı |
| Bir Işık | `#4078F2` | `#FAFAFA` | Yapışkan, Aralıklı |
| Everforest Işığı | `#3A94C5` | `#FDF6E3` | Yapışkan, Aralıklı |
| Solarize Işık | `#268BD2` | `#FDF6E3` | Yapışkan, Aralıklı |
| Kuzey Işık | `#5E81AC` | `#ECEFF4` | Yapışkan, Aralıklı |
| Suluboya | `#1A639C` | `#E9E3D3` | Yapışkan, Aralıklı |

Tabloda gösterilen renk, o aile seçildiğinde veya **Varsayılan temaya sıfırla** kullanıldığında `config.json`'ya yazılan değerdir. Manuel olarak seçilen altı basamaklı bir RGB vurgusu, onu seçilen temanın varsayılanıyla değiştiren bir sonraki tema anahtarına kadar uygulanır.

**Yapışkan** kompakt düzendir: döşemeli pencerelerde küçük iç boşluklar kullanılır, dış boşluklar sıfırdır ve görev çubuğu ekranın kenarına yakın durur. **Aralıklı** görev çubuğu ve kabuk yüzeylerinin çevresine daha büyük boşluklar, yuvarlatılmış köşeler, gölgeler ve kenar boşlukları ekler.

Mevcut `float` düzen değerleri ve `-float` tema tanımlayıcıları, Spaced için eski adlar olarak tanınır. Yeni düzen değerleri ve çözümlenen tema tanımlayıcıları `spaced` ve `-spaced`'yu kullanır.

Mod, geometriyi ve yüzey işlemini değiştirir; altıncı bir renk ailesi oluşturmaz. Seçim için [Temalar](/docs/user-guide/appearance/) ve görev çubuğu ve pencere yerleşimi için [Masaüstü](/docs/user-guide/desktop/) konusuna bakın.

- [Temalar](/docs/user-guide/appearance/)
- [Duvar Kağıtları](/docs/argvus-appearance/wallpapers/)
- [Yazı tipleri ve simgeler](/docs/argvus-appearance/fonts-and-icons/)
- [Efektler](/docs/argvus-appearance/effects/)

Kontrol Merkezi kullanıcıya yönelik ayarları ortaya çıkarır. Uygulama `argvus-appearance`, `argvus-session`, kabuk bileşenleri ve varlık paketleri tarafından paylaşılmaktadır. Bkz. [görünüm mimarisi](/docs/developer-guide/subsystems/appearance-and-themes/).

## Pencere sallanıyor

**Masaüstü ve Giriş → Hyprland → Animasyonlar → Windows**'u açın ve
**Sallanma** kontrolleri. `wobble`'yu arayın veya doğrudan sayfayı açın:

```sh
argvus control-center hyprland animations windows
```

Wobble varsayılan olarak devre dışıdır. ARGVUS varsayılanları sertlik `100`, sönümlemedir`10`, kütle `1`, yoğunluk `1` ve ağ çözünürlüğü `8`. Sertlik ve kütle kabul edilir
`0.0001–1000`, sönümleme `0–1000`, yoğunluk `0–2` ve bir `2–32` tamsayısını meshleyin.
Daha yüksek ağ çözünürlüğü, oluşturma maliyetini artırır. Ayarlamak için ok tuşlarını kullanın
değerleri yazın veya Enter tuşuna basın. **Wobble varsayılanlarını geri yükle** taslağı değiştirir;
**Uygula** bekleyen tüm animasyon düzenlemelerini kaydeder ve Hyprland'i bir kez yeniden yükler.
oturumu yeniden başlatıyorum. Animasyonlar'a geri döndüğünüzde paylaşılan taslak korunur.

Genel Animasyonlar anahtarı, bireysel özelliğini korurken Wobble'ı bastırır
ayarlar. Kanonik değerler ARGVUS durumunda `effects.wobble_*` altında bulunur;
`hypr/state.lua` oluşturulur ve manuel olarak düzenlenmemelidir.

Bu, `argvus-hyprland` tarafından zorunlu kılınan Hyprland **0.57.0 veya daha yenisini** gerektirir
paket bağımlılığı. Sürüm 0.56 bu yapılandırma için desteklenmiyor.
maksimum yoğunluk [yayınlanan 0.57.0 kaynağını](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp) takip eder,
daha yeni belgelerde 3 listelense de bu onu `2` `3`  ile sınırlandırır.

## Görsel tema seçici

`argvus theme-selector`, parametresiz `argvus theme-switch` veya `SUPER + Shift + T` ile açın. Bağımsız GTK katmanı etkin monitörde orta önizlemeyi ve komşu kartları gösterir. Arayın veya Tümü/Koyu/Açık filtrelerini kullanın. Oklar, fare tekerleği veya yan kartlarla gezin; Enter uygular, Escape kapatır. Gezinmek masaüstünü değiştirmez. Sistem ve kullanıcı temaları mevcut kataloğu paylaşır. Geçerli `preview` görüntüsü önceliklidir; yoksa duvar kâğıdı ve palet, uygulama özelleştirmelerini tam olarak yansıtmayan örnek bir masaüstü oluşturur. Mevcut işlem Sticky/Spaced düzenini korur ve hatalarda yeniden denenebilir. JXL küçük resimleri `libjxl` içindeki `djxl` aracını gerektirir; XDG önbellek yolu `argvus/theme-selector/v2` olur.

Seçici açıldığında başlangıç odağı karusel gezinmesindedir. Aramaya odaklanmak veya odağı kaldırmak için `/` tuşuna basın; arama odaktayken yazmak temaları filtreler. **Özel**, kullanıcı tarafından yüklenen temaları; **Koyu** ve **Açık**, ilgili kategorideki sistem temalarını listeler. Arama satırı orta önizlemeyle aynı genişliktedir. Sticky tüm köşeleri düz yapar; Spaced yuvarlak köşeleri korur. Kompakt tek aralıklı yazı tipi ARGVUS masaüstü araçlarının görünümünü izler.

Uygulama sırasında orta önizleme kararır ve yükleme göstergesi görünür. Arama odağı etkin vurgu rengini kullanır; filtrenin tek kenarlığı vardır.

`argvus layout-mode-menu` (SUPER + Shift + M): ↑/↓ ile Sticky veya Spaced seçin. Sağdaki dört kart döşenmiş veya yüzen pencereleri üstte veya altta görev çubuğuyla birleştirir; ←/→ uçta başa dönmez. Seçmek için tıklayın, uygulamak için Enter, iptal için Escape kullanın. Seçili olmayan mod soluktur; önizlemeler geçerli tema ve duvar kâğıdını kullanır. Bekleyen durum görünür kalır ve hata durumunda yeniden denenebilir.
