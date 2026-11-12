---
title: ARGVUS Disk Kullanımı Analizörü
description: Temalı bir Kitty penceresinde ncdu ile disk kullanımına etkileşimli olarak göz atın.
---

Uygulama başlatıcıdan **ARGVUS Disk Kullanım Analizcisi**'ni açın veya şunu çalıştırın:

```sh
argvus du
```

Analizör özel, ortalanmış bir Kitty penceresi açar ve gerçek ncdu arayüzünü ARGVUS çerçevesinin içine yerleştirir. `$HOME`'da başlar. Dizin yolunu düzenleyin veya yapılandırılmış dosya yöneticisini açmak için **Gözat**'ı seçin. Superfile ile bir dizine gidin ve onu seçmek için **Boşluk** tuşuna basın; `q` veya `Esc` iptal eder. Diğer yapılandırılmış dosya yöneticileri normal şekilde açılır ancak seçilen dizini döndürmeyebilir, bu nedenle yolu analizciye girin veya yapıştırın.

Dizin, seçenekler ve eylemler arasında geçiş yapmak için Sekme veya ok tuşlarını kullanın. Grafik, yüzde, koyu renk şeması ve dizinlerin ilk sıralaması varsayılan olarak etkindir. Geriye kalan tarama ve görüntüleme seçenekleri **Analiz** seçilmeden önce değiştirilebilir. Sonuçlar'da ncdu'nun klavye kontrollerini kullanın; `q` forma geri döner ve `Esc` taramayı durdurur. Seçilen dosyayı veya dizini silmek için `d` tuşuna basın. ncdu onay ister; silme işlemi kalıcıdır ve mevcut kullanıcının izinlerini kullanır. Yükseltilmiş ayrıcalıklar talep edilmez.

Dış arayüz aktif ARGVUS temasını ve yerel ayarını takip eder. Kitty ve ncdu gereklidir; `argvus-devtools` paketi ncdu'yu sağlar. Kitty veya ncdu eksikse uygulama eksik bağımlılığı bildirir.
